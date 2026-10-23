use serde_json::{Value, json};
use std::{collections::BTreeMap, error::Error, net::Ipv4Addr};
use wirebox_kernel::{
    Error as KernelError,
    network::{
        Field, Network, Packet, Summary, Trace, Transmission, field_name, kind_name, reason_name,
    },
    simulation::DeviceRole,
};
type Result<T> = std::result::Result<T, Box<dyn Error>>;
const FIXTURE: &str = include_str!("../../../fixtures/scenarios/icmp-static-v1.json");
const GOLDEN: &str = include_str!("../../../fixtures/packets/ipv4-forwarding-v1.json");
fn ensure(ok: bool, message: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(message.to_owned().into())
    }
}
fn ip(text: &str) -> Result<u32> {
    Ok(u32::from(text.parse::<Ipv4Addr>()?))
}
fn cidr(text: &str) -> Result<(u32, u32)> {
    let (address, prefix) = text.split_once('/').ok_or("missing prefix")?;
    Ok((ip(address)?, prefix.parse()?))
}
fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value[key]
        .as_str()
        .ok_or_else(|| format!("missing {key}").into())
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn unhex(value: &str) -> Result<Vec<u8>> {
    ensure(value.len().is_multiple_of(2), "odd hex length")?;
    (0..value.len())
        .step_by(2)
        .map(|i| Ok(u8::from_str_radix(&value[i..i + 2], 16)?))
        .collect()
}
/// Independent host oracle: sum network-order words first, then fold carries.
fn checksum(bytes: &[u8]) -> u16 {
    let mut sum: u64 = bytes
        .chunks(2)
        .map(|pair| u64::from(pair[0]) * 256 + u64::from(*pair.get(1).unwrap_or(&0)))
        .sum();
    while sum > 65535 {
        sum = (sum & 65535) + (sum >> 16);
    }
    !(sum as u16)
}
fn setup(case: &str, retention: usize) -> Result<(Network, BTreeMap<String, u64>)> {
    let fixture: Value = serde_json::from_str(FIXTURE)?;
    let mut net = Network::new(fixture["clock"]["seed"].as_u64().unwrap(), retention)?;
    let mut ids = BTreeMap::new();
    for node in fixture["nodes"].as_array().unwrap() {
        let name = text(node, "id")?;
        let role = match text(node, "kind")? {
            "host" => DeviceRole::Host,
            "switch" => DeviceRole::Switch,
            "router" => DeviceRole::Router,
            _ => return Err("unknown role".into()),
        };
        let dev = net.device(role)?;
        ids.insert(name.to_owned(), dev);
        for port in node["ports"].as_array().unwrap() {
            let name = format!("{name}:{}", text(port, "id")?);
            let id = net.port(dev)?;
            ids.insert(name.clone(), id);
            let mac = port["mac"]
                .as_str()
                .map(|s| u64::from_str_radix(&s.replace(':', ""), 16))
                .transpose()?
                .unwrap_or(0);
            let (address, prefix) = port["ipv4"]
                .as_array()
                .and_then(|a| a.first())
                .and_then(Value::as_str)
                .map(cidr)
                .transpose()?
                .unwrap_or((0, 0));
            net.configure_port(
                id,
                mac,
                address,
                prefix,
                !(case == "S04" && name == "PC2:eth0"),
            )?;
        }
        if let Some(aliases) = node["local_ipv4"].as_array() {
            for alias in aliases {
                net.alias(dev, cidr(alias.as_str().unwrap())?.0)?;
            }
        }
        for route in node["routes"].as_array().unwrap() {
            if case == "S02" && name == "R1" && route["prefix"] == "198.51.100.2/32" {
                continue;
            }
            let (network, prefix) = cidr(text(route, "prefix")?)?;
            net.route(
                dev,
                ids[&format!("{name}:{}", text(route, "port")?)],
                network,
                prefix,
                ip(text(route, "via")?)?,
            )?;
        }
    }
    for edge in fixture["edges"].as_array().unwrap() {
        let endpoints = edge["endpoints"].as_array().unwrap();
        let link = net.link(
            ids[endpoints[0].as_str().unwrap()],
            ids[endpoints[1].as_str().unwrap()],
            fixture["links"]["bitrate_bps"].as_u64().unwrap(),
            fixture["links"]["propagation_ns"].as_u64().unwrap(),
        )?;
        ids.insert(text(edge, "id")?.to_owned(), link);
    }
    Ok((net, ids))
}
#[derive(Debug, PartialEq, Eq)]
struct Report {
    summary: Summary,
    packets: Vec<Packet>,
    fields: Vec<Vec<Field>>,
    trace: Vec<Trace>,
    txs: Vec<Transmission>,
    ids: BTreeMap<String, u64>,
}
fn collect(net: Network, ids: BTreeMap<String, u64>) -> Result<Report> {
    let summary = net.summary()?;
    ensure(
        summary.failed == 0,
        "network failed; outcome cannot be trusted",
    )?;
    let mut packets = Vec::new();
    let mut fields = Vec::new();
    for id in 1..=summary.packet_versions {
        packets.push(net.packet(id)?);
        fields.push(match net.fields(id) {
            Ok(view) => view,
            Err(KernelError::InvalidPacket | KernelError::Unsupported) => Vec::new(),
            Err(e) => return Err(e.into()),
        });
    }
    let mut txs = Vec::new();
    for id in 1..=summary.transmissions {
        txs.push(net.transmission(id)?);
    }
    let mut trace = Vec::new();
    loop {
        let after = trace.last().map_or(0, |t: &Trace| t.ordinal);
        let batch = net.trace(after, 256)?;
        if batch.is_empty() {
            break;
        }
        trace.extend(batch);
    }
    net.close()?;
    Ok(Report {
        summary,
        packets,
        fields,
        trace,
        txs,
        ids,
    })
}
fn run(case: &str, retention: usize, stepwise: bool) -> Result<Report> {
    let (mut net, ids) = setup(case, retention)?;
    let fixture: Value = serde_json::from_str(FIXTURE)?;
    if case == "alias" {
        net.ping(ids["PC1:eth0"], ip("198.51.100.2")?, 64)?;
    } else {
        let scenario = fixture["scenarios"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["id"] == case)
            .ok_or("unknown scenario")?;
        if case == "S05" {
            net.inject(ids["PC1:eth0"], 1, &unhex(text(scenario, "input_hex")?)?)?;
        } else {
            net.ping(
                ids["PC1:eth0"],
                ip(text(scenario, "destination")?)?,
                scenario["ttl"].as_u64().unwrap() as u32,
            )?;
        }
    }
    for _ in 0..1024 {
        let progress = net.run_until(5_000_000_000, if stepwise { 1 } else { 512 })?;
        if progress.reason == 1 {
            return collect(net, ids);
        }
        ensure(
            stepwise && progress.reason == 2,
            "unexpected budget exhaustion",
        )?;
        let _visible = net.trace(0, 1)?;
    }
    Err("execution did not reach horizon within 1024 bounded calls".into())
}
fn packet(report: &Report, version: u64) -> &Packet {
    &report.packets[(version - 1) as usize]
}
fn validate(report: &Report, case: &str) -> Result<()> {
    let expected = match case {
        "S01" | "alias" => 1,
        "S02" => 2,
        "S03" => 3,
        "S04" => 4,
        "S05" => 5,
        _ => return Err("unknown case".into()),
    };
    ensure(
        report.summary.outcome == expected,
        &format!("{case}: incorrect outcome {}", report.summary.outcome),
    )?;
    ensure(
        report.summary.pending_events == 0 && report.summary.pending_arp == 0,
        "pending work leaked",
    )?;
    for p in &report.packets {
        ensure(
            p.parent_version < p.version_id,
            "version ancestry is not acyclic",
        )?;
        ensure(p.packet_id > 0, "packet identity missing")?;
        if case == "S05" {
            continue;
        }
        let bytes = &p.bytes;
        let offset = if p.layer == 2 {
            ensure(
                bytes.len() >= 60 && bytes.len() <= 1514,
                "encoded frame length invalid",
            )?;
            if bytes[12..14] == [8, 6] {
                ensure(
                    bytes[14..21] == [0, 1, 8, 0, 6, 4, 0] && matches!(bytes[21], 1 | 2),
                    "ARP wire header invalid",
                )?;
                ensure(
                    bytes[42..].iter().all(|b| *b == 0),
                    "ARP padding not initialized",
                )?;
                continue;
            }
            ensure(bytes[12..14] == [8, 0], "unexpected ethertype")?;
            14
        } else {
            0
        };
        let ip = &bytes[offset..];
        let total = u16::from_be_bytes([ip[2], ip[3]]) as usize;
        ensure(
            total >= 28 && total <= ip.len(),
            "IPv4 total length invalid",
        )?;
        ensure(checksum(&ip[..20]) == 0, "host oracle: bad IPv4 checksum")?;
        ensure(
            checksum(&ip[20..total]) == 0,
            "host oracle: bad ICMP checksum",
        )?;
        ensure(
            ip[total..].iter().all(|b| *b == 0),
            "Ethernet padding not initialized",
        )?;
    }
    for (p, fields) in report.packets.iter().zip(&report.fields) {
        for f in fields {
            let first = f.byte_offset as usize * 8 + f.bit_offset as usize;
            ensure(
                first + f.bit_width as usize <= p.bytes.len() * 8,
                "field extends beyond version bytes",
            )?;
            if f.bit_width <= 64 {
                let mut value = 0_u64;
                for bit in first..first + f.bit_width as usize {
                    value = (value << 1) | u64::from((p.bytes[bit / 8] >> (7 - bit % 8)) & 1);
                }
                ensure(value == f.value, "field does not describe actual byte bits")?;
            }
        }
    }
    let mut free = BTreeMap::new();
    for tx in &report.txs {
        let bytes = &packet(report, tx.version).bytes;
        let delay = (bytes.len() as u64 * 8 * 1_000_000_000).div_ceil(100_000_000);
        ensure(
            tx.arrival_ns == tx.start_ns + delay + 1000,
            "serialization/propagation mismatch",
        )?;
        ensure(
            tx.start_ns >= *free.get(&tx.from_port).unwrap_or(&0),
            "egress serialization overlaps",
        )?;
        free.insert(tx.from_port, tx.start_ns + delay);
        let arrival = report
            .trace
            .iter()
            .find(|r| r.kind == 11 && r.transmission == tx.id)
            .ok_or("missing frame arrival trace")?;
        ensure(
            arrival.event_id == tx.event_id
                && arrival.cause_id == tx.cause_id
                && arrival.time_ns == tx.arrival_ns,
            "event/transmission causality mismatch",
        )?;
    }
    if case == "S01" {
        let row = report
            .trace
            .iter()
            .find(|r| {
                r.kind == 13
                    && r.entity == report.ids["R1"]
                    && packet(report, r.prior).bytes[20] == 8
            })
            .ok_or("missing request forwarding")?;
        let golden: Value = serde_json::from_str(GOLDEN)?;
        let before = &packet(report, row.prior).bytes;
        let after = &packet(report, row.version).bytes;
        ensure(
            before[..20] == unhex(text(&golden, "before_hex")?)?,
            "frozen before header mismatch",
        )?;
        ensure(
            after[..20] == unhex(text(&golden, "after_hex")?)?,
            "frozen after header mismatch",
        )?;
        ensure(
            before[20..] == after[20..],
            "forwarding changed ICMP payload",
        )?;
        ensure(
            packet(report, row.prior).packet_id == packet(report, row.version).packet_id,
            "forward changed logical packet ID",
        )?;
        ensure(
            report.trace.iter().any(|r| r.kind == 7)
                && report.trace.iter().any(|r| r.kind == 8)
                && report.trace.iter().any(|r| r.kind == 9),
            "switch learning/flood/unicast path missing",
        )?;
        let forwarded_frame = report
            .txs
            .iter()
            .find(|tx| {
                tx.from_port == report.ids["R1:eth1"] && {
                    let p = packet(report, tx.version);
                    p.bytes[12..14] == [8, 0] && p.bytes[34] == 8
                }
            })
            .ok_or("forwarded Ethernet frame missing")?;
        ensure(
            packet(report, forwarded_frame.version).bytes[..12]
                == unhex("020000000102020000000101")?,
            "router failed Ethernet rewrite",
        )?;
    }
    if case == "S02" || case == "S03" {
        let row = report
            .trace
            .iter()
            .find(|r| r.kind == 15)
            .ok_or("ICMP error not generated")?;
        let error = &packet(report, row.version).bytes;
        let original = &packet(report, row.prior).bytes;
        ensure(
            error[20] == if case == "S02" { 3 } else { 11 },
            "incorrect ICMP error type",
        )?;
        ensure(
            error[21] == 0 && error[28..56] == original[..28],
            "ICMP quote/code invalid",
        )?;
        ensure(
            report
                .trace
                .iter()
                .any(|r| r.kind == 16 && r.entity == report.ids["PC1"]),
            "ICMP error did not arrive at source",
        )?;
        ensure(
            !report
                .trace
                .iter()
                .any(|r| r.kind == 13 && r.entity == report.ids["R1"]),
            "dropped request was forwarded",
        )?;
    }
    if case == "S04" {
        let requests: Vec<_> = report
            .trace
            .iter()
            .filter(|r| r.kind == 4 && r.entity == report.ids["R1:eth1"])
            .collect();
        ensure(requests.len() == 3, "ARP retry count invalid")?;
        for (i, row) in requests.iter().enumerate() {
            ensure(
                row.time_ns == requests[0].time_ns + i as u64 * 1_000_000_000,
                "ARP retry timing invalid",
            )?;
        }
        let timeout = report
            .trace
            .iter()
            .find(|r| r.kind == 18)
            .ok_or("ARP timeout missing")?;
        ensure(
            timeout.time_ns == requests[0].time_ns + 3_000_000_000
                && timeout.entity == report.ids["R1"],
            "ARP timeout timing/owner invalid",
        )?;
    }
    if case == "S05" {
        ensure(
            report.summary.reason == 5 && report.packets[0].bytes == [0x45, 0],
            "truncation reason/bytes invalid",
        )?;
    }
    if case == "alias" {
        ensure(
            report
                .trace
                .iter()
                .any(|r| r.kind == 2 && r.reason == 201 && r.entity == report.ids["R1"]),
            "healthy static route not selected",
        )?;
    }
    Ok(())
}
fn report_json(case: &str, report: &Report, full: bool) -> Value {
    let s = report.summary;
    let mut value = json!({
        "scenario":case, "outcome":match s.outcome {1=>"echo_reply",2=>"unreachable",3=>"time_exceeded",4=>"arp_timeout",5=>"invalid_packet",6=>"unsupported",7=>"dropped",_=>"pending"},
        "reason":reason_name(s.reason), "now_ns":s.now_ns, "executed_events":s.executed,
        "packet_versions":s.packet_versions, "transmissions":s.transmissions,
        "pending_events":s.pending_events,"pending_arp":s.pending_arp,
        "trace_rows":s.trace_total,"trace_dropped":s.trace_dropped,"failure_entity":s.failure_entity,
    });
    if full {
        value["entities"] = json!(report.ids);
        value["trace"] = json!(report.trace.iter().map(|r| json!({
            "ordinal":r.ordinal,"event_id":r.event_id,"time_ns":r.time_ns,"cause_id":r.cause_id,
            "entity":r.entity,"packet_id":r.packet_id,"version":r.version,"prior":r.prior,
            "transmission":r.transmission,"kind":kind_name(r.kind),"reason":reason_name(r.reason),
            "detail_a":r.detail_a,"detail_b":r.detail_b
        })).collect::<Vec<_>>());
        value["packets"] = json!(report.packets.iter().zip(&report.fields).map(|(p,fields)| json!({
            "version":p.version_id,"packet_id":p.packet_id,"parent_version":p.parent_version,
            "cause_event":p.cause_event,"layer":if p.layer==1 {"ipv4"} else {"ethernet"},
            "hex":hex(&p.bytes),"fields":fields.iter().map(|f| json!({
                "name":field_name(f.code),"byte_offset":f.byte_offset,"bit_offset_msb":f.bit_offset,
                "bit_width":f.bit_width,"value":if f.bit_width<=64 {Some(f.value)} else {None}
            })).collect::<Vec<_>>()
        })).collect::<Vec<_>>());
        value["transmission_records"] = json!(report.txs.iter().map(|tx| json!({
            "id":tx.id,"version":tx.version,"from":tx.from_port,"to":tx.to_port,"link":tx.link,
            "start_ns":tx.start_ns,"arrival_ns":tx.arrival_ns,"event_id":tx.event_id,"cause_id":tx.cause_id
        })).collect::<Vec<_>>());
    }
    value
}
fn diagnostic_cases() -> Result<Vec<Value>> {
    let mut base = unhex("4500001c00010000400165dd0a0000020a0001020800000012340001")?;
    let sum = checksum(&base[20..]);
    base[22..24].copy_from_slice(&sum.to_be_bytes());
    let mut cases = Vec::<(&str, u32, Vec<u8>, u32, u32)>::new();
    for len in 0..28 {
        cases.push(("truncated_ipv4", 1, base[..len].to_vec(), 5, 5));
    }
    let mut bad = base.clone();
    bad[8] ^= 1;
    cases.push(("bad_ip_checksum", 1, bad, 5, 7));
    let mut bad = base.clone();
    bad[22] ^= 1;
    cases.push(("bad_icmp_checksum", 1, bad, 5, 12));
    for (name, byte, value, reason) in [("fragment", 6, 0x20, 9), ("protocol", 9, 17, 10)] {
        let mut data = base.clone();
        data[byte] = value;
        data[10] = 0;
        data[11] = 0;
        let sum = checksum(&data[..20]);
        data[10..12].copy_from_slice(&sum.to_be_bytes());
        cases.push((name, 1, data, 6, reason));
    }
    let mut options = vec![0; 32];
    options[..20].copy_from_slice(&base[..20]);
    options[0] = 0x46;
    options[3] = 32;
    options[10] = 0;
    options[11] = 0;
    options[24..].copy_from_slice(&base[20..]);
    let sum = checksum(&options[..24]);
    options[10..12].copy_from_slice(&sum.to_be_bytes());
    cases.push(("options", 1, options, 6, 8));
    let mut frame = vec![0; 60];
    frame[..6].copy_from_slice(&[2, 0, 0, 0, 0, 2]);
    frame[6..12].copy_from_slice(&[2, 0, 0, 0, 0, 1]);
    frame[12..14].copy_from_slice(&[0x81, 0]);
    cases.push(("vlan", 2, frame.clone(), 6, 15));
    frame[12..14].copy_from_slice(&[8, 0]);
    frame[6] = 1;
    cases.push(("multicast_source_mac", 2, frame, 5, 16));
    let mut unsupported_icmp = base.clone();
    unsupported_icmp[20] = 99;
    unsupported_icmp[22] = 0;
    unsupported_icmp[23] = 0;
    let sum = checksum(&unsupported_icmp[20..]);
    unsupported_icmp[22..24].copy_from_slice(&sum.to_be_bytes());
    cases.push(("icmp_type", 1, unsupported_icmp, 6, 13));
    let mut jumbo = vec![0; 1501];
    jumbo[..20].copy_from_slice(&base[..20]);
    jumbo[2..4].copy_from_slice(&1501_u16.to_be_bytes());
    jumbo[10] = 0;
    jumbo[11] = 0;
    let sum = checksum(&jumbo[..20]);
    jumbo[10..12].copy_from_slice(&sum.to_be_bytes());
    cases.push(("mtu", 1, jumbo, 6, 14));
    let mut results = Vec::new();
    for (name, layer, bytes, outcome, reason) in cases {
        let (mut net, ids) = setup("S01", 1024)?;
        net.inject(ids["PC1:eth0"], layer, &bytes)?;
        let s = net.summary()?;
        ensure(
            s.outcome == outcome && s.reason == reason,
            &format!(
                "{name}: unexpected outcome/reason {}/{}",
                s.outcome, s.reason
            ),
        )?;
        ensure(
            s.failed == 0 && s.pending_events == 0,
            "invalid input scheduled work",
        )?;
        results.push(
            json!({"case":name,"bytes":bytes.len(),"reason":reason_name(s.reason),"result":"pass"}),
        );
    }
    Ok(results)
}
pub fn demo(case: &str) -> Result<Value> {
    let report = run(case, 1024, false)?;
    validate(&report, case)?;
    Ok(report_json(case, &report, true))
}
pub fn check() -> Result<Value> {
    let mut cases = Vec::new();
    for case in ["S01", "S02", "S03", "S04", "S05", "alias"] {
        let baseline = run(case, 1024, false)?;
        validate(&baseline, case)?;
        ensure(
            run(case, 1024, true)? == baseline,
            "stepwise and batched runs differ",
        )?;
        ensure(run(case, 1024, false)? == baseline, "repeated run differs")?;
        for retention in [0, 1, 16] {
            let mut limited = run(case, retention, false)?;
            let keep = retention.min(baseline.trace.len());
            ensure(
                limited.trace == baseline.trace[baseline.trace.len() - keep..],
                "retention trace suffix differs",
            )?;
            limited.summary.trace_dropped = baseline.summary.trace_dropped;
            ensure(
                limited.summary == baseline.summary
                    && limited.packets == baseline.packets
                    && limited.txs == baseline.txs,
                "trace retention changed execution",
            )?;
        }
        cases.push(report_json(case, &baseline, false));
    }
    Ok(
        json!({"phase":3,"result":"pass","model":"ethernet-arp-ipv4-icmp-v1","scenarios":cases,
        "negative_cases":diagnostic_cases()?,"oracle":"frozen IPv4 header + independent Rust checksum/field/timing/correlation checks",
        "live_device_comparison":"not_run; G5","scenario_runs":36}),
    )
}
#[cfg(test)]
mod tests {
    #[test]
    fn frozen_network_scenarios_and_negative_inputs() {
        assert_eq!(super::check().unwrap()["result"], "pass");
    }
}
