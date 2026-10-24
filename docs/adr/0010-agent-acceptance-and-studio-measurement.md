# ADR-0010 — Agent operator acceptance and Studio measurement

24 October 2026. Status: Accepted for the test method; gate status must follow completed evidence runs.

The user requested that the agent perform all Phase 4 testing. The agent inspected U1–U5 and the editor through native computer use. Human independence and understanding remain unknown and are not required from the user to complete technical acceptance. Human usability study remains a beta evaluation candidate. This decision changes the operator acceptance method, not the observable result or performance targets.

The performance method is frozen for the final repeat: four nodes, 10,000 retained rows projected from 160 actually executed S01 kernel runs, rebased event/packet/transmission IDs and time, and copied bytes/fields unchanged. This is projected multi-run history, not a claim that one engine supports an unbounded workload. The oldest 80 rows are evicted with an explicit counter; all remaining version references stay available and are tested.

UI acceptance uses 50-row pagination after the 200-row baseline missed its target. The script runs 1,800 pulses with a one-second timer, covering kernel/FFI events, snapshots, pause, cross-page selection, and save/load every ten pulses. Time is measured from handler/dispatch entry to the GPUI callback after frame rendering. OS delay before the handler and compositor/display presentation are excluded; the result is called handler-to-render. Native click/Space samples are reported separately from scripted samples; the small count is not treated as representative OS-input p95.

The target remains p95 <100 ms for each selection/pause operation, at least 100 samples per type, without dropping outliers. Frames delayed by another application or compilation remain in the data. GPUI may skip a draw when another application covers the window; actual sample counts must be reported. The session must run at least 30 minutes without crash/hang and with working set ≤1 GiB. Memory is sampled every second; process peak is recorded separately when available.

Acceptance mode is active only with `WYRLEX_ACCEPTANCE`; the UI has a distinct title, history is read-only, and history is not exported as a normal runtime session. The driver owns Network only on the GPUI thread. No new dependency or ABI change is allowed.

Technical G4 may be marked passed only when the walkthrough and final run meet these criteria. It is not human-user verification, active checkpoint restore, vendor conformance, or remote CI evidence.
