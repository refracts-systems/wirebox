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


def compare_case(case, simulator, captures, stats, contract, revision=2):
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
