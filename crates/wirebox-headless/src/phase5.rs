use serde_json::{Value, json};
use std::{collections::BTreeMap, error::Error};
use wyrlex_kernel::network::{Field, Network, Noise, Packet, Summary, Trace, Transmission};
use wyrlex_kernel::simulation::DeviceRole;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Observable {
    summary: Summary,
    packets: Vec<Packet>,
    fields: Vec<Vec<Field>>,
    trace: Vec<Trace>,
    transmissions: Vec<Transmission>,
    noise: Noise,
    rng_state: u64,
}

impl Observable {
    fn difference(&self, other: &Self) -> Option<&'static str> {
        if self.summary != other.summary {
            Some("summary")
        } else if self.packets != other.packets {
            Some("packet bytes/ancestry")
        } else if self.fields != other.fields {
            Some("field mapping")
        } else if self.transmissions != other.transmissions {
            Some("transmissions")
        } else if self.noise != other.noise {
            Some("noise counters")
        } else if self.rng_state != other.rng_state {
            Some("PRNG state")
        } else if self.trace != other.trace {
            Some("retained trace")
        } else {
            None
        }
    }
}

fn host_pair(seed: u64) -> Result<(Network, u64), Box<dyn Error>> {
    let mut network = Network::new(seed, 1024)?;
    let left = network.device(DeviceRole::Host)?;
    let right = network.device(DeviceRole::Host)?;
    let left_port = network.port(left)?;
    let right_port = network.port(right)?;
    network.configure_port(left_port, 0x020000000001, 0x0a000001, 24, true)?;
    network.configure_port(right_port, 0x020000000002, 0x0a000002, 24, true)?;
    network.link(left_port, right_port, 100_000_000, 1_000)?;
    Ok((network, left_port))
}

fn drain(network: &mut Network) -> Result<(), Box<dyn Error>> {
    for _ in 0..1024 {
        let progress = network.run_until(5_000_000_000, 512)?;
        if progress.reason == 1 {
            return Ok(());
        }
    }
    Err("Phase 5 exceeded 1024 bounded run calls".into())
}

fn observable(network: &Network) -> Result<Observable, Box<dyn Error>> {
    let summary = network.summary()?;
    if summary.failed != 0 {
        return Err("failed network cannot be accepted as replay evidence".into());
    }
    let packets = (1..=summary.packet_versions)
        .map(|id| network.packet(id))
        .collect::<Result<Vec<_>, _>>()?;
    let mut trace = Vec::new();
    loop {
        let after = trace.last().map_or(0, |row: &Trace| row.ordinal);
        let batch = network.trace(after, 256)?;
        if batch.is_empty() {
            break;
        }
        let first = after.max(summary.trace_dropped) + 1;
        if batch
            .iter()
            .enumerate()
            .any(|(i, row)| row.ordinal != first + i as u64)
        {
            return Err("retained trace is not a contiguous suffix".into());
        }
        trace.extend(batch);
    }
    if trace.len() as u64 != summary.trace_total - summary.trace_dropped {
        return Err("retained trace count disagrees with summary".into());
    }
    let fields = (1..=summary.packet_versions)
        .map(|id| match network.fields(id) {
            Ok(fields) => Ok(fields),
            Err(wyrlex_kernel::Error::InvalidPacket | wyrlex_kernel::Error::Unsupported) => {
                Ok(Vec::new())
            }
            Err(e) => Err(e),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let transmissions = (1..=summary.transmissions)
        .map(|id| network.transmission(id))
        .collect::<Result<Vec<_>, _>>()?;
    let rng_state = network.checkpoint()?.info()?.rng_state;
    Ok(Observable {
        summary,
        packets,
        fields,
        trace,
        transmissions,
        noise: network.noise()?,
        rng_state,
    })
}

fn start(
    case: &str,
    seed: u64,
    retention: usize,
    loss: u32,
    jitter: u64,
) -> Result<Network, Box<dyn Error>> {
    let (mut network, ids) = crate::packet_demo::setup_seeded(case, retention, seed)?;
    network.set_noise(loss, jitter)?;
    if case == "S05" {
        network.inject(ids["PC1:eth0"], 1, &[0x45, 0])?;
    } else {
        let destination = if case == "alias" || case == "S02" {
            0xc6336402
        } else {
            0x0a000102
        };
        network.ping(
            ids["PC1:eth0"],
            destination,
            if case == "S03" { 1 } else { 64 },
        )?;
    }
    Ok(network)
}

fn validate_link_timing(run: &Observable, jitter: u64) -> Result<(), Box<dyn Error>> {
    let mut free = BTreeMap::<u64, u64>::new();
    for tx in &run.transmissions {
        let packet = run
            .packets
            .get(tx.version as usize - 1)
            .ok_or("missing Tx packet version")?;
        let serialization = (packet.bytes.len() as u64 * 8 * 1_000_000_000).div_ceil(100_000_000);
        let base = tx.start_ns + serialization + 1000;
        if tx.arrival_ns < base
            || tx.arrival_ns - base > jitter
            || tx.start_ns < *free.get(&tx.from_port).unwrap_or(&0)
        {
            return Err(format!(
                "Tx {} violates serialization/propagation/jitter bounds",
                tx.id
            )
            .into());
        }
        free.insert(tx.from_port, tx.start_ns + serialization);
    }
    Ok(())
}

fn replay_matrix() -> Result<Value, Box<dyn Error>> {
    let mut comparisons = 0;
    let mut incomplete_histories = 0;
    let mut unresolved_runs = 0;
    for case in ["S01", "S02", "S03", "S04", "S05", "alias"] {
        for seed in [0, 1, 42, u64::MAX] {
            for retention in [0, 1, 16, 1024] {
                for (loss, jitter) in [(0, 0), (0, 10_000), (250_000, 10_000), (1_000_000, 0)] {
                    let mut baseline = start(case, seed, retention, loss, jitter)?;
                    drain(&mut baseline)?;
                    let expected = observable(&baseline)?;
                    validate_link_timing(&expected, jitter)?;
                    if expected.summary.outcome == 0 {
                        unresolved_runs += 1;
                    }
                    let clean_outcome = match case {
                        "S01" | "alias" => 1,
                        "S02" => 2,
                        "S03" => 3,
                        "S04" => 4,
                        _ => 5,
                    };
                    if loss == 0 && expected.summary.outcome != clean_outcome {
                        return Err(format!("{case}: baseline outcome differs from fixture").into());
                    }
                    if expected.summary.trace_dropped > 0 {
                        incomplete_histories += 1;
                    }
                    drop(baseline);
                    for cut in [0, 1, 2, 6, 12] {
                        let mut home = start(case, seed, retention, loss, jitter)?;
                        home.run_until(5_000_000_000, cut)?;
                        let checkpoint = home.checkpoint()?;
                        let info = checkpoint.info()?;
                        let at_cut = observable(&home)?;
                        let mut branch = checkpoint.fork()?;
                        drain(&mut branch)?;
                        if observable(&home)? != at_cut || checkpoint.info()? != info {
                            return Err("fork mutated home or immutable checkpoint".into());
                        }
                        drain(&mut home)?;
                        for (label, actual) in
                            [("home", observable(&home)?), ("fork", observable(&branch)?)]
                        {
                            if let Some(diff) = expected.difference(&actual) {
                                return Err(format!("{case} seed={seed} retention={retention} loss={loss} jitter={jitter} cut={cut}: {label} differs at {diff}").into());
                            }
                        }
                        home.restore(&checkpoint)?;
                        if observable(&home)? != at_cut {
                            return Err("restore did not recover checkpoint observables".into());
                        }
                        drain(&mut home)?;
                        if let Some(diff) = expected.difference(&observable(&home)?) {
                            return Err(format!(
                                "{case} seed={seed} cut={cut}: restore differs at {diff}"
                            )
                            .into());
                        }
                        comparisons += 1;
                    }
                }
            }
        }
    }
    let mut runs = Vec::new();
    for seed in [0, 1] {
        let mut network = start("S01", seed, 1024, 0, 10_000)?;
        drain(&mut network)?;
        runs.push(observable(&network)?);
    }
    if runs[0].transmissions == runs[1].transmissions {
        return Err("different jitter seeds produced identical fixture timings".into());
    }
    Ok(
        json!({"checkpoint_cases": comparisons, "continuations_compared": comparisons * 3,
        "scenarios": 6, "seeds": 4, "retention_capacities": [0,1,16,1024],
        "impairments": 4, "cut_budgets": [0,1,2,6,12],
        "incomplete_baseline_histories": incomplete_histories,
        "unresolved_baseline_outcomes": unresolved_runs,
        "different_seed_timings": "different", "tolerance": "exact"}),
    )
}

pub fn check() -> Result<Value, Box<dyn Error>> {
    let (mut home, source) = host_pair(0x51)?;
    home.ping(source, 0x0a000002, 64)?;
    let checkpoint = home.checkpoint()?;
    let checkpoint_info = checkpoint.info()?;
    let pending_at_checkpoint = home.summary()?.pending_events;
    let mut branch = checkpoint.fork()?;
    drain(&mut home)?;
    drain(&mut branch)?;
    let expected = observable(&home)?;
    let branch_matches = expected == observable(&branch)?;
    home.restore(&checkpoint)?;
    let restore_pending = home.summary()?.pending_events;
    drain(&mut home)?;
    let restore_matches = expected == observable(&home)?;
    drop((home, branch, checkpoint));

    let (mut noisy, source) = host_pair(0x1234)?;
    noisy.set_noise(0, 10_000)?;
    noisy.ping(source, 0x0a000002, 64)?;
    let noisy_checkpoint = noisy.checkpoint()?;
    let mut noisy_branch = noisy_checkpoint.fork()?;
    drain(&mut noisy)?;
    drain(&mut noisy_branch)?;
    let jitter_matches = observable(&noisy)? == observable(&noisy_branch)?
        && noisy.noise()? == noisy_branch.noise()?;
    let jitter_noise = noisy.noise()?;
    drop((noisy, noisy_branch, noisy_checkpoint));

    let (mut lossy, source) = host_pair(0x1234)?;
    lossy.set_noise(1_000_000, 0)?;
    lossy.ping(source, 0x0a000002, 64)?;
    drain(&mut lossy)?;
    let loss_summary = lossy.summary()?;
    let loss_noise = lossy.noise()?;
    drop(lossy);
    let matrix = replay_matrix()?;

    if pending_at_checkpoint == 0
        || checkpoint_info.model_version != 1
        || checkpoint_info.input_kind != 1
        || checkpoint_info.pending_events != pending_at_checkpoint
        || restore_pending != pending_at_checkpoint
        || !branch_matches
        || !restore_matches
        || !jitter_matches
        || jitter_noise.draws == 0
        || loss_noise.dropped == 0
        || loss_summary.outcome != 4
        || loss_summary.pending_events != 0
    {
        return Err("Phase 5 replay or impairment comparator failed".into());
    }
    Ok(json!({
        "phase": 5,
        "replay_matrix": matrix,
        "g5_status": "incomplete; Linux capture and comparison not run",
        "result": "pass",
        "checkpoint": {
            "model_version": checkpoint_info.model_version,
            "input_kind": checkpoint_info.input_kind,
            "pending_events": pending_at_checkpoint,
            "fork_matches": branch_matches,
            "restore_matches": restore_matches
        },
        "jitter": {
            "replay_matches": jitter_matches,
            "draws": jitter_noise.draws,
            "delayed": jitter_noise.delayed
        },
        "loss": {
            "loss_per_million": 1_000_000,
            "dropped": loss_noise.dropped,
            "outcome": loss_summary.outcome,
            "reason": loss_summary.reason
        },
        "lab_comparison": "not_run",
        "scope": "value-owned checkpoint, active ARP queue, deterministic PRNG impairment"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase5_headless_check_passes() {
        assert_eq!(super::check().unwrap()["result"], "pass");
    }

    #[test]
    fn comparator_detects_mutations_in_every_observable_category() {
        let mut expected = Observable::default();
        expected.packets.push(Packet {
            version_id: 1,
            packet_id: 1,
            parent_version: 0,
            cause_event: 0,
            layer: 1,
            bytes: vec![0x45, 0],
        });
        expected.fields.push(vec![Field::default()]);
        expected.transmissions.push(Transmission::default());
        expected.trace.push(Trace::default());
        assert_eq!(expected.difference(&expected), None);
        for category in 0..7 {
            let mut changed = expected.clone();
            let label = match category {
                0 => {
                    changed.summary.executed += 1;
                    "summary"
                }
                1 => {
                    changed.packets[0].bytes[0] ^= 1;
                    "packet bytes/ancestry"
                }
                2 => {
                    changed.fields[0][0].byte_offset += 1;
                    "field mapping"
                }
                3 => {
                    changed.transmissions[0].arrival_ns += 1;
                    "transmissions"
                }
                4 => {
                    changed.noise.draws += 1;
                    "noise counters"
                }
                5 => {
                    changed.rng_state ^= 1;
                    "PRNG state"
                }
                _ => {
                    changed.trace[0].cause_id += 1;
                    "retained trace"
                }
            };
            assert_eq!(expected.difference(&changed), Some(label));
        }
    }
}
