use serde_json::{Value, json};
use std::{collections::HashSet, error::Error};

const SCENARIO: &str = include_str!("../../../fixtures/scenarios/icmp-static-v1.json");
const GOLDEN: &str = include_str!("../../../fixtures/packets/ipv4-forwarding-v1.json");
const INVENTORY: &str = include_str!("../../../fixtures/inventory/pilot-v1.json");

fn require(condition: bool, message: &str) -> Result<(), Box<dyn Error>> {
    if condition {
        Ok(())
    } else {
        Err(message.to_string().into())
    }
}

fn bytes(value: &Value, name: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let text = value[name].as_str().ok_or("hex string missing")?;
    require(text.len() % 2 == 0 && text.is_ascii(), "invalid hex width")?;
    (0..text.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&text[offset..offset + 2], 16).map_err(Into::into))
        .collect()
}

fn checksum_valid(header: &[u8]) -> bool {
    let mut sum: u32 = header
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u16::from_be_bytes([pair[0], pair[1]]) as u32)
        .sum();
    while sum > 0xffff {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    sum == 0xffff
}

fn validate_golden(golden: &Value) -> Result<(), Box<dyn Error>> {
    let before = bytes(golden, "before_hex")?;
    let after = bytes(golden, "after_hex")?;
    require(
        before.len() == 20 && after.len() == 20,
        "IPv4 fixture must contain 20-byte headers",
    )?;
    require(
        before[0] == 0x45 && after[0] == 0x45,
        "IPv4 version/IHL mismatch",
    )?;
    require(
        checksum_valid(&before) && checksum_valid(&after),
        "IPv4 fixture checksum mismatch",
    )?;
    require(before[8] == 64 && after[8] == 63, "forwarding TTL mismatch")?;
    require(
        before
            .iter()
            .zip(&after)
            .enumerate()
            .all(|(offset, (a, b))| a == b || [8, 10, 11].contains(&offset)),
        "unexpected header mutation",
    )?;
    Ok(())
}

pub fn check() -> Result<Value, Box<dyn Error>> {
    let scenario: Value = serde_json::from_str(SCENARIO)?;
    let golden: Value = serde_json::from_str(GOLDEN)?;
    let inventory: Value = serde_json::from_str(INVENTORY)?;
    for value in [&scenario, &golden, &inventory] {
        require(value["schema_version"] == 1, "unsupported fixture schema")?;
    }
    let nodes = scenario["nodes"].as_array().ok_or("nodes missing")?;
    require(nodes.len() == 4, "canonical scenario requires four nodes")?;
    let mut ports = HashSet::new();
    let mut names = HashSet::new();
    for node in nodes {
        let name = node["id"].as_str().ok_or("node ID missing")?;
        require(names.insert(name), "duplicate node ID")?;
        for port in node["ports"].as_array().ok_or("ports missing")? {
            let port_id = port["id"].as_str().ok_or("port ID missing")?;
            require(ports.insert(format!("{name}:{port_id}")), "duplicate port")?;
        }
    }
    let mut linked = HashSet::new();
    for edge in scenario["edges"].as_array().ok_or("edges missing")? {
        let endpoints = edge["endpoints"].as_array().ok_or("endpoints missing")?;
        require(endpoints.len() == 2, "point-to-point link needs two ports")?;
        for endpoint in endpoints {
            let name = endpoint.as_str().ok_or("endpoint string missing")?;
            require(ports.contains(name), "unknown link endpoint")?;
            require(linked.insert(name), "port connected more than once")?;
        }
    }
    let cases = scenario["scenarios"]
        .as_array()
        .ok_or("scenarios missing")?;
    let ids: HashSet<_> = cases.iter().filter_map(|v| v["id"].as_str()).collect();
    require(
        ids.len() == 5
            && ["S01", "S02", "S03", "S04", "S05"]
                .iter()
                .all(|id| ids.contains(id)),
        "canonical scenario IDs mismatch",
    )?;
    validate_golden(&golden)?;
    Ok(json!({
        "mode": "fixture_validation", "network_simulation_run": false,
        "schema_version": 1, "nodes": nodes.len(), "scenarios": cases.len(),
        "ipv4_checksums": "valid", "inventory": "user_reported_with_unknowns", "result": "pass"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_fixture_set_is_consistent() {
        check().unwrap();
    }

    #[test]
    fn checksum_or_mutation_regression_in_golden_is_detected() {
        let mut golden: Value = serde_json::from_str(GOLDEN).unwrap();
        golden["after_hex"] = json!("4500001c000100003f0165dd0a0000020a000102");
        assert!(validate_golden(&golden).is_err());
    }
}
