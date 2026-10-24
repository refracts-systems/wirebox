use serde_json::{Value, json};
use std::{collections::BTreeMap, error::Error};
use wyrlex_kernel::simulation::{Action, DeviceRole, Entity, Snapshot, Trace, World};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const TOPOLOGY: &str = include_str!("../../../fixtures/scenarios/icmp-static-v1.json");

#[derive(Debug, PartialEq, Eq)]
struct Outcome {
    state: Vec<Entity>,
    snapshot: Snapshot,
    trace: Vec<Trace>,
    names: BTreeMap<String, u64>,
}

fn ensure(condition: bool, message: &'static str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}

fn run(seed: u64, retention: usize) -> Result<Outcome> {
    let fixture: Value = serde_json::from_str(TOPOLOGY)?;
    let mut world = World::new(seed, retention)?;
    let mut names = BTreeMap::new();
    // Phase 2 loads identity/port/link relations only, not IPv4 or protocol behavior.
    for node in fixture["nodes"].as_array().ok_or("nodes missing")? {
        let name = node["id"].as_str().ok_or("node ID missing")?;
        let role = match node["kind"].as_str() {
            Some("host") => DeviceRole::Host,
            Some("switch") => DeviceRole::Switch,
            Some("router") => DeviceRole::Router,
            _ => return Err("unsupported device role".into()),
        };
        let id = world.add_device(role)?;
        names.insert(name.to_owned(), id);
        for port in node["ports"].as_array().ok_or("ports missing")? {
            let port_name = port["id"].as_str().ok_or("port ID missing")?;
            names.insert(format!("{name}:{port_name}"), world.add_port(id)?);
        }
    }
    for edge in fixture["edges"].as_array().ok_or("edges missing")? {
        let name = edge["id"].as_str().ok_or("link ID missing")?;
        let endpoints = edge["endpoints"].as_array().ok_or("endpoints missing")?;
        let a = *names
            .get(endpoints[0].as_str().ok_or("endpoint missing")?)
            .ok_or("unknown port")?;
        let b = *names
            .get(endpoints[1].as_str().ok_or("endpoint missing")?)
            .ok_or("unknown port")?;
        names.insert(name.to_owned(), world.connect(a, b)?);
    }
    ensure(
        world.entities()?.len() == 13,
        "graph should have 13 entities",
    )?;
    world.schedule(names["PC1"], 10, 0, Action::AddCounter, 1)?;
    world.schedule(names["R1"], 5, 0, Action::AddCounter, 10)?;
    world.schedule(names["R1"], 10, 0, Action::AddCounter, 2)?;
    let periodic = world.schedule(names["SW1"], 10, 5, Action::RandomAddCounter, 100)?;
    let cancelled = world.schedule(names["PC2"], 7, 0, Action::AddCounter, 99)?;
    world.cancel(cancelled)?;
    let temporary = world.add_device(DeviceRole::Host)?;
    world.schedule(temporary, 1, 0, Action::AddCounter, 99)?;
    world.remove(temporary)?;
    let first = world.step()?;
    ensure(
        first.now_ns == 5 && first.processed == 1,
        "step order failed",
    )?;
    // Reading and a one-row visible projection are consumers, not the execution queue.
    let visible = world.trace(0, 1)?;
    ensure(visible.len() <= 1, "visible projection failed")?;
    let bounded = world.run_until(100, 4)?;
    ensure(
        bounded.reason == 2 && bounded.processed == 4 && bounded.now_ns == 15,
        "periodic run failed to yield at budget",
    )?;
    world.cancel(periodic)?;
    ensure(world.run_until(100, 0)?.reason == 1, "empty horizon failed")?;
    ensure(world.step()?.now_ns == 100, "empty step moved clock")?;
    let state = world.entities()?;
    for (name, expected) in [("PC1", 1), ("R1", 12), ("PC2", 0)] {
        ensure(
            state.iter().find(|e| e.id == names[name]).unwrap().counter == expected,
            "counter expectation failed",
        )?;
    }
    let snapshot = world.snapshot()?;
    ensure(
        snapshot.executed == 5 && snapshot.pending_count == 0 && snapshot.timer_count == 0,
        "final lifecycle invariant failed",
    )?;
    let trace = world.trace(0, 256)?;
    world.close()?;
    Ok(Outcome {
        state,
        snapshot,
        trace,
        names,
    })
}

fn trace_json(trace: &[Trace]) -> Value {
    json!(
        trace
            .iter()
            .map(|row| json!({
                "ordinal": row.ordinal, "event_id": row.event.id,
                "time_ns": row.event.time_ns, "target_id": row.event.target_id,
                "timer_id": row.event.timer_id, "cause_id": row.event.cause_id,
                "action": row.event.action, "amount": row.event.amount,
                "before": row.before, "after": row.after,
            }))
            .collect::<Vec<_>>()
    )
}

pub fn demo() -> Result<Value> {
    let result = run(20260117, 256)?;
    Ok(json!({
        "phase": 2, "mode": "scheduler_demo", "result": "pass",
        "model": "counter-actions-v1", "packet_simulation": "separate --packet-demo / --check-network",
        "fixture": "icmp-static-v1 (graph only)", "seed": result.snapshot.seed,
        "prng": "SplitMix64-v1", "rng_state": result.snapshot.rng_state,
        "now_ns": result.snapshot.now_ns, "executed": result.snapshot.executed,
        "pending": result.snapshot.pending_count, "timers": result.snapshot.timer_count,
        "entities": result.state.len(), "names": result.names,
        "counters": result.state.iter().filter(|e| e.kind == 1)
            .map(|e| json!({"entity_id": e.id, "value": e.counter})).collect::<Vec<_>>(),
        "trace": trace_json(&result.trace),
        "budget_stop_ns": 15, "empty_horizon_ns": 100,
        "cancelled_timer_did_not_fire": true, "deleted_owner_did_not_fire": true
    }))
}

pub fn check() -> Result<Value> {
    let seeds = [0, 1, 42, 20260117, u64::MAX];
    let mut runs = 0;
    for seed in seeds {
        let baseline = run(seed, 256)?;
        ensure(
            baseline
                .trace
                .iter()
                .map(|t| t.event.id)
                .collect::<Vec<_>>()
                == [2, 1, 3, 4, 7],
            "independent expected event order mismatch",
        )?;
        ensure(
            baseline
                .trace
                .iter()
                .map(|t| t.event.time_ns)
                .collect::<Vec<_>>()
                == [5, 10, 10, 10, 15],
            "expected event times mismatch",
        )?;
        ensure(baseline.trace[4].event.cause_id == 4, "causality mismatch")?;
        for _ in 0..3 {
            ensure(run(seed, 256)? == baseline, "same-seed rerun differs")?;
            runs += 1;
        }
        for retention in [0, 1, 3, 256] {
            let mut limited = run(seed, retention)?;
            let keep = retention.min(5);
            ensure(
                limited.trace == baseline.trace[5 - keep..],
                "retained trace suffix differs",
            )?;
            ensure(
                limited.snapshot.dropped == (5 - keep) as u64,
                "dropped count differs",
            )?;
            limited.snapshot.dropped = baseline.snapshot.dropped;
            limited.snapshot.journal_count = baseline.snapshot.journal_count;
            ensure(
                limited.state == baseline.state && limited.snapshot == baseline.snapshot,
                "retention changed simulation state",
            )?;
        }
    }
    Ok(json!({
        "phase": 2, "result": "pass", "seeds": seeds,
        "repeat_runs": runs, "baseline_runs": 5, "retention_runs": 20,
        "compared": ["execution_order", "trace", "graph_state", "clock", "timer_lifecycle", "rng_state", "ids"],
        "retention_capacities": [0, 1, 3, 256],
        "expected_event_ids": [2, 1, 3, 4, 7],
        "scope": "counter model fresh-run reproducibility; checkpoint/restore not implemented; packets checked separately"
    }))
}

#[cfg(test)]
mod tests {
    #[test]
    fn acceptance_matrix_runs_through_real_zig_ffi() {
        assert_eq!(super::check().unwrap()["result"], "pass");
    }
}
