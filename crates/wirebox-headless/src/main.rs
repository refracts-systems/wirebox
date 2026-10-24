use std::{error::Error, process::ExitCode};
use wyrlex_kernel::{ABI_VERSION, BATCH_LIMIT, PROBE_BYTES, Session, ZIG_VERSION};
mod fixtures;
mod packet_demo;
mod runtime_demo;

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args == ["--check-network"] {
        println!("{}", serde_json::to_string_pretty(&packet_demo::check()?)?);
        return Ok(());
    }
    if args.first().is_some_and(|arg| arg == "--packet-demo") && args.len() <= 2 {
        println!(
            "{}",
            serde_json::to_string_pretty(&packet_demo::demo(
                args.get(1).map_or("S01", String::as_str)
            )?)?
        );
        return Ok(());
    }
    if args == ["--scheduler-demo"] || args == ["--check-runtime"] {
        let result = if args == ["--scheduler-demo"] {
            runtime_demo::demo()?
        } else {
            runtime_demo::check()?
        };
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }
    if args == ["--check-fixtures"] {
        println!("{}", fixtures::check()?);
        return Ok(());
    }
    if !args.is_empty() && args != ["--probe"] {
        return Err(
            "usage: wyrlex-headless [--probe | --check-fixtures | --scheduler-demo | --check-runtime | --packet-demo [S01..S05|alias] | --check-network]"
                .into(),
        );
    }
    let mut session = Session::new()?;
    session.submit_probe(PROBE_BYTES)?;
    let events = session.read_probe(BATCH_LIMIT)?;
    if events.len() != 1 || events[0].bytes != PROBE_BYTES {
        return Err("Zig/Rust probe payload mismatch".into());
    }
    session.close()?;
    println!(
        "{}",
        serde_json::json!({
            "phase": 1, "mode": "abi_probe", "simulation": "not_exercised_by_probe",
            "abi_version": ABI_VERSION, "zig_version": ZIG_VERSION,
            "event_count": events.len(), "event_id": events[0].id,
            "hex": events[0].bytes.iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
            "ownership": "copied", "session": "destroyed", "result": "pass"
        })
    );
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
