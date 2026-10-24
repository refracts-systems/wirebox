"""Compare a bounded, identical-input Linux experiment with Wyrlex output.

No timing equality or firmware-wide conformance is inferred from these checks.
"""
import ipaddress


def checksum(data):
    padded = data + (b"\0" if len(data) % 2 else b"")
    total = sum(int.from_bytes(padded[i:i + 2], "big") for i in range(0, len(padded), 2))
    while total >> 16:
        total = (total & 0xffff) + (total >> 16)
    return (~total) & 0xffff


def ipv4(data):
    if len(data) < 20 or data[0] != 0x45:
        raise ValueError("expected complete IPv4 IHL=5")
    length = int.from_bytes(data[2:4], "big")
    if length < 28 or length > len(data) or data[9] != 1:
        raise ValueError("expected complete ICMP over IPv4")
    data = data[:length]
    return {"bytes": data, "src": str(ipaddress.IPv4Address(data[12:16])),
            "dst": str(ipaddress.IPv4Address(data[16:20])), "ttl": data[8],
            "type": data[20], "code": data[21], "payload": data[20:],
            "checksums_valid": checksum(data[:20]) == 0 and checksum(data[20:]) == 0}


def compare_case(case, simulator, captures, stats, contract, revision=2, context=None):
    if revision == 3:
        return compare_v3(case, simulator, captures, stats, contract, context)
    if revision not in (1, 2):
        raise ValueError("unsupported comparator revision")
    packet_by_id = {p["version"]: bytes.fromhex(p["hex"]) for p in simulator["packets"]}
    stimulus = packet_by_id[1]
    original = ipv4(stimulus)
    expected = contract["expected"][case]
    decoded, errors = [], []
    background_igmp = 0
    for row in captures:
        data = bytes.fromhex(row["hex"])
        if len(data) < 14 or data[12:14] != b"\x08\x00":
            continue
        if revision == 2 and len(data) >= 34 and data[14] >> 4 == 4 and data[23] == 2 and data[30] & 0xf0 == 0xe0:
            # Linux bridge multicast membership reports are instrumentation
            # background, outside this ICMP-only contract. Revision 1 is kept
            # reproducible so the initial classifier failure is never rewritten.
            background_igmp += 1
            continue
        try:
            packet = ipv4(data[14:])
        except ValueError as error:
            errors.append(str(error))
            continue
        decoded.append((row, packet, data))

    ingress = [(r, p, f) for r, p, f in decoded if r["role"] == "router"
               and r["interface"] == "eth0" and r["packet_type"] != 4
               and p["src"] == original["src"] and p["dst"] == original["dst"] and p["type"] == 8]
    forwarded = [(r, p, f) for r, p, f in decoded if r["role"] == "router"
                 and r["interface"] == "eth1" and r["packet_type"] == 4
                 and p["src"] == original["src"] and p["dst"] == original["dst"] and p["type"] == 8]
    received = [(r, p, f) for r, p, f in decoded if r["role"] == "source"
                and r["packet_type"] != 4 and p["dst"] == original["src"] and p["type"] in (0, 3, 11)]
    sim_delivered = [r for r in simulator["trace"] if r["kind"] == "delivered"]
    response = ipv4(packet_by_id[sim_delivered[-1]["version"]]) if sim_delivered else None
    sim_forwarded = [r for r in simulator["trace"] if r["kind"] == "decrement_ttl"
                     and ipv4(packet_by_id[r["prior"]])["type"] == 8]
    checks = []

    def check(name, ok, want, actual):
        checks.append({"name": name, "passed": bool(ok), "expected": want, "actual": actual})

    check("fixture_outcome", simulator["outcome"] == expected["outcome"], expected["outcome"], simulator["outcome"])
    check("identical_router_ingress_ipv4", len(ingress) == 1 and ingress[0][1]["bytes"] == stimulus,
          stimulus.hex(), [p["bytes"].hex() for _, p, _ in ingress])
    check("source_receives_expected_icmp", len(received) == 1 and received[0][1]["type"] == expected["icmp_type"]
          and received[0][1]["code"] == expected["icmp_code"],
          [expected["icmp_type"], expected["icmp_code"]], [[p["type"], p["code"]] for _, p, _ in received])
    check("valid_ip_and_icmp_checksums", not errors and bool(decoded) and original["checksums_valid"]
          and all(p["checksums_valid"] for _, p, _ in decoded), True,
          {"decode_errors": errors, "invalid_checksums": sum(not p["checksums_valid"] for _, p, _ in decoded)})
    if revision == 2:
        checks[-1]["actual"]["background_igmp_frames"] = background_igmp
    want_forward = expected["forwarded_ttl"] is not None
    forward_ok = (len(forwarded) == 1 and len(sim_forwarded) == 1
                  and forwarded[0][1]["ttl"] == expected["forwarded_ttl"]
                  and forwarded[0][1]["bytes"] == packet_by_id[sim_forwarded[0]["version"]]) if want_forward else not forwarded and not sim_forwarded
    check("forwarded_request_or_no_forward", forward_ok, expected["forwarded_ttl"],
          [p["ttl"] for _, p, _ in forwarded])
    # With fixed synthetic MAC identities the egress rewrite is exact. Padding
    # remains excluded because Linux veth captures can omit Ethernet padding.
    expected_mac_header = "0200000001020200000001010800"
    rewrite_ok = (len(forwarded) == 1 and forwarded[0][2][:14].hex() == expected_mac_header) if want_forward else not forwarded
    check("router_ethernet_rewrite", rewrite_ok, expected_mac_header if want_forward else None,
          [frame[:14].hex() for _, _, frame in forwarded])
    # Generated response IP IDs/checksum bytes are not compared across stacks;
    # each real checksum must validate above, and the ICMP payload/quote is exact.
    response_ok = response is not None and len(received) == 1 and all(
        received[0][1][key] == response[key] for key in ("src", "dst", "ttl", "payload"))
    check("icmp_payload_or_error_quote", response_ok, response["payload"].hex() if response else None,
          [p["payload"].hex() for _, p, _ in received])
    check("capture_no_kernel_drops", bool(stats) and all(s["kernel_drops"] == 0 and s["complete"] for s in stats),
          0, [s["kernel_drops"] for s in stats])
    return {"case": case, "status": "pass" if all(c["passed"] for c in checks) else "fail",
            "checks": checks, "mismatches": [c["name"] for c in checks if not c["passed"]]}


def _decoded(rows):
    result, errors = [], []
    for row in rows:
        frame = bytes.fromhex(row["hex"])
        if len(frame) < 14 or frame[12:14] != b"\x08\x00":
            continue
        if len(frame) >= 34 and frame[23] == 2 and frame[30] & 0xf0 == 0xe0:
            continue
        try:
            result.append((row, ipv4(frame[14:]), frame))
        except ValueError as error:
            errors.append(str(error))
    return result, errors


def _window(captures, probe):
    return [row for row in captures if probe["start_ns"] <= row["monotonic_ns"] <= probe["end_ns"]]


def _ingress(decoded, stimulus):
    return [p for row, p, _ in decoded if row["role"] == "router" and row["interface"] == "eth0"
            and row["packet_type"] != 4 and p["bytes"] == stimulus]


def _replies(decoded):
    return [p for row, p, _ in decoded if row["role"] == "source" and row["packet_type"] != 4
            and p["dst"] == "10.0.0.2" and p["type"] in (0, 3, 11)]


def _requests_forwarded(decoded):
    return [p for row, p, _ in decoded if row["role"] == "router" and row["interface"] == "eth1"
            and row["packet_type"] == 4 and p["type"] == 8]


def compare_v3(case, simulator, captures, stats, contract, context):
    """Different failure policies stay explicit; no runtime claim is inferred
    from absent capture alone. Negative probes require a same-path control.
    """
    if context is None:
        raise ValueError("revision 3 requires experiment context")
    experiment, observed = context["experiment"], context["observed"]
    probes = {p["name"]: p for p in experiment["probes"]}
    if len(probes) != len(experiment["probes"]):
        raise ValueError("duplicate probe window")
    primary_name = "after" if case == "R01" else "primary"
    probe = probes[primary_name]
    stimulus = bytes.fromhex(simulator["packets"][0]["hex"])
    primary = _window(captures, probe)
    decoded, errors = _decoded(primary)
    checks = []

    def check(name, ok, expected, actual):
        checks.append({"name": name, "passed": bool(ok), "expected": expected, "actual": actual})

    def route_present(state):
        return any(r.get("dst") == "198.51.100.2" and r.get("gateway") == "10.0.1.2"
                   and r.get("dev") == "eth1" for r in state["r1"]["routes"])

    before, after = observed["before"], observed["after"]
    preconditions = (before["r1"]["sysctls"]["ip_forward"] == 1
                     and before["pc2"]["sysctls"]["arp_ignore"] == (8 if case == "S04" else 0)
                     and any(a.get("local") == "10.0.0.2" for link in before["pc1"]["addresses"]
                             for a in link.get("addr_info", []))
                     and all("UP" in link.get("flags", []) for role in ("pc1", "r1", "pc2")
                             for link in before[role]["links"] if link["ifname"].startswith("eth"))
                     and probe["input_hex"] == stimulus.hex()
                     and probe["end_ns"] >= probe["start_ns"])
    capture_ok = len(stats) == 2 and all(s["complete"] and s["kernel_drops"] == 0 for s in stats)

    if case in ("S01", "S02", "S03", "R01"):
        result = compare_case(case, simulator, primary, stats, contract, revision=2)
        checks.extend(result["checks"])
        if case == "R01":
            before_probe = probes["before"]
            missing = compare_case("S02", context["before_simulator"], _window(captures, before_probe),
                                   stats, contract, revision=2)
            check("route_missing_before_change", missing["status"] == "pass" and not route_present(before)
                  and before_probe["input_hex"] == stimulus.hex(), "missing route and ICMP 3/0",
                  {"comparison": missing, "route_present": route_present(before)})
            action = experiment["route_add"]
            check("route_added_after_failure", before_probe["end_ns"] <= action["start_ns"]
                  <= action["end_ns"] <= probe["start_ns"] and route_present(after),
                  "route installed between failed and successful probes", action)
        else:
            preconditions = preconditions and route_present(before) == (case != "S02")
    elif case == "S04":
        check("fixture_outcome", simulator["outcome"] == "arp_timeout", "arp_timeout", simulator["outcome"])
        check("identical_router_ingress_ipv4", len(_ingress(decoded, stimulus)) == 1,
              stimulus.hex(), [p["bytes"].hex() for p in _ingress(decoded, stimulus)])
        arp = [(r, bytes.fromhex(r["hex"])) for r in primary if r["role"] == "router" and r["interface"] == "eth1"]
        requests = [f for r, f in arp if len(f) >= 42 and f[12:14] == b"\x08\x06"
                    and f[20:22] == b"\0\1" and f[38:42] == b"\x0a\0\x01\x02" and r["packet_type"] == 4]
        answers = [f for r, f in arp if len(f) >= 42 and f[12:14] == b"\x08\x06"
                   and f[20:22] == b"\0\2" and f[28:32] == b"\x0a\0\x01\x02"]
        check("arp_requests_without_reply", bool(requests) and not answers, "one or more requests, no answer",
              {"requests": len(requests), "answers": len(answers)})
        replies = _replies(decoded)
        check("arp_failure_result", len(replies) == 1 and (replies[0]["type"], replies[0]["code"]) == (3, 1)
              and simulator["pending_arp"] == 0 and simulator["pending_events"] == 0,
              "Linux ICMP 3/1; model internal arp_timeout with drained work",
              {"linux": [[p["type"], p["code"]] for p in replies], "model": simulator["outcome"]})
        check("no_forwarded_request", not _requests_forwarded(decoded), [],
              [p["bytes"].hex() for p in _requests_forwarded(decoded)])
        check("valid_observed_checksums", not errors and bool(decoded) and all(p["checksums_valid"] for _, p, _ in decoded),
              True, {"decode_errors": errors, "invalid": sum(not p["checksums_valid"] for _, p, _ in decoded)})
        check("capture_no_kernel_drops", capture_ok, 0, [s["kernel_drops"] for s in stats])
    elif case in ("N01", "N02"):
        control = simulator["control"]
        good = bytes.fromhex(control["packets"][0]["hex"])
        control_probe = probes["control"]
        control_rows, control_errors = _decoded(_window(captures, control_probe))
        control_replies = _replies(control_rows)
        control_packets = {p["version"]: bytes.fromhex(p["hex"]) for p in control["packets"]}
        received = [r for r in control["trace"] if r["reason"] == "unsolicited_reply"
                    and r["entity"] == control["entities"]["PC1"]]
        expected_reply = ipv4(control_packets[received[-1]["version"]]) if received else None
        fault_ok = (checksum(stimulus[:20]) != 0 and checksum(stimulus[20:]) == 0) if case == "N01" else (
            checksum(stimulus[:20]) == 0 and checksum(stimulus[20:]) != 0 and ipv4(stimulus)["dst"] == "10.0.0.1")
        check("fixture_outcome", simulator["outcome"] == "invalid_packet", "invalid_packet", simulator["outcome"])
        check("checksum_fault_preserved", fault_ok and len(_ingress(decoded, stimulus)) == 1,
              stimulus.hex(), [p["bytes"].hex() for p in _ingress(decoded, stimulus)])
        check("invalid_input_silent", not errors and not _replies(decoded) and not _requests_forwarded(decoded),
              "no reply or forwarded request during primary window", {"decode_errors": errors,
                  "replies": [p["bytes"].hex() for p in _replies(decoded)], "forwarded": len(_requests_forwarded(decoded))})
        control_ok = (not control_errors and len(_ingress(control_rows, good)) == 1 and len(control_replies) == 1
                      and all(p["checksums_valid"] for _, p, _ in control_rows) and expected_reply is not None
                      and all(control_replies[0][key] == expected_reply[key] for key in ("src", "dst", "ttl", "payload")))
        check("valid_control_recovers", control_ok and control_probe["input_hex"] == good.hex(),
              "corrected same-path packet elicits the model's reply bytes",
              {"ingress": len(_ingress(control_rows, good)), "replies": [p["bytes"].hex() for p in control_replies]})
        duration = probe["end_ns"] - probe["start_ns"]
        check("quiet_window_bounded", duration >= 1_000_000_000 and probe["end_ns"] <= control_probe["start_ns"],
              "at least 1 second before the healthy control", duration)
        check("capture_no_kernel_drops", capture_ok, 0, [s["kernel_drops"] for s in stats])
        check("model_rejects_without_transmit", simulator["reason"] == ("invalid_ipv4_checksum" if case == "N01" else "invalid_icmp_checksum")
              and simulator["transmissions"] == 0 and simulator["pending_events"] == 0,
              "bounded rejection without scheduled work", {"reason": simulator["reason"], "transmissions": simulator["transmissions"]})
        preconditions = preconditions and probe["sent_via"] == control_probe["sent_via"] == "AF_PACKET sw1:p1"
    else:
        raise ValueError("unknown revision 3 case")
    check("lab_preconditions", preconditions, True, {"ip_forward": before["r1"]["sysctls"]["ip_forward"],
          "arp_ignore": before["pc2"]["sysctls"]["arp_ignore"], "stimulus_matches": probe["input_hex"] == stimulus.hex()})
    return {"case": case, "status": "pass" if all(c["passed"] for c in checks) else "fail", "checks": checks,
            "mismatches": [c["name"] for c in checks if not c["passed"]]}
