# ADR-0009 — Configuration draft, session v2, and trace retention

Date: 24 October 2026. Status: Accepted for implementation; G4 still requires acceptance evidence.

Config supports an explicit static-route table for fixture L3 ports, an administrative-down port list, and per-link timing. Topology remains four nodes. Routes use `device prefix port gateway`, separated by `;`; timing uses `L1 bitrate_bps delay_ns`. The text form is an initial editor validated into typed records before FFI. It is not a vendor CLI.

Fields are stored in a draft. Apply builds a new Network and replaces the owner only after construction succeeds. Validation failure preserves the active result and draft. Discard drops the draft. Run/Save use active configuration; an unapplied draft is not saved. A successful reset/preset/load drops the draft.

Port down means the port's administrative-enabled flag is false; it does not remove a cable. Apply creates a new simulation with empty queue/cache and does not change pending packets in the old branch. There is no physical-carrier simulation claim.

The writer uses session schema 2; the reader accepts v1 and v2. v1 config is converted to new defaults: fixture routes, no additional ports down, and no timing overrides. Rerun report validation remains mandatory, including the baseline. A session still reruns from the beginning rather than restoring active checkpoint state.

Network kernel retention is at most 10,000 rows. Ring reads start directly after the cursor so a batch does not rescan an old prefix. Packet/tx capacity remains bounded and never overwrites. UI shows at most 200 rows per page; every retained row remains selectable through pagination. Retention does not change the scheduler.

Synthetic journal-write tests prove wrap and version references, not a 10,000-event network workload. Input-to-visible p95 and a 30-minute application soak still require separate measurement. ABI layout and ownership are unchanged.
