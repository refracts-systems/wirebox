# Time as a first-class dimension: verifiable network history

Recorded 24 October 2026 from a user-provided product thesis. Status: **proposed direction**, grounded in existing virtual time and local checkpoint/replay. Historical queries, symbolic temporal verification, safe multi-device sequencing, and observation-driven projections are not implemented.

## Thesis

The fourth dimension in this vision is **time**: a network is a trajectory of states that engineers can replay, query, and verify. Wyrlex should connect the history of a service, the transitions caused by an operational plan, and the possible consequences of future events through one semantic model.

Snapshot analysis asks whether a modeled configuration satisfies a property. Live inspection asks what is being observed now. The proposed Wyrlex experience adds: **when did the property stop holding, why did it change, and can it remain true while the network is being modified?** These are complementary ways of reasoning. References to snapshot verifiers such as Batfish and live tools such as Winbox are positioning analogies, not evidence that those products or the broader research field lack temporal capabilities. No new competitive or literature survey accompanies this proposal.

The intended product statement is:

> Wyrlex aims to explain and verify whether a network remains correct over time, including while it is being changed, within its declared model and evidence scope.

This extends the existing foundation: `graph + state + behavior + constraints + time`, deterministic virtual time, causal journals, and [Phase 5 checkpoint/replay](../evidence/phase-5-hardening/README.md). Planned Phase 6 observations with provenance would connect that simulated history to recorded real systems. A local simulation checkpoint currently restores only an in-process Wyrlex network; it cannot reconstruct an unrecorded production incident.

## Past: since when, and because of what?

An ISP customer reports losing connectivity around 20:00. The intended workflow starts from recorded observations, changes, external inputs, and a compatible model version. Wyrlex reconstructs the available history, evaluates reachability along it, and identifies the interval or transition in which the property changed. A causal explanation links that change to affected objects, protocol decisions, and packet bytes.

This resembles `git bisect` for a network, but the search algorithm must respect the history. Connectivity can fail, recover, and fail again. Binary search is valid only over an interval with the required monotonic property; arbitrary histories need transition scanning or another appropriate query strategy. Results must distinguish a modeled cause, a recorded correlation, and a hypothesis that still needs a probe.

Exact incident timing requires sufficient observations and ordering evidence. Gaps, journal retention, clock uncertainty across devices, missing external inputs, and unsupported behavior can limit the answer to an interval or make it unknown. Replaying a synthetic scenario is not proof of the actual sequence of production events.

## Transitions: in what order can changes be applied safely?

A plan may have a correct final configuration while temporarily breaking service. For example, moving a service VLAN on a ZTE OLT before the MikroTik side accepts it can disconnect the customer during the transition. The proposed extension to Phase 7 is to verify the path between configurations, rather than evaluating only the final configuration.

Represent a candidate plan as ordered operations and explicit intermediate states. Each operation needs preconditions, modeled effects, completion evidence, and relevant failure or delay possibilities. The verifier checks declared service properties at each modeled transition; a violation produces a counterexample that the concrete simulator can explain down to bytes. The planner can then compare alternative orderings, such as preparing acceptance of a new VLAN before retiring the old path, when device capabilities permit that sequence.

Checking one sequence of stable snapshots is not enough to claim uninterrupted service. Devices may apply changes asynchronously; packets can be in flight; operations may time out or partially succeed. A stronger claim requires coverage of the relevant intermediate states and interleavings, with lab probes validating the modeled assumptions. Wyrlex must report when no safe ordering exists within available capabilities, or when the result is unsupported or inconclusive.

The initial target is a bounded service and operation subset. Multi-device OLT/ONU/MikroTik verification depends on Phase 8 behavior models and interoperability evidence. The relationship to consistent-update and transient-state research is a research question; novelty across multi-vendor PON transitions has not been established.

## Future: what would be affected if this condition occurs?

Periodic observations may reveal a declining optical power measurement for an ONU. Wyrlex could attach that observation history to a scenario such as “this ONU becomes unavailable,” then evaluate which customers, services, VLAN paths, and upstream dependencies would be affected.

The prediction here is **logical impact under an explicit scenario**. It is not an optical-physics model or a verified forecast of when the ONU will fail. Any threshold, extrapolation, or failure assumption must carry its source, observation window, and uncertainty. A useful first feature can answer the conditional impact question without predicting physical failure probability.

## Value to validate

| Audience | Proposed value | Evidence needed |
|---|---|---|
| ISP operators | Explain incidents and find change sequences that preserve the declared service | Replayed lab incidents, causal explanations, and probes during intermediate steps |
| Business and operations | Retain historical change and service evidence for audits and SLA discussions | Reliable provenance, clocks, retention, integrity, and explicit coverage gaps; no automatic contractual conclusion |
| Researchers | Study temporal verification of multi-vendor service transitions, including PON dependencies | A literature review, formal scope, reproducible models, counterexamples, and real-device comparisons |

## Relationship to the roadmap

- Phase 5 provides local deterministic continuation and bounded checkpoints. Durable history, portable replay, and incident reconstruction remain additional work.
- Phase 6 would supply timestamped observations, identities, model/firmware context, and explicit missing-state assumptions.
- Phase 7's [shared Verify IR](verification.md) is the proposed basis for checking properties during supported plans. Transition-system semantics and counterexample replay need their own contract and evidence before a temporal guarantee is claimed.
- Phase 8 would validate selected ZTE/Huawei/VSOL/MikroTik service transitions and seed the [device behavior corpus](behavior-corpus.md).
- Later phases can evaluate historical queries and conditional impact analysis based on pilot needs.

These additions do not pass or silently expand current gates. The immediate priority remains model fidelity and comparison with the lab. A temporal verifier can be confidently wrong at every step if its underlying device model is wrong.
