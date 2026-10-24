"""Isolated Linux namespace experiment; never attach a physical/WSL interface.

The Windows host exports Wyrlex output. This Linux runner sends its original
IPv4 input through Linux and records independent kernel behavior and captures.
"""
import argparse
import datetime as dt
import hashlib
import importlib.metadata
import json
import os
from pathlib import Path
import platform
import re
import select
import signal
import socket
import struct
import subprocess
import sys
import time
import uuid

from compare import compare_case, ipv4


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


def read_json(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def utc():
    return dt.datetime.now(dt.timezone.utc).isoformat().replace("+00:00", "Z")


def slug(value):
    result = re.sub(r"[^a-z0-9._-]+", "-", value.lower()).strip(".-")
    if not result:
        raise ValueError("empty target path component")
    return result


def capture(args):
    running = True

    def stop(_signal, _frame):
        nonlocal running
        running = False

    signal.signal(signal.SIGTERM, stop)
    sockets = []
    count = 0
    out = Path(args.output)
    with out.with_suffix(".jsonl").open("w", encoding="utf-8") as records, out.with_suffix(".pcap").open("wb") as pcap:
        pcap.write(struct.pack("<IHHIIII", 0xa1b2c3d4, 2, 4, 0, 0, 65535, 1))
        try:
            for interface in args.interfaces:
                sock = socket.socket(socket.AF_PACKET, socket.SOCK_RAW, socket.htons(3))
                sock.bind((interface, 0))
                sockets.append(sock)
            print("ready", flush=True)
            while running:
                readable, _, _ = select.select(sockets, [], [], 0.1)
                for sock in readable:
                    data, address = sock.recvfrom(65535)
                    if len(data) < 14 or data[12:14] not in (b"\x08\x00", b"\x08\x06"):
                        continue
                    now = time.time_ns()
                    row = {"role": args.role, "interface": address[0], "packet_type": address[2],
                           "received_unix_ns": now, "monotonic_ns": time.monotonic_ns(), "hex": data.hex()}
                    records.write(json.dumps(row) + "\n")
                    records.flush()
                    pcap.write(struct.pack("<IIII", now // 1_000_000_000, (now // 1000) % 1_000_000, len(data), len(data)))
                    pcap.write(data)
                    count += 1
            # Linux PACKET_STATISTICS (if_packet.h): tp_packets, tp_drops.
            drops = sum(struct.unpack("II", sock.getsockopt(263, 6, 8))[1] for sock in sockets)
            write_json(out.with_suffix(".stats.json"), {"complete": True, "kernel_drops": drops, "captured_frames": count})
        finally:
            for sock in sockets:
                sock.close()


class Lab:
    def __init__(self, folder):
        self.folder = folder
        self.names = {name: "wyrlex-p5-" + uuid.uuid4().hex[:10] + "-" + name for name in ("pc1", "sw1", "r1", "pc2")}
        self.created = []
        self.processes = []
        self.log = []

    def command(self, *args, check=True):
        result = subprocess.run(args, text=True, capture_output=True, timeout=15)
        self.log.append({"argv": list(args), "exit_code": result.returncode, "stdout": result.stdout, "stderr": result.stderr})
        if check and result.returncode:
            raise RuntimeError(f"command failed: {list(args)}: {result.stderr}")
        return result.stdout

    def ip(self, name, *args):
        return self.command("ip", "-n", self.names[name], *args)

    def within(self, name, *args):
        return self.command("ip", "netns", "exec", self.names[name], *args)

    def setup(self, case):
        for name, namespace in self.names.items():
            self.command("ip", "netns", "add", namespace)
            self.created.append(namespace)
            self.within(name, "sysctl", "-qw", "net.ipv6.conf.all.disable_ipv6=1", "net.ipv6.conf.default.disable_ipv6=1",
                        "net.ipv4.conf.all.rp_filter=0", "net.ipv4.conf.default.rp_filter=0",
                        "net.ipv4.conf.all.send_redirects=0", "net.ipv4.conf.default.send_redirects=0",
                        "net.ipv4.ip_default_ttl=64", "net.ipv4.icmp_echo_ignore_all=0")
            self.ip(name, "link", "set", "lo", "up")
        for left, left_port, right, right_port in [("pc1", "eth0", "sw1", "p0"), ("sw1", "p1", "r1", "eth0"), ("r1", "eth1", "pc2", "eth0")]:
            self.ip(left, "link", "add", left_port, "type", "veth", "peer", "name", right_port, "netns", self.names[right])
        self.ip("sw1", "link", "add", "br0", "type", "bridge", "stp_state", "0", "forward_delay", "0")
        for name, port, mac in [("pc1", "eth0", "02:00:00:00:00:02"), ("sw1", "p0", "02:00:00:00:ff:10"),
                                ("sw1", "p1", "02:00:00:00:ff:11"), ("sw1", "br0", "02:00:00:00:ff:01"),
                                ("r1", "eth0", "02:00:00:00:00:01"), ("r1", "eth1", "02:00:00:00:01:01"), ("pc2", "eth0", "02:00:00:00:01:02")]:
            self.ip(name, "link", "set", "dev", port, "address", mac)
        for port in ("p0", "p1"):
            self.ip("sw1", "link", "set", port, "master", "br0")
        for name, port, address in [("pc1", "eth0", "10.0.0.2/24"), ("r1", "eth0", "10.0.0.1/24"),
                                    ("r1", "eth1", "10.0.1.1/24"), ("pc2", "eth0", "10.0.1.2/24"), ("pc2", "lo", "198.51.100.2/32")]:
            self.ip(name, "address", "add", address, "dev", port)
        for name, ports in [("pc1", ["eth0"]), ("sw1", ["br0", "p0", "p1"]), ("r1", ["eth0", "eth1"]), ("pc2", ["eth0"])]:
            for port in ports:
                self.ip(name, "link", "set", port, "up")
        self.ip("pc1", "route", "add", "default", "via", "10.0.0.1")
        self.ip("pc2", "route", "add", "default", "via", "10.0.1.1")
        self.within("r1", "sysctl", "-qw", "net.ipv4.ip_forward=1")
        if case == "S04":
            # Keep carrier up so ARP attempts are observable. This establishes
            # no-answer semantics, not equivalence to a physical/admin link down.
            self.within("pc2", "sysctl", "-qw", "net.ipv4.conf.eth0.arp_ignore=8")
        if case not in ("S02", "R01"):
            self.ip("r1", "route", "add", "198.51.100.2/32", "via", "10.0.1.2")

    def observe(self):
        result = {name: {kind: json.loads(self.ip(name, "-j", *args)) for kind, args in
                       [("links", ["link", "show"]), ("addresses", ["address", "show"]), ("routes", ["route", "show"]), ("neighbors", ["neighbor", "show"])]}
                for name in self.names}
        result["r1"]["sysctls"] = {"ip_forward": int(self.within("r1", "sysctl", "-n", "net.ipv4.ip_forward"))}
        result["pc2"]["sysctls"] = {"arp_ignore": int(self.within("pc2", "sysctl", "-n", "net.ipv4.conf.eth0.arp_ignore"))}
        return result

    def probe(self, name, stimulus, raw=False, quiet=False):
        start = time.monotonic_ns()
        if raw:
            # AF_INET/IP_HDRINCL can recalculate the IPv4 header checksum.
            # Inject at SW1:p1 so the exact malformed bytes arrive at R1:eth0.
            frame = bytes.fromhex("0200000000010200000000020800") + stimulus
            self.within("sw1", sys.executable, "-c",
                        "import socket,sys;s=socket.socket(socket.AF_PACKET,socket.SOCK_RAW);s.bind(('p1',0));s.send(bytes.fromhex(sys.argv[1]));s.close()", frame.hex())
        else:
            self.within("pc1", sys.executable, "-c",
                        "import socket,sys;s=socket.socket(socket.AF_INET,socket.SOCK_RAW,socket.IPPROTO_RAW);s.setsockopt(socket.IPPROTO_IP,socket.IP_HDRINCL,1);s.sendto(bytes.fromhex(sys.argv[1]),(sys.argv[2],0));s.close()",
                        stimulus.hex(), ipv4(stimulus)["dst"])
        deadline = time.monotonic() + (1.0 if quiet else 6.0)
        while time.monotonic() < deadline:
            replies = []
            for line in (self.folder / "source.jsonl").read_text().splitlines():
                try:
                    row = json.loads(line)
                except json.JSONDecodeError:
                    continue  # A live reader may see the final line mid-write.
                frame = bytes.fromhex(row["hex"])
                if row["monotonic_ns"] >= start and row["packet_type"] != 4 and len(frame) >= 42 and frame[12:14] == b"\x08\x00" and frame[23] == 1:
                    packet = ipv4(frame[14:])
                    if packet["dst"] == "10.0.0.2" and packet["type"] in (0, 3, 11):
                        replies.append(row)
            if replies and not quiet:
                break
            time.sleep(0.025)
        time.sleep(0.05)
        return {"name": name, "input_hex": stimulus.hex(), "start_ns": start, "end_ns": time.monotonic_ns(),
                "sent_via": "AF_PACKET sw1:p1" if raw else "AF_INET IP_HDRINCL pc1:eth0"}

    def start_captures(self):
        for name, role, ports in [("pc1", "source", ["eth0"]), ("r1", "router", ["eth0", "eth1"])]:
            stderr = (self.folder / f"{role}.stderr.log").open("w")
            process = subprocess.Popen(["ip", "netns", "exec", self.names[name], sys.executable, str(Path(__file__).resolve()),
                                        "capture", "--role", role, "--interfaces", *ports, "--output", str(self.folder / role)],
                                       stdout=subprocess.PIPE, stderr=stderr, text=True)
            self.processes.append((process, stderr))
            ready, _, _ = select.select([process.stdout], [], [], 5)
            if not ready or process.stdout.readline().strip() != "ready":
                raise RuntimeError("capture did not become ready")

    def stop_captures(self):
        for process, stderr in self.processes:
            if process.poll() is None:
                process.terminate()
            try:
                result = process.wait(timeout=3)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
                result = -1
            stderr.close()
            if result != 0:
                raise RuntimeError("capture failed or did not stop cleanly")
        self.processes.clear()

    def close(self):
        # Cleanup only names successfully created by this Lab. No host links,
        # routes or namespace-wide settings outside these namespaces are edited.
        for process, stderr in self.processes:
            if process.poll() is None:
                process.kill()
            process.wait(timeout=3)
            stderr.close()
        self.processes.clear()
        failures = []
        for namespace in reversed(self.created):
            self.command("ip", "netns", "delete", namespace, check=False)
            if self.log[-1]["exit_code"]:
                failures.append(namespace)
        self.created.clear()
        write_json(self.folder / "commands.json", self.log)
        write_json(self.folder / "cleanup.json", {"complete": not failures, "failed_namespaces": failures})
        if failures:
            raise RuntimeError(f"namespace cleanup failed: {failures}")


def run(args):
    if os.geteuid() != 0:
        raise RuntimeError("namespace experiment requires root in WSL")
    if "microsoft" not in platform.release().lower():
        raise RuntimeError("this initial runner records WSL targets only")
    repo, inputs = Path(args.repo).resolve(), Path(args.input).resolve()
    if not args.run_id.startswith("linux-forwarding-") or any(c not in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-" for c in args.run_id):
        raise ValueError("invalid run id")
    if not 1 <= args.repetitions <= 20:
        raise ValueError("repetitions must be 1..20")
    distro = dict(line.rstrip().split("=", 1) for line in Path("/etc/os-release").read_text().splitlines() if "=" in line)
    identity = {"vendor": "Linux", "model": "WSL2 network namespaces", "architecture": platform.machine(),
                "distribution": distro["PRETTY_NAME"].strip('"'), "distribution_version": distro["VERSION_ID"].strip('"'),
                "kernel_version": platform.release(), "firmware_image_sha256": None,
                "software": [{"component": "linux-kernel", "version": platform.release(), "sha256": None},
                             {"component": "distribution", "version": distro["VERSION_ID"].strip('"'), "sha256": None}],
                "tools": {"iproute2": subprocess.check_output(["ip", "-Version"], text=True).strip(),
                          "python": platform.python_version(), "jsonschema": importlib.metadata.version("jsonschema")}}
    identity_hash = hashlib.sha256(json.dumps(identity, sort_keys=True).encode()).hexdigest()
    model = slug(f"{distro['ID'].strip(chr(34))}-{identity['distribution_version']}-wsl2-{platform.machine()}")
    version = slug(platform.release()) + "-" + identity_hash[:8]
    target_id = "linux-" + identity_hash[:16]
    target_dir = repo / "corpus/devices/linux" / model / version
    target_dir.mkdir(parents=True, exist_ok=True)
    target_path = target_dir / "target.json"
    target = {"schema_version": 1, "target_id": target_id, "family": "linux", "identity": identity}
    if target_path.exists() and read_json(target_path) != target:
        raise ValueError("target identity collision")
    if not target_path.exists():
        write_json(target_path, target)
    folder = target_dir / "runs" / args.run_id
    folder.mkdir(parents=True, exist_ok=False)
    contract_path = repo / "corpus/scenarios/linux-static-ipv4-v3.json"
    contract = read_json(contract_path)
    for filename in ["engine-provenance.json", *[f"{case}.json" for case in contract["cases"]]]:
        (folder / filename).write_bytes((inputs / filename).read_bytes())
    write_json(folder / "scenario.json", contract)
    archived_sources = []
    (folder / "runner-sources").mkdir()
    for name in ("linux_lab.py", "compare.py", "validate.py", "run-linux-lab.ps1"):
        source = repo / "scripts/corpus" / name
        archived = folder / "runner-sources" / name
        archived.write_bytes(source.read_bytes())
        archived_sources.append({"path": archived.relative_to(folder).as_posix(), "sha256": digest(archived)})
    started_at = utc()
    comparisons = []
    try:
        for case in contract["cases"]:
            simulator = read_json(folder / f"{case}.json")
            stimulus = bytes.fromhex(simulator["packets"][0]["hex"])
            packet = ipv4(stimulus)
            if packet["src"] != "10.0.0.2" or packet["type"] != 8 or (packet["checksums_valid"] == (case in ("N01", "N02"))):
                raise ValueError("unexpected simulator stimulus")
            for repetition in range(1, args.repetitions + 1):
                case_dir = folder / f"{case}-r{repetition}"
                case_dir.mkdir()
                lab = Lab(case_dir)
                try:
                    lab.setup(case)
                    before = lab.observe()
                    lab.start_captures()
                    experiment = {"probes": []}
                    if case == "R01":
                        experiment["probes"].append(lab.probe("before", stimulus))
                        action_start = time.monotonic_ns()
                        lab.ip("r1", "route", "add", "198.51.100.2/32", "via", "10.0.1.2")
                        experiment["route_add"] = {"start_ns": action_start, "end_ns": time.monotonic_ns()}
                        experiment["probes"].append(lab.probe("after", stimulus))
                    else:
                        raw = case in ("N01", "N02")
                        experiment["probes"].append(lab.probe("primary", stimulus, raw=raw, quiet=raw))
                        if raw:
                            good = bytes.fromhex(simulator["control"]["packets"][0]["hex"])
                            experiment["probes"].append(lab.probe("control", good, raw=True))
                    lab.stop_captures()
                    write_json(case_dir / "experiment.json", experiment)
                    write_json(case_dir / "observed.json", {"before": before, "after": lab.observe()})
                finally:
                    lab.close()
                captures, stats = [], []
                for role in ("source", "router"):
                    captures.extend(json.loads(line) for line in (case_dir / f"{role}.jsonl").read_text().splitlines())
                    stats.append(read_json(case_dir / f"{role}.stats.json"))
                context = {"experiment": read_json(case_dir / "experiment.json"), "observed": read_json(case_dir / "observed.json")}
                if case == "R01":
                    context["before_simulator"] = read_json(folder / "S02.json")
                result = compare_case(case, simulator, captures, stats, contract, revision=3, context=context)
                result["repetition"] = repetition
                comparisons.append(result)
        status = "pass" if comparisons and all(c["status"] == "pass" for c in comparisons) else "fail"
        write_json(folder / "comparison.json", {"schema_version": 2, "comparator_revision": 3, "scenario_id": contract["scenario_id"],
                                               "status": status, "comparisons": comparisons,
                                               "scope": "IPv4/ICMP forwarding subset; no timing or firmware conformance"})
        artifacts = [{"path": p.relative_to(folder).as_posix(), "sha256": digest(p), "bytes": p.stat().st_size}
                     for p in sorted(folder.rglob("*")) if p.is_file()]
        record = {"schema_version": 1, "record_id": args.run_id, "target_id": target_id,
                  "scenario_id": contract["scenario_id"], "status": status, "evidence_status": "locally_verified",
                  "started_at": started_at, "finished_at": utc(), "repetitions": args.repetitions,
                  "target": {"path": target_path.relative_to(repo).as_posix(), "sha256": digest(target_path)},
                  "scenario": {"path": contract_path.relative_to(repo).as_posix(), "sha256": digest(contract_path)},
                  "runner_sources": archived_sources,
                  "environment": {"wsl_distribution_name": args.distro_name, "network": "isolated namespaces and veth only"},
                  "comparison": "comparison.json", "artifacts": artifacts,
                  "limits": contract["comparison_policy"]["excluded"],
                  "sharing": "synthetic lab only; no customer or physical-device data"}
        write_json(folder / "record.json", record)
        print(json.dumps({"record": (folder / "record.json").relative_to(repo).as_posix(), "status": status,
                          "comparisons": len(comparisons), "mismatches": [c for c in comparisons if c["status"] != "pass"]}, indent=2))
        return 0 if status == "pass" else 1
    except Exception as error:
        write_json(folder / "failure.json", {"error": str(error), "finished_at": utc(), "comparisons_completed": comparisons})
        raise


def main():
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="command", required=True)
    runner = sub.add_parser("run")
    for name in ("repo", "input", "run-id", "distro-name"):
        runner.add_argument("--" + name, required=True)
    runner.add_argument("--repetitions", type=int, default=3)
    cap = sub.add_parser("capture")
    cap.add_argument("--role", required=True)
    cap.add_argument("--interfaces", nargs="+", required=True)
    cap.add_argument("--output", required=True)
    args = parser.parse_args()
    if args.command == "capture":
        capture(args)
        return 0
    return run(args)


if __name__ == "__main__":
    sys.exit(main())
