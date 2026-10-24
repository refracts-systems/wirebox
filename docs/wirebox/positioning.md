# Feasibility of benchmarking ideas and projects

Wyrlex makes sense as a series of capabilities built in stages. The greatest difficulty is the breadth of the model and the equivalence between the simulation and the real system. The mere existence of a graph, UI, or intent compiler does not prove accuracy or operational benefit.

## Projects discussed

The following functional summary was checked from official sources on October 19, 2026. The relevance column is an interpretation for Wyrlex, not a claim that the products are identical or lack certain features.

| Project | Relevant focus | Lessons for Wyrlex |
|---|---|---|
| [Batfish](https://batfish.org/) | Network configuration analysis | Separate reachability/policy validation from time simulation of each packet |
| [Forward Networks](https://www.forwardnetworks.com/) | Network digital twin models for change inspection and validation | There is a real need for model-based querying of the behavior and impact of changes |
| [NetBox Labs](https://netboxlabs.com/) | Infrastructure data/model, intended and operational state, and drift | Object identity and state provenance need a strong foundation |
| [Apstra](https://www.juniper.net/us/en/products/network-automation/apstra-data-center-director.html) | Intent-based automation and data center network validation | A defined domain helps maintain intent and backend contracts |
| [SmartOLT](https://www.smartolt.com/) | OLT/ONU management, provisioning, monitoring, and operations | ISP access domains have concrete workflows that can be selected later |
| [OpenConfig](https://www.openconfig.net/) | Vendor-neutral data model, telemetry, and management protocols | Existing models can be mapped; schema alone is not yet a behavior simulator |
| [containerlab](https://containerlab.dev/) | Container-based network lab orchestration and virtual connections | A real implementation lab can serve as a comparison for the Wyrlex | model

Vendor descriptions describe product positions, not independent benchmark results. There has been no direct evaluation of Wyrlex against these products or decision on integration with any of them.

## Differentiation space that you want to prove

Wyrlex hypotheses are experiences that connect infrastructure objects, intents, experiments, causal traces, and byte inspections. Users can move from asking “why did it fail?” to state, protocol decisions, and relevant bytes without losing their provenance relationship.

This hypothesis needs to be proven through a small, complete workflow. Initial example: ping failed because route was lost; the user looks at the reason for the drop, changes the route on the simulation branch, then compares the results and byte changes.

For the next operational use case, the initial vision proposes customer reachability across ONU → OLT → VLAN → router/NAT → uplink. That scenario requires a broader model and adapters that don't yet exist.

## Limits of ambition

The initial priority is a model that can be explained and tested on limited scenarios. Multi-vendor support, AI, full time travel, and production execution become expanded once the foundation is proven. There are no pricing decisions, business models, scale targets, or promises of vendor-wide support in these conversations.

[Roadmap](roadmap.md) translates the direction into stages with examineable results.
