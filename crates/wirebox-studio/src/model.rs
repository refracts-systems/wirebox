//! UI-independent controller. No engine pointers or native handles enter saved files.
use serde_json::{Value, json};
use std::{collections::BTreeMap, net::Ipv4Addr};
use wyrlex_kernel::{
    Error,
    network::{Field, Network, Packet, Summary, Trace, Transmission, field_name},
    simulation::DeviceRole,
};

type RouteEntry = (String, String, u32, u32, u32);
pub type Result<T> = std::result::Result<T, String>;
pub const MODEL: &str = "ethernet-arp-ipv4-icmp-v1";
pub const HORIZON: u64 = 5_000_000_000;
pub const FILE_LIMIT: usize = 2 * 1024 * 1024;
const FIXTURE: &str = include_str!("../../../fixtures/scenarios/icmp-static-v1.json");
const PORTS: [&str; 4] = ["PC1:eth0", "R1:eth0", "R1:eth1", "PC2:eth0"];

fn require(ok: bool, message: &str) -> Result<()> {
    if ok { Ok(()) } else { Err(message.into()) }
}
fn ip(text: &str) -> Result<u32> {
    text.parse::<Ipv4Addr>()
        .map(u32::from)
        .map_err(|e| e.to_string())
}
fn cidr(text: &str) -> Result<(u32, u32)> {
    let (address, prefix) = text.split_once('/').ok_or("Gunakan IPv4/prefix")?;
    let prefix = prefix.parse::<u32>().map_err(|e| e.to_string())?;
    require(prefix <= 32, "Prefix harus 0..32")?;
    Ok((ip(address)?, prefix))
}
fn number(value: &Value, key: &str) -> Result<u64> {
    value[key]
        .as_u64()
        .ok_or_else(|| format!("{key} harus integer unsigned"))
}
fn text(value: &Value, key: &str) -> Result<String> {
    value[key]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("{key} harus string"))
}
fn err(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub routes: Option<String>,
    pub disabled_ports: String,
    pub links: String,
    pub case: String,
    pub addresses: BTreeMap<String, String>,
    pub route_via: Option<String>,
    pub pc2_enabled: bool,
    pub destination: String,
    pub ttl: u32,
    pub bitrate: u64,
    pub propagation: u64,
}
impl Config {
    pub fn preset(case: &str) -> Result<Self> {
        require(
            ["S01", "S02", "S03", "S04", "S05"].contains(&case),
            "Skenario tidak didukung",
        )?;
        Ok(Self {
            routes: None,
            disabled_ports: String::new(),
            links: String::new(),
            case: case.into(),
            addresses: PORTS
                .into_iter()
                .zip(["10.0.0.2/24", "10.0.0.1/24", "10.0.1.1/24", "10.0.1.2/24"])
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
            route_via: (case != "S02").then(|| "10.0.1.2".into()),
            pc2_enabled: case != "S04",
            destination: if case == "S02" {
                "198.51.100.2"
            } else {
                "10.0.1.2"
            }
            .into(),
            ttl: if case == "S03" { 1 } else { 64 },
            bitrate: 100_000_000,
            propagation: 1000,
        })
    }
    fn validate(&self) -> Result<()> {
        self.route_entries()?;
        self.link_entries()?;
        for port in self.disabled_ports.split_whitespace() {
            require(
                [
                    "PC1:eth0", "SW1:p0", "SW1:p1", "R1:eth0", "R1:eth1", "PC2:eth0",
                ]
                .contains(&port),
                "Port disable tidak dikenal",
            )?;
        }
        Self::preset(&self.case)?;
        require(
            self.addresses.len() == PORTS.len(),
            "Empat port L3 wajib tersedia",
        )?;
        for port in PORTS {
            cidr(self.addresses.get(port).ok_or("Port L3 tidak ditemukan")?)?;
        }
        ip(&self.destination)?;
        if let Some(via) = &self.route_via {
            ip(via)?;
        }
        require((1..=255).contains(&self.ttl), "TTL harus 1..255")?;
        require(
            (1..=1_000_000_000_000).contains(&self.bitrate),
            "Bitrate harus 1..1e12 bps",
        )?;
        require(self.propagation <= HORIZON, "Propagation maksimum 5 detik")
    }
    pub fn route_text(&self) -> String {
        self.routes.clone().unwrap_or_else(|| {
            let mut text = "PC1 0.0.0.0/0 eth0 10.0.0.1; PC2 0.0.0.0/0 eth0 10.0.1.1".to_owned();
            if let Some(via) = &self.route_via {
                text.push_str(&format!("; R1 198.51.100.2/32 eth1 {via}"));
            }
            text
        })
    }
    pub fn set_alias_route(&mut self, via: Option<String>) {
        if self.routes.is_some() {
            let mut rows: Vec<String> = self
                .route_text()
                .split(';')
                .filter(|r| {
                    let p: Vec<_> = r.split_whitespace().collect();
                    !(p.first() == Some(&"R1") && p.get(1) == Some(&"198.51.100.2/32"))
                })
                .map(str::to_owned)
                .collect();
            if let Some(v) = &via {
                rows.push(format!("R1 198.51.100.2/32 eth1 {v}"));
            }
            self.routes = Some(rows.join("; "));
        }
        self.route_via = via;
    }
    // None preserves the frozen fixture routes; Some replaces the whole static table.
    pub fn route_entries(&self) -> Result<Vec<RouteEntry>> {
        let mut result = Vec::new();
        if let Some(routes) = &self.routes {
            for row in routes.split(';').filter(|r| !r.trim().is_empty()) {
                let parts: Vec<_> = row.split_whitespace().collect();
                require(parts.len() == 4, "Route: device prefix port gateway; ...")?;
                let port = format!("{}:{}", parts[0], parts[2]);
                require(
                    PORTS.contains(&port.as_str()),
                    "Route harus memakai port L3 milik device",
                )?;
                let (address, prefix) = cidr(parts[1])?;
                let mask = if prefix == 0 {
                    0
                } else {
                    u32::MAX << (32 - prefix)
                };
                require(
                    address & mask == address,
                    "Route prefix harus network address",
                )?;
                require(
                    !result
                        .iter()
                        .any(|r: &RouteEntry| r.0 == parts[0] && r.2 == address && r.3 == prefix),
                    "Duplicate route prefix",
                )?;
                result.push((parts[0].into(), port, address, prefix, ip(parts[3])?));
            }
        }
        require(result.len() <= 32, "Maksimum 32 route")?;
        Ok(result)
    }
    pub fn link_entries(&self) -> Result<BTreeMap<String, (u64, u64)>> {
        let mut result = BTreeMap::new();
        for row in self.links.split(';').filter(|r| !r.trim().is_empty()) {
            let parts: Vec<_> = row.split_whitespace().collect();
            require(parts.len() == 3, "Link: L1 bitrate_bps delay_ns; ...")?;
            require(["L1", "L2", "L3"].contains(&parts[0]), "Link tidak dikenal")?;
            let bitrate = parts[1].parse::<u64>().map_err(err)?;
            let delay = parts[2].parse::<u64>().map_err(err)?;
            require(
                (1..=1_000_000_000_000).contains(&bitrate) && delay <= HORIZON,
                "Batas bitrate/delay tidak valid",
            )?;
            require(
                result.insert(parts[0].into(), (bitrate, delay)).is_none(),
                "Duplicate link",
            )?;
        }
        Ok(result)
    }
    pub fn to_json(&self) -> Value {
        json!({"routes":self.routes,"disabled_ports":self.disabled_ports,"links":self.links,"case":self.case,"addresses":self.addresses,"route_via":self.route_via,
            "pc2_enabled":self.pc2_enabled,"destination":self.destination,"ttl":self.ttl,
            "bitrate":self.bitrate,"propagation":self.propagation})
    }
    fn from_json(value: &Value) -> Result<Self> {
        require(
            value
                .as_object()
                .is_some_and(|o| o.len() == 8 || o.len() == 11),
            "Konfigurasi tidak dikenal",
        )?;
        let addresses = value["addresses"]
            .as_object()
            .ok_or("addresses harus object")?
            .iter()
            .map(|(k, v)| {
                Ok((
                    k.clone(),
                    v.as_str().ok_or("Alamat harus string")?.to_owned(),
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        let object = value.as_object().unwrap();
        let legacy = object.len() == 8;
        for key in [
            "case",
            "addresses",
            "route_via",
            "pc2_enabled",
            "destination",
            "ttl",
            "bitrate",
            "propagation",
        ] {
            require(
                object.contains_key(key),
                "Field Config wajib tidak ditemukan",
            )?;
        }
        if !legacy {
            for key in ["routes", "disabled_ports", "links"] {
                require(
                    object.contains_key(key),
                    "Field Config v2 wajib tidak ditemukan",
                )?;
            }
        }
        let config = Self {
            routes: if legacy || value["routes"].is_null() {
                None
            } else {
                Some(text(value, "routes")?)
            },
            disabled_ports: if legacy {
                String::new()
            } else {
                text(value, "disabled_ports")?
            },
            links: if legacy {
                String::new()
            } else {
                text(value, "links")?
            },
            case: text(value, "case")?,
            addresses,
            route_via: if value["route_via"].is_null() {
                None
            } else {
                Some(text(value, "route_via")?)
            },
            pc2_enabled: value["pc2_enabled"]
                .as_bool()
                .ok_or("pc2_enabled harus boolean")?,
            destination: text(value, "destination")?,
            ttl: u32::try_from(number(value, "ttl")?).map_err(err)?,
            bitrate: number(value, "bitrate")?,
            propagation: number(value, "propagation")?,
        };
        config.validate()?;
        Ok(config)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PacketView {
    pub packet: Packet,
    pub fields: Vec<Field>,
    pub decode_error: Option<String>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Frame {
    pub summary: Summary,
    pub trace: Vec<Trace>,
    pub packets: BTreeMap<u64, PacketView>,
    pub transmissions: BTreeMap<u64, Transmission>,
}
pub struct Model {
    pub config: Config,
    pub ids: BTreeMap<String, u64>,
    pub frame: Frame,
    pub baseline: Option<Value>,
    network: Network,
    started: bool,
    commands: Vec<u64>,
}
impl Model {
    pub fn new(config: Config) -> Result<Self> {
        config.validate()?;
        // The topology is a versioned, embedded fixture. External files edit only Config.
        let fixture: Value = serde_json::from_str(FIXTURE).map_err(err)?;
        let mut network =
            Network::new(fixture["clock"]["seed"].as_u64().unwrap(), 10_000).map_err(err)?;
        let mut ids = BTreeMap::new();
        for node in fixture["nodes"].as_array().unwrap() {
            let name = node["id"].as_str().unwrap();
            let role = match node["kind"].as_str().unwrap() {
                "host" => DeviceRole::Host,
                "switch" => DeviceRole::Switch,
                _ => DeviceRole::Router,
            };
            let device = network.device(role).map_err(err)?;
            ids.insert(name.to_owned(), device);
            for port in node["ports"].as_array().unwrap() {
                let name = format!("{name}:{}", port["id"].as_str().unwrap());
                let id = network.port(device).map_err(err)?;
                ids.insert(name.clone(), id);
                let mac = port["mac"]
                    .as_str()
                    .map(|s| u64::from_str_radix(&s.replace(':', ""), 16).unwrap())
                    .unwrap_or(0);
                let (address, prefix) = config
                    .addresses
                    .get(&name)
                    .map(|s| cidr(s))
                    .transpose()?
                    .unwrap_or((0, 0));
                network
                    .configure_port(
                        id,
                        mac,
                        address,
                        prefix,
                        (name != "PC2:eth0" || config.pc2_enabled)
                            && !config.disabled_ports.split_whitespace().any(|p| p == name),
                    )
                    .map_err(err)?;
            }
            if let Some(aliases) = node["local_ipv4"].as_array() {
                for alias in aliases {
                    network
                        .alias(device, cidr(alias.as_str().unwrap())?.0)
                        .map_err(err)?;
                }
            }
            for route in node["routes"].as_array().unwrap() {
                if config.routes.is_some() {
                    continue;
                }
                let via = if name == "R1" {
                    let Some(via) = &config.route_via else {
                        continue;
                    };
                    ip(via)?
                } else {
                    ip(route["via"].as_str().unwrap())?
                };
                let (address, prefix) = cidr(route["prefix"].as_str().unwrap())?;
                network
                    .route(
                        device,
                        ids[&format!("{name}:{}", route["port"].as_str().unwrap())],
                        address,
                        prefix,
                        via,
                    )
                    .map_err(err)?;
            }
        }
        for (device, port, address, prefix, via) in config.route_entries()? {
            network
                .route(ids[&device], ids[&port], address, prefix, via)
                .map_err(err)?;
        }
        let links = config.link_entries()?;
        for edge in fixture["edges"].as_array().unwrap() {
            let (bitrate, propagation) = links
                .get(edge["id"].as_str().unwrap())
                .copied()
                .unwrap_or((config.bitrate, config.propagation));
            let endpoints = edge["endpoints"].as_array().unwrap();
            let id = network
                .link(
                    ids[endpoints[0].as_str().unwrap()],
                    ids[endpoints[1].as_str().unwrap()],
                    bitrate,
                    propagation,
                )
                .map_err(err)?;
            ids.insert(edge["id"].as_str().unwrap().to_owned(), id);
        }
        let mut model = Self {
            config,
            ids,
            network,
            frame: Frame::default(),
            started: false,
            commands: Vec::new(),
            baseline: None,
        };
        model.refresh()?;
        Ok(model)
    }
    pub fn entity_name(&self, id: u64) -> &str {
        self.ids
            .iter()
            .find_map(|(name, value)| (*value == id).then_some(name.as_str()))
            .unwrap_or("—")
    }
    pub fn finished(&self) -> bool {
        self.started && (self.frame.summary.failed != 0 || self.frame.summary.now_ns >= HORIZON)
    }
    pub fn advance(&mut self, budget: u64) -> Result<()> {
        require((1..=8).contains(&budget), "Budget Studio harus 1..8 event")?;
        require(
            self.commands.len() < 1024,
            "Batas 1024 batch tercapai; reset skenario",
        )?;
        require(
            !self.finished(),
            "Run sudah berhenti; reset untuk mengulang",
        )?;
        if !self.started {
            if self.config.case == "S05" {
                self.network
                    .inject(self.ids["PC1:eth0"], 1, &[0x45, 0])
                    .map_err(err)?;
            } else {
                self.network
                    .ping(
                        self.ids["PC1:eth0"],
                        ip(&self.config.destination)?,
                        self.config.ttl,
                    )
                    .map_err(err)?;
            }
            self.started = true;
        }
        let result = self.network.run_until(HORIZON, budget).map_err(err);
        self.commands.push(budget);
        // The !Send owner is mutated only on this thread. Publish a complete copied
        // frame after the command; rendering never observes half of a refresh.
        self.refresh()?;
        result.map(|_| ())
    }
    fn refresh(&mut self) -> Result<()> {
        let mut next = self.frame.clone();
        next.summary = self.network.summary().map_err(err)?;
        for id in (next.packets.len() as u64 + 1)..=next.summary.packet_versions {
            let packet = self.network.packet(id).map_err(err)?;
            let (fields, decode_error) = match self.network.fields(id) {
                Ok(fields) => (fields, None),
                Err(e @ (Error::InvalidPacket | Error::Unsupported)) => {
                    (Vec::new(), Some(e.to_string()))
                }
                Err(e) => return Err(e.to_string()),
            };
            next.packets.insert(
                id,
                PacketView {
                    packet,
                    fields,
                    decode_error,
                },
            );
        }
        for id in (next.transmissions.len() as u64 + 1)..=next.summary.transmissions {
            next.transmissions
                .insert(id, self.network.transmission(id).map_err(err)?);
        }
        loop {
            let batch = self
                .network
                .trace(next.trace.last().map_or(0, |t| t.ordinal), 256)
                .map_err(err)?;
            if batch.is_empty() {
                break;
            }
            next.trace.extend(batch);
        }
        if next.trace.len() > 10_000 {
            next.trace.drain(..next.trace.len() - 10_000);
        }
        self.frame = next;
        Ok(())
    }
    pub fn branch(&self, config: Config) -> Result<Self> {
        let mut branch = Self::new(config)?;
        branch.baseline = Some(
            json!({"config":self.config.to_json(),"commands":self.commands,"report":self.report()}),
        );
        Ok(branch)
    }
    pub fn report(&self) -> Value {
        json!({"summary":summary_json(self.frame.summary),
            "trace":self.frame.trace.iter().map(|t| json!({"ordinal":t.ordinal,"event_id":t.event_id,"time_ns":t.time_ns,
                "cause_id":t.cause_id,"entity":t.entity,"packet_id":t.packet_id,"version":t.version,"prior":t.prior,
                "transmission":t.transmission,"detail_a":t.detail_a,"detail_b":t.detail_b,"kind":t.kind,"reason":t.reason})).collect::<Vec<_>>(),
            "packets":self.frame.packets.values().map(|v| json!({"version_id":v.packet.version_id,"packet_id":v.packet.packet_id,
                "parent_version":v.packet.parent_version,"cause_event":v.packet.cause_event,"layer":v.packet.layer,"bytes":v.packet.bytes,
                "decode_error":v.decode_error,"fields":v.fields.iter().map(|f| json!({"code":f.code,"byte_offset":f.byte_offset,
                    "bit_offset":f.bit_offset,"bit_width":f.bit_width,"value":(f.bit_width<=64).then_some(f.value)})).collect::<Vec<_>>() })).collect::<Vec<_>>(),
            "transmissions":self.frame.transmissions.values().map(|t| json!({"id":t.id,"version":t.version,"from_port":t.from_port,
                "to_port":t.to_port,"link":t.link,"start_ns":t.start_ns,"arrival_ns":t.arrival_ns,"event_id":t.event_id,"cause_id":t.cause_id})).collect::<Vec<_>>()})
    }
    pub fn save(&self) -> Result<String> {
        let payload = serde_json::to_string_pretty(&json!({"schema_version":2,"model":MODEL,"fixture":"icmp-static-v1",
            "config":self.config.to_json(),"commands":self.commands,"report":self.report(),"baseline":self.baseline})).map_err(err)?;
        require(
            payload.len() <= FILE_LIMIT,
            "Hasil terlalu besar untuk schema v2 (maksimum 2 MiB)",
        )?;
        Ok(payload)
    }
    pub fn load(bytes: &[u8]) -> Result<Self> {
        require(
            bytes.len() <= FILE_LIMIT,
            "File terlalu besar (maksimum 2 MiB)",
        )?;
        let value: Value = serde_json::from_slice(bytes).map_err(err)?;
        require(
            value["schema_version"] == 1 || value["schema_version"] == 2,
            "Versi schema tidak didukung",
        )?;
        require(
            value["model"] == MODEL && value["fixture"] == "icmp-static-v1",
            "Model/fixture tidak didukung",
        )?;
        let commands = value["commands"].as_array().ok_or("commands harus array")?;
        require(commands.len() <= 1024, "Terlalu banyak command")?;
        if value["schema_version"] == 1 {
            require(
                value["config"].as_object().is_some_and(|c| c.len() == 8),
                "Schema v1 membutuhkan Config v1",
            )?;
        }
        let mut model = Self::new(Config::from_json(&value["config"])?)?;
        // Re-execute a bounded command log from an empty engine, then compare all
        // copied results. This is not restore of ARP/cache/queue state from a file.
        for (index, budget) in commands.iter().enumerate() {
            if let Err(error) = model.advance(budget.as_u64().ok_or("Budget bukan integer")?) {
                require(
                    model.frame.summary.failed != 0 && index + 1 == commands.len(),
                    &error,
                )?;
            }
        }
        require(
            model.report() == value["report"],
            "Hasil tersimpan tidak cocok dengan model saat ini; file tidak dimuat",
        )?;
        if !value["baseline"].is_null() {
            // Validate the source run too; imported comparison labels must not
            // claim an outcome which this model cannot reproduce. No nested history.
            let baseline = &value["baseline"];
            let source = json!({"schema_version":2,"model":MODEL,"fixture":"icmp-static-v1",
                "config":baseline["config"],"commands":baseline["commands"],"report":baseline["report"],"baseline":null});
            drop(Self::load(source.to_string().as_bytes())?);
            model.baseline = Some(value["baseline"].clone());
        }
        Ok(model)
    }
}
/// Read-only acceptance projection from 160 independent real kernel runs.
/// IDs/time are rebased; this is not one long-running or restored engine.
pub fn acceptance_history() -> Result<Frame> {
    let mut history = Frame::default();
    for run in 0..160u64 {
        let mut model = Model::new(Config::preset("S01")?)?;
        for _ in 0..1024 {
            if model.finished() {
                break;
            }
            model.advance(4)?;
        }
        require(
            model.finished() && model.frame.summary.outcome == 1,
            "History source did not finish",
        )?;
        let offset = run * 1_000_000;
        let id = |v| if v == 0 { 0 } else { v + offset };
        let time = run * HORIZON;
        history.summary.executed += model.frame.summary.executed;
        for (_, mut view) in model.frame.packets {
            let p = &mut view.packet;
            p.version_id = id(p.version_id);
            p.packet_id = id(p.packet_id);
            p.parent_version = id(p.parent_version);
            p.cause_event = id(p.cause_event);
            history.packets.insert(p.version_id, view);
        }
        for (_, mut tx) in model.frame.transmissions {
            tx.id = id(tx.id);
            tx.version = id(tx.version);
            tx.event_id = id(tx.event_id);
            tx.cause_id = id(tx.cause_id);
            tx.start_ns += time;
            tx.arrival_ns += time;
            history.transmissions.insert(tx.id, tx);
        }
        for mut row in model.frame.trace {
            row.ordinal = history.trace.len() as u64 + 1;
            row.event_id = id(row.event_id);
            row.cause_id = id(row.cause_id);
            row.packet_id = id(row.packet_id);
            row.version = id(row.version);
            row.prior = id(row.prior);
            row.transmission = id(row.transmission);
            row.time_ns += time;
            history.trace.push(row);
        }
    }
    history.summary.trace_total = history.trace.len() as u64;
    history.summary.trace_dropped = history.trace.len().saturating_sub(10_000) as u64;
    history
        .trace
        .drain(..history.trace.len().saturating_sub(10_000));
    history.summary.packet_versions = history.packets.len() as u64;
    history.summary.transmissions = history.transmissions.len() as u64;
    history.summary.now_ns = 160 * HORIZON;
    history.summary.outcome = 1;
    Ok(history)
}
pub fn summary_json(s: Summary) -> Value {
    json!({"now_ns":s.now_ns,"executed":s.executed,"trace_total":s.trace_total,"trace_dropped":s.trace_dropped,
        "packet_versions":s.packet_versions,"transmissions":s.transmissions,"failure_entity":s.failure_entity,
        "pending_events":s.pending_events,"pending_arp":s.pending_arp,"outcome":s.outcome,"reason":s.reason,"failed":s.failed})
}
pub fn outcome_name(outcome: u32) -> &'static str {
    match outcome {
        0 => "pending",
        1 => "echo_reply",
        2 => "unreachable",
        3 => "time_exceeded",
        4 => "arp_timeout",
        5 => "invalid_packet",
        6 => "unsupported",
        7 => "dropped",
        _ => "unknown",
    }
}
pub fn covers(field: &Field, byte: usize) -> bool {
    let start = field.byte_offset as usize * 8 + field.bit_offset as usize;
    let end = start + field.bit_width as usize;
    byte * 8 < end && byte * 8 + 8 > start
}
pub fn field_description(field: &Field) -> String {
    let value = if field.bit_width <= 64 {
        format!("{} / 0x{:x}", field.value, field.value)
    } else {
        "lihat raw bytes".into()
    };
    format!(
        "{} · byte {} bit {} · {} bit · {value}",
        field_name(field.code),
        field.byte_offset,
        field.bit_offset,
        field.bit_width
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use wyrlex_kernel::network::{kind_name, reason_name};
    // Tests may own a source, branch and loader; the registry has four slots.
    static OWNERS: std::sync::Mutex<()> = std::sync::Mutex::new(());
    fn drain(model: &mut Model, budget: u64) {
        for _ in 0..1024 {
            if model.finished() {
                return;
            }
            model.advance(budget).unwrap();
        }
        panic!("bounded fixture did not terminate")
    }
    fn compare_phase3(model: &Model, reference: &str) {
        let reference: Value = serde_json::from_str(reference).unwrap();
        let packets = model.frame.packets.values().map(|view| {
            let p = &view.packet;
            json!({"version":p.version_id,"packet_id":p.packet_id,"parent_version":p.parent_version,
                "cause_event":p.cause_event,"layer":if p.layer==1{"ipv4"}else{"ethernet"},
                "hex":p.bytes.iter().map(|b|format!("{b:02x}")).collect::<String>(),
                "fields":view.fields.iter().map(|f|json!({"name":field_name(f.code),"byte_offset":f.byte_offset,
                    "bit_offset_msb":f.bit_offset,"bit_width":f.bit_width,"value":(f.bit_width<=64).then_some(f.value)})).collect::<Vec<_>>()})
        }).collect::<Vec<_>>();
        assert_eq!(json!(packets), reference["packets"]);
        let mut report = model.report();
        for trace in report["trace"].as_array_mut().unwrap() {
            trace["kind"] = json!(kind_name(trace["kind"].as_u64().unwrap() as u32));
            trace["reason"] = json!(reason_name(trace["reason"].as_u64().unwrap() as u32));
        }
        assert_eq!(report["trace"], reference["trace"]);
        for tx in report["transmissions"].as_array_mut().unwrap() {
            let object = tx.as_object_mut().unwrap();
            let from = object.remove("from_port").unwrap();
            let to = object.remove("to_port").unwrap();
            object.insert("from".into(), from);
            object.insert("to".into(), to);
        }
        assert_eq!(report["transmissions"], reference["transmission_records"]);
        assert_eq!(json!(model.ids), reference["entities"]);
        assert_eq!(
            json!(model.frame.summary.executed),
            reference["executed_events"]
        );
        assert_eq!(json!(model.frame.summary.now_ns), reference["now_ns"]);
        assert_eq!(
            json!(reason_name(model.frame.summary.reason)),
            reference["reason"]
        );
    }
    #[test]
    fn presets_batch_step_and_owned_frames_match() {
        let _owners = OWNERS.lock().unwrap();
        for (case, outcome) in [("S01", 1), ("S02", 2), ("S03", 3), ("S04", 4), ("S05", 5)] {
            let mut batch = Model::new(Config::preset(case).unwrap()).unwrap();
            drain(&mut batch, 8);
            let report = batch.report();
            assert_eq!(batch.frame.summary.outcome, outcome);
            assert_eq!(batch.frame.summary.failed, 0);
            assert_eq!(batch.frame.summary.pending_events, 0);
            drop(batch);
            let mut step = Model::new(Config::preset(case).unwrap()).unwrap();
            step.advance(1).unwrap();
            let owned = step.frame.clone();
            let immutable = owned.clone();
            drain(&mut step, 1);
            assert_eq!(owned, immutable);
            assert_eq!(step.report(), report, "{case}");
            let reference = match case {
                "S01" => include_str!("../../../docs/evidence/phase-3/S01.json"),
                "S02" => include_str!("../../../docs/evidence/phase-3/S02.json"),
                "S03" => include_str!("../../../docs/evidence/phase-3/S03.json"),
                "S04" => include_str!("../../../docs/evidence/phase-3/S04.json"),
                _ => include_str!("../../../docs/evidence/phase-3/S05.json"),
            };
            compare_phase3(&step, reference);
            println!(
                "{case}: outcome={outcome}, batch=step, events={}, trace={}, packets={}",
                step.frame.summary.executed,
                step.frame.summary.trace_total,
                step.frame.summary.packet_versions
            );
        }
    }
    #[test]
    fn branch_repairs_missing_route_without_changing_source() {
        let _owners = OWNERS.lock().unwrap();
        let mut source = Model::new(Config::preset("S02").unwrap()).unwrap();
        drain(&mut source, 8);
        let original = source.save().unwrap();
        let mut config = source.config.clone();
        config.route_via = Some("10.0.1.2".into());
        let mut branch = source.branch(config).unwrap();
        drain(&mut branch, 8);
        assert_eq!(branch.frame.summary.outcome, 1);
        assert_eq!(
            branch.baseline.as_ref().unwrap()["report"]["summary"]["outcome"],
            2
        );
        assert_eq!(source.save().unwrap(), original);
        assert!(
            branch
                .frame
                .trace
                .iter()
                .any(|t| t.kind == 2 && t.reason == 201)
        );
        println!("S02 branch: unreachable -> echo_reply; source report unchanged");
    }
    #[test]
    fn save_load_partial_complete_and_invalid_files() {
        let _owners = OWNERS.lock().unwrap();
        let mut model = Model::new(Config::preset("S04").unwrap()).unwrap();
        model.advance(1).unwrap();
        let saved = model.save().unwrap();
        let mut loaded = Model::load(saved.as_bytes()).unwrap();
        assert_eq!(loaded.frame, model.frame);
        drain(&mut model, 8);
        drain(&mut loaded, 8);
        assert_eq!(loaded.report(), model.report());
        drop(loaded);
        let saved = model.save().unwrap();
        assert_eq!(
            Model::load(saved.as_bytes()).unwrap().report(),
            model.report()
        );
        let mut bad: Value = serde_json::from_str(&saved).unwrap();
        bad["schema_version"] = json!(99);
        assert!(Model::load(bad.to_string().as_bytes()).is_err());
        bad["schema_version"] = json!(2);
        bad["report"]["summary"]["outcome"] = json!(1);
        assert!(Model::load(bad.to_string().as_bytes()).is_err());
        bad["commands"] = json!([9]);
        assert!(Model::load(bad.to_string().as_bytes()).is_err());
        assert!(Model::load(&vec![b' '; FILE_LIMIT + 1]).is_err());
    }
    #[test]
    fn configuration_errors_and_packet_field_intersection() {
        let _owners = OWNERS.lock().unwrap();
        let mut config = Config::preset("S01").unwrap();
        config
            .addresses
            .insert("PC1:eth0".into(), "10.0.0.2/33".into());
        assert!(Model::new(config).is_err());
        let field = Field {
            code: 30,
            byte_offset: 14,
            bit_offset: 0,
            bit_width: 4,
            value: 4,
        };
        assert!(covers(&field, 14));
        assert!(!covers(&field, 13));
        assert!(!covers(&field, 15));
        let mut model = Model::new(Config::preset("S01").unwrap()).unwrap();
        drain(&mut model, 8);
        let ttl = model.frame.trace.iter().find(|t| t.kind == 13).unwrap();
        let before = &model.frame.packets[&ttl.prior];
        let after = &model.frame.packets[&ttl.version];
        assert_eq!(before.packet.bytes[8], 64);
        assert_eq!(after.packet.bytes[8], 63);
        assert_ne!(&before.packet.bytes[10..12], &after.packet.bytes[10..12]);
        assert!(after.fields.iter().any(|f| f.code == 38 && covers(f, 8)));
    }

    #[test]
    fn repaired_bundle_validates_baseline_and_keeps_active_report() {
        let _owners = OWNERS.lock().unwrap();
        let mut source = Model::new(Config::preset("S02").unwrap()).unwrap();
        drain(&mut source, 4);
        let mut config = source.config.clone();
        config.route_via = Some("10.0.1.2".into());
        let mut branch = source.branch(config).unwrap();
        drain(&mut branch, 4);
        drop(source);
        let saved = branch.save().unwrap();
        let loaded = Model::load(saved.as_bytes()).unwrap();
        assert_eq!(loaded.save().unwrap(), saved);
        drop(loaded);
        let mut forged: Value = serde_json::from_str(&saved).unwrap();
        forged["baseline"]["report"]["summary"]["outcome"] = json!(1);
        assert!(Model::load(forged.to_string().as_bytes()).is_err());
        assert_eq!(branch.save().unwrap(), saved);
        forged["model"] = json!("unknown-v2");
        assert!(Model::load(forged.to_string().as_bytes()).is_err());
    }

    #[test]
    fn horizon_with_pending_work_is_never_success() {
        let _owners = OWNERS.lock().unwrap();
        let mut config = Config::preset("S01").unwrap();
        config.bitrate = 1;
        let mut model = Model::new(config).unwrap();
        model.advance(4).unwrap();
        assert!(model.finished());
        assert_eq!(model.frame.summary.now_ns, HORIZON);
        // At one bit/second, ARP times out at 3 s while old frames remain
        // scheduled beyond the 5 s horizon. Pending frames do not erase failure.
        assert_eq!(model.frame.summary.outcome, 4);
        assert!(model.frame.summary.pending_events > 0);
        let saved = model.save().unwrap();
        assert_eq!(
            Model::load(saved.as_bytes()).unwrap().report(),
            model.report()
        );
        println!(
            "Horizon reached with pending frames: outcome=arp_timeout, not success; save/load identical"
        );
    }
    #[test]
    fn draft_subnet_routes_links_and_legacy_session() {
        let _guard = OWNERS.lock().unwrap();
        let legacy = include_bytes!("../../../docs/evidence/phase-4/native-session.wb.json");
        let old = Model::load(legacy).unwrap();
        assert_eq!(old.frame.summary.outcome, 1);
        drop(old);
        let mut source = Model::new(Config::preset("S01").unwrap()).unwrap();
        drain(&mut source, 4);
        let original = source.report();
        let mut config = source.config.clone();
        config
            .addresses
            .insert("PC1:eth0".into(), "10.2.0.2/24".into());
        assert!(source.branch(config.clone()).is_err());
        assert_eq!(source.report(), original);
        config
            .addresses
            .insert("R1:eth0".into(), "10.2.0.1/24".into());
        config.routes = Some("PC1 0.0.0.0/0 eth0 10.2.0.1; PC2 0.0.0.0/0 eth0 10.0.1.1".into());
        config.links = "L1 10000000 5000; L3 100000000 2000".into();
        let mut branch = source.branch(config.clone()).unwrap();
        drain(&mut branch, 1);
        assert_eq!(branch.frame.summary.outcome, 1);
        let saved = branch.save().unwrap();
        let loaded = Model::load(saved.as_bytes()).unwrap();
        assert_eq!(loaded.config, config);
        assert_eq!(loaded.report(), branch.report());
        drop(loaded);
        drop(branch);
        config.disabled_ports = "PC2:eth0".into();
        let mut down = source.branch(config.clone()).unwrap();
        drain(&mut down, 4);
        assert_eq!(down.frame.summary.outcome, 4);
        drop(down);
        for routes in [
            "PC1 0.0.0.0/0 eth0 10.9.0.1",
            "R1 10.0.1.5/24 eth1 10.0.1.2",
            "SW1 0.0.0.0/0 p0 10.0.0.1",
            "PC1 0.0.0.0/0 eth0 10.2.0.1; PC1 0.0.0.0/0 eth0 10.2.0.1",
        ] {
            config.routes = Some(routes.into());
            assert!(source.branch(config.clone()).is_err(), "accepted {routes}");
            assert_eq!(source.report(), original);
        }
    }

    #[test]
    fn explicit_longest_prefix_route_wins_and_removal_changes_outcome() {
        let _guard = OWNERS.lock().unwrap();
        let mut config = Config::preset("S02").unwrap();
        config.routes = Some("PC1 0.0.0.0/0 eth0 10.0.0.1; PC2 0.0.0.0/0 eth0 10.0.1.1; R1 198.51.100.0/24 eth1 10.0.1.99; R1 198.51.100.2/32 eth1 10.0.1.2".into());
        let mut model = Model::new(config.clone()).unwrap();
        drain(&mut model, 4);
        assert_eq!(model.frame.summary.outcome, 1);
        drop(model);
        config.routes = Some("PC1 0.0.0.0/0 eth0 10.0.0.1; PC2 0.0.0.0/0 eth0 10.0.1.1".into());
        let mut model = Model::new(config).unwrap();
        drain(&mut model, 4);
        assert_eq!(model.frame.summary.outcome, 2);
    }
    #[test]
    fn acceptance_history_preserves_all_rebased_references() {
        let _guard = OWNERS.lock().unwrap();
        let h = acceptance_history().unwrap();
        assert_eq!(h.trace.len(), 10_000);
        assert_eq!(h.summary.trace_dropped, 80);
        assert_eq!(h.packets.len(), 2560);
        for r in &h.trace {
            for v in [r.version, r.prior] {
                if v != 0 {
                    assert!(h.packets.contains_key(&v));
                }
            }
            if r.transmission != 0 {
                assert!(h.transmissions.contains_key(&r.transmission));
            }
            if r.version != 0 {
                assert_eq!(h.packets[&r.version].packet.packet_id, r.packet_id);
            }
        }
        for t in h.transmissions.values() {
            assert!(h.packets.contains_key(&t.version));
        }
    }
}
