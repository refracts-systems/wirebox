//! Phase-3 network ownership. All packet/trace/field views are copied from Zig.
use crate::{
    Error, api_lock,
    simulation::{DeviceRole, Progress, RunError},
    status,
};
use std::{marker::PhantomData, rc::Rc};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Summary {
    pub now_ns: u64,
    pub executed: u64,
    pub trace_total: u64,
    pub trace_dropped: u64,
    pub packet_versions: u64,
    pub transmissions: u64,
    pub failure_entity: u64,
    pub pending_events: u32,
    pub pending_arp: u32,
    pub outcome: u32,
    pub reason: u32,
    pub failed: u32,
    reserved: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Noise {
    pub loss_per_million: u32,
    reserved: u32,
    pub jitter_ns: u64,
    pub draws: u64,
    pub dropped: u64,
    pub delayed: u64,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CheckpointInfo {
    pub model_version: u32,
    pub input_kind: u32,
    pub input_port: u64,
    pub source_ip: u32,
    pub destination: u32,
    reserved: u32,
    pub now_ns: u64,
    pub rng_state: u64,
    pub pending_events: u32,
    reserved2: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Trace {
    pub ordinal: u64,
    pub event_id: u64,
    pub time_ns: u64,
    pub cause_id: u64,
    pub entity: u64,
    pub packet_id: u64,
    pub version: u64,
    pub prior: u64,
    pub transmission: u64,
    pub detail_a: u64,
    pub detail_b: u64,
    pub kind: u32,
    pub reason: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Transmission {
    pub id: u64,
    pub version: u64,
    pub from_port: u64,
    pub to_port: u64,
    pub link: u64,
    pub start_ns: u64,
    pub arrival_ns: u64,
    pub event_id: u64,
    pub cause_id: u64,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Field {
    pub code: u32,
    pub byte_offset: u32,
    pub bit_offset: u32,
    pub bit_width: u32,
    pub value: u64,
}
#[repr(C)]
struct RawPacket {
    version_id: u64,
    packet_id: u64,
    parent_version: u64,
    cause_event: u64,
    layer: u32,
    length: u32,
    bytes: [u8; 1600],
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Packet {
    pub version_id: u64,
    pub packet_id: u64,
    pub parent_version: u64,
    pub cause_event: u64,
    pub layer: u32,
    pub bytes: Vec<u8>,
}
unsafe extern "C" {
    fn wb_net_create(version: u32, seed: u64, retention: usize, out: *mut u64) -> u32;
    fn wb_net_destroy(handle: u64) -> u32;
    fn wb_net_checkpoint(handle: u64, out: *mut u64) -> u32;
    fn wb_net_checkpoint_destroy(handle: u64) -> u32;
    fn wb_net_restore(handle: u64, checkpoint: u64) -> u32;
    fn wb_net_fork(checkpoint: u64, out: *mut u64) -> u32;
    fn wb_net_add(handle: u64, kind: u32, role: u32, a: u64, b: u64, out: *mut u64) -> u32;
    fn wb_net_port(handle: u64, port: u64, mac: u64, ip: u32, prefix: u32, enabled: u32) -> u32;
    fn wb_net_link(handle: u64, link: u64, bitrate: u64, propagation: u64) -> u32;
    fn wb_net_noise(handle: u64, loss_per_million: u32, jitter_ns: u64) -> u32;
    fn wb_net_alias(handle: u64, device: u64, ip: u32) -> u32;
    fn wb_net_route(
        handle: u64,
        device: u64,
        port: u64,
        network: u32,
        prefix: u32,
        via: u32,
    ) -> u32;
    fn wb_net_ping(handle: u64, port: u64, destination: u32, ttl: u32) -> u32;
    fn wb_net_inject(handle: u64, port: u64, layer: u32, bytes: *const u8, len: usize) -> u32;
    fn wb_net_run(handle: u64, horizon: u64, budget: u64, out: *mut Progress) -> u32;
    fn wb_net_summary(handle: u64, out: *mut Summary) -> u32;
    fn wb_net_noise_state(handle: u64, out: *mut Noise) -> u32;
    fn wb_net_checkpoint_info(handle: u64, out: *mut CheckpointInfo) -> u32;
    fn wb_net_packet(handle: u64, version: u64, out: *mut RawPacket) -> u32;
    fn wb_net_tx(handle: u64, id: u64, out: *mut Transmission) -> u32;
    fn wb_net_trace(
        handle: u64,
        after: u64,
        out: *mut Trace,
        capacity: usize,
        count: *mut usize,
    ) -> u32;
    fn wb_net_fields(
        handle: u64,
        version: u64,
        out: *mut Field,
        capacity: usize,
        count: *mut usize,
    ) -> u32;
}
/// A thread-bound network with a frozen configuration after its first workload.
/// ```compile_fail
/// let network = wyrlex_kernel::network::Network::new(0, 1024).unwrap();
/// std::thread::spawn(move || drop(network));
/// ```
pub struct Network {
    handle: Option<u64>,
    _thread: PhantomData<Rc<()>>,
}
/// Immutable, thread-bound checkpoint owned by Zig. It is a complete copy of
/// a network at one execution point, including pending work and PRNG state.
/// It is process-local, not a serialized checkpoint file.
///
/// ```compile_fail
/// let network = wyrlex_kernel::network::Network::new(0, 16).unwrap();
/// let checkpoint = network.checkpoint().unwrap();
/// std::thread::spawn(move || drop(checkpoint));
/// ```
pub struct Checkpoint {
    handle: Option<u64>,
    _thread: PhantomData<Rc<()>>,
}
impl Network {
    pub fn new(seed: u64, retention: usize) -> Result<Self, Error> {
        let _guard = api_lock();
        let mut handle = 0;
        // SAFETY: serialized create with a writable output; Zig owns the allocation.
        status(unsafe { wb_net_create(1, seed, retention, &mut handle) })?;
        Ok(Self {
            handle: Some(handle),
            _thread: PhantomData,
        })
    }
    fn add(&mut self, kind: u32, role: u32, a: u64, b: u64) -> Result<u64, Error> {
        let _guard = api_lock();
        let mut id = 0;
        // SAFETY: live handle, exclusive call, writable scalar output.
        status(unsafe { wb_net_add(self.handle.unwrap(), kind, role, a, b, &mut id) })?;
        Ok(id)
    }
    pub fn device(&mut self, role: DeviceRole) -> Result<u64, Error> {
        self.add(1, role as u32, 0, 0)
    }
    pub fn port(&mut self, device: u64) -> Result<u64, Error> {
        self.add(2, 0, device, 0)
    }
    pub fn link(&mut self, a: u64, b: u64, bitrate: u64, propagation: u64) -> Result<u64, Error> {
        if bitrate == 0 {
            return Err(Error::InvalidArgument);
        }
        let id = self.add(3, 0, a, b)?;
        let _guard = api_lock();
        // SAFETY: owned live handle and scalar configuration.
        status(unsafe { wb_net_link(self.handle.unwrap(), id, bitrate, propagation) })?;
        Ok(id)
    }
    pub fn set_noise(&mut self, loss_per_million: u32, jitter_ns: u64) -> Result<(), Error> {
        let _guard = api_lock();
        // SAFETY: scalar configuration is copied synchronously and validated in Zig.
        status(unsafe { wb_net_noise(self.handle.unwrap(), loss_per_million, jitter_ns) })
    }
    pub fn configure_port(
        &mut self,
        port: u64,
        mac: u64,
        ip: u32,
        prefix: u32,
        enabled: bool,
    ) -> Result<(), Error> {
        let _guard = api_lock();
        // SAFETY: owned live handle; numeric fields validated by Zig.
        status(unsafe {
            wb_net_port(
                self.handle.unwrap(),
                port,
                mac,
                ip,
                prefix,
                u32::from(enabled),
            )
        })
    }
    pub fn alias(&mut self, device: u64, ip: u32) -> Result<(), Error> {
        let _guard = api_lock();
        // SAFETY: owned live handle, scalar arguments.
        status(unsafe { wb_net_alias(self.handle.unwrap(), device, ip) })
    }
    pub fn route(
        &mut self,
        device: u64,
        port: u64,
        network: u32,
        prefix: u32,
        via: u32,
    ) -> Result<(), Error> {
        let _guard = api_lock();
        // SAFETY: owned live handle, scalar arguments validated in Zig.
        status(unsafe { wb_net_route(self.handle.unwrap(), device, port, network, prefix, via) })
    }
    pub fn ping(&mut self, port: u64, destination: u32, ttl: u32) -> Result<(), Error> {
        let _guard = api_lock();
        // SAFETY: serialized mutation; no borrowed data retained.
        status(unsafe { wb_net_ping(self.handle.unwrap(), port, destination, ttl) })
    }
    pub fn inject(&mut self, port: u64, layer: u32, bytes: &[u8]) -> Result<(), Error> {
        let _guard = api_lock();
        // SAFETY: bytes remain valid throughout the synchronous copy.
        status(unsafe {
            wb_net_inject(
                self.handle.unwrap(),
                port,
                layer,
                bytes.as_ptr(),
                bytes.len(),
            )
        })
    }
    pub fn run_until(&mut self, horizon: u64, budget: u64) -> Result<Progress, RunError> {
        let _guard = api_lock();
        let mut progress = Progress::default();
        // SAFETY: valid aligned progress output; bounded exclusive mutation.
        let code = unsafe { wb_net_run(self.handle.unwrap(), horizon, budget, &mut progress) };
        status(code).map_err(|error| RunError { error, progress })?;
        Ok(progress)
    }
    pub fn checkpoint(&self) -> Result<Checkpoint, Error> {
        let _guard = api_lock();
        let mut handle = 0;
        // SAFETY: Zig allocates an independent value copy and returns only an opaque handle.
        status(unsafe { wb_net_checkpoint(self.handle.unwrap(), &mut handle) })?;
        Ok(Checkpoint {
            handle: Some(handle),
            _thread: PhantomData,
        })
    }
    pub fn restore(&mut self, checkpoint: &Checkpoint) -> Result<(), Error> {
        let _guard = api_lock();
        let checkpoint_handle = checkpoint.handle.ok_or(Error::InvalidHandle)?;
        // SAFETY: both handles are live and Zig copies the complete value-owned state.
        status(unsafe { wb_net_restore(self.handle.unwrap(), checkpoint_handle) })
    }
    pub fn noise(&self) -> Result<Noise, Error> {
        let _guard = api_lock();
        let mut out = Noise::default();
        // SAFETY: initialized output is filled synchronously; no pointer is retained.
        status(unsafe { wb_net_noise_state(self.handle.unwrap(), &mut out) })?;
        Ok(out)
    }
    pub fn summary(&self) -> Result<Summary, Error> {
        let _guard = api_lock();
        let mut out = Summary::default();
        // SAFETY: valid aligned output, owned handle.
        status(unsafe { wb_net_summary(self.handle.unwrap(), &mut out) })?;
        Ok(out)
    }
    pub fn packet(&self, id: u64) -> Result<Packet, Error> {
        let _guard = api_lock();
        let mut out = RawPacket {
            version_id: 0,
            packet_id: 0,
            parent_version: 0,
            cause_event: 0,
            layer: 0,
            length: 0,
            bytes: [0; 1600],
        };
        // SAFETY: complete initialized ABI output; kernel copies bytes and retains no pointer.
        status(unsafe { wb_net_packet(self.handle.unwrap(), id, &mut out) })?;
        if out.length > 1600 {
            return Err(Error::InvalidResponse(out.length));
        }
        Ok(Packet {
            version_id: out.version_id,
            packet_id: out.packet_id,
            parent_version: out.parent_version,
            cause_event: out.cause_event,
            layer: out.layer,
            bytes: out.bytes[..out.length as usize].to_vec(),
        })
    }
    pub fn transmission(&self, id: u64) -> Result<Transmission, Error> {
        let _guard = api_lock();
        let mut out = Transmission::default();
        // SAFETY: complete initialized output and owned handle.
        status(unsafe { wb_net_tx(self.handle.unwrap(), id, &mut out) })?;
        Ok(out)
    }
    pub fn trace(&self, after: u64, capacity: usize) -> Result<Vec<Trace>, Error> {
        if capacity > 256 {
            return Err(Error::InvalidArgument);
        }
        let _guard = api_lock();
        let mut out = vec![Trace::default(); capacity];
        let mut count = 0;
        // SAFETY: capacity initialized records, non-overlapping count output.
        status(unsafe {
            wb_net_trace(
                self.handle.unwrap(),
                after,
                out.as_mut_ptr(),
                capacity,
                &mut count,
            )
        })?;
        if count > capacity {
            return Err(Error::InvalidResponse(0));
        }
        out.truncate(count);
        Ok(out)
    }
    pub fn fields(&self, id: u64) -> Result<Vec<Field>, Error> {
        let _guard = api_lock();
        let mut out = vec![Field::default(); 40];
        let mut count = 0;
        // SAFETY: initialized records for the full view capacity.
        status(unsafe {
            wb_net_fields(
                self.handle.unwrap(),
                id,
                out.as_mut_ptr(),
                out.len(),
                &mut count,
            )
        })?;
        if count > out.len() {
            return Err(Error::InvalidResponse(0));
        }
        out.truncate(count);
        Ok(out)
    }
    pub fn close(mut self) -> Result<(), Error> {
        self.close_inner()
    }
    fn close_inner(&mut self) -> Result<(), Error> {
        if let Some(handle) = self.handle.take() {
            let _guard = api_lock();
            // SAFETY: sole owner destroys once using the allocating Zig side.
            status(unsafe { wb_net_destroy(handle) })?;
        }
        Ok(())
    }
}
impl Checkpoint {
    pub fn info(&self) -> Result<CheckpointInfo, Error> {
        let _guard = api_lock();
        let mut out = CheckpointInfo::default();
        let handle = self.handle.ok_or(Error::InvalidHandle)?;
        // SAFETY: initialized output is filled from the immutable checkpoint.
        status(unsafe { wb_net_checkpoint_info(handle, &mut out) })?;
        Ok(out)
    }

    pub fn fork(&self) -> Result<Network, Error> {
        let _guard = api_lock();
        let mut handle = 0;
        let checkpoint_handle = self.handle.ok_or(Error::InvalidHandle)?;
        // SAFETY: Zig allocates a separate network copy from the immutable checkpoint.
        status(unsafe { wb_net_fork(checkpoint_handle, &mut handle) })?;
        Ok(Network {
            handle: Some(handle),
            _thread: PhantomData,
        })
    }
}
impl Drop for Checkpoint {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _guard = api_lock();
            // SAFETY: sole owner destroys the checkpoint exactly once.
            let _ = unsafe { wb_net_checkpoint_destroy(handle) };
        }
    }
}
impl Drop for Network {
    fn drop(&mut self) {
        let _ = self.close_inner();
    }
}

pub fn reason_name(reason: u32) -> &'static str {
    match reason {
        0 => "none",
        1 => "truncated_ethernet",
        2 => "unsupported_ethertype",
        3 => "truncated_arp",
        4 => "invalid_arp",
        5 => "truncated_ipv4",
        6 => "invalid_ipv4",
        7 => "invalid_ipv4_checksum",
        8 => "unsupported_options",
        9 => "unsupported_fragment",
        10 => "unsupported_protocol",
        11 => "truncated_icmp",
        12 => "invalid_icmp_checksum",
        13 => "unsupported_icmp",
        14 => "unsupported_mtu",
        15 => "unsupported_vlan",
        16 => "invalid_ethernet",
        100 => "no_route",
        101 => "ttl_expired",
        102 => "arp_timeout",
        103 => "port_down",
        104 => "mac_filter",
        105 => "not_router",
        106 => "icmp_error_suppressed",
        107 => "unsolicited_reply",
        108 => "unsupported_address",
        109 => "link_loss",
        200 => "connected_route",
        201 => "static_route",
        _ => "unknown",
    }
}
pub fn kind_name(kind: u32) -> &'static str {
    match kind {
        1 => "packet_version",
        2 => "route_selected",
        3 => "arp_lookup",
        4 => "arp_request",
        5 => "arp_learn",
        6 => "arp_reply",
        7 => "mac_learn",
        8 => "flood",
        9 => "switch_forward",
        10 => "frame_tx",
        11 => "frame_rx",
        12 => "decapsulate",
        13 => "decrement_ttl",
        14 => "icmp_echo_reply",
        15 => "icmp_error",
        16 => "delivered",
        17 => "drop",
        18 => "arp_timeout",
        19 => "packet_buffered",
        20 => "arp_cache_hit",
        _ => "unknown",
    }
}
pub fn field_name(code: u32) -> &'static str {
    match code {
        1 => "ethernet.dst",
        2 => "ethernet.src",
        3 => "ethernet.type",
        4 => "ethernet.padding",
        10 => "arp.hardware_type",
        11 => "arp.protocol_type",
        12 => "arp.hardware_len",
        13 => "arp.protocol_len",
        14 => "arp.opcode",
        15 => "arp.sender_mac",
        16 => "arp.sender_ip",
        17 => "arp.target_mac",
        18 => "arp.target_ip",
        30 => "ipv4.version",
        31 => "ipv4.ihl",
        32 => "ipv4.dscp",
        33 => "ipv4.ecn",
        34 => "ipv4.total_length",
        35 => "ipv4.identification",
        36 => "ipv4.flags",
        37 => "ipv4.fragment_offset",
        38 => "ipv4.ttl",
        39 => "ipv4.protocol",
        40 => "ipv4.checksum",
        41 => "ipv4.src",
        42 => "ipv4.dst",
        50 => "icmp.type",
        51 => "icmp.code",
        52 => "icmp.checksum",
        53 => "icmp.identifier",
        54 => "icmp.sequence",
        55 => "icmp.payload",
        56 => "icmp.quote",
        _ => "unknown",
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static TEST_GUARD: Mutex<()> = Mutex::new(());

    fn direct_hosts(seed: u64) -> (Network, u64, u64) {
        let mut network = Network::new(seed, 1024).unwrap();
        let left = network.device(DeviceRole::Host).unwrap();
        let right = network.device(DeviceRole::Host).unwrap();
        let left_port = network.port(left).unwrap();
        let right_port = network.port(right).unwrap();
        network
            .configure_port(left_port, 0x020000000001, 0x0a000001, 24, true)
            .unwrap();
        network
            .configure_port(right_port, 0x020000000002, 0x0a000002, 24, true)
            .unwrap();
        network
            .link(left_port, right_port, 100_000_000, 1_000)
            .unwrap();
        (network, left_port, right_port)
    }

    fn drain(network: &mut Network) {
        loop {
            let progress = network.run_until(5_000_000_000, 512).unwrap();
            if progress.reason == 1 {
                break;
            }
        }
    }

    fn observable(network: &Network) -> (Summary, Vec<Packet>, Vec<Trace>) {
        let summary = network.summary().unwrap();
        let packets = (1..=summary.packet_versions)
            .map(|id| network.packet(id).unwrap())
            .collect();
        let mut trace = Vec::new();
        loop {
            let after = trace.last().map_or(0, |row: &Trace| row.ordinal);
            let batch = network.trace(after, 256).unwrap();
            if batch.is_empty() {
                break;
            }
            trace.extend(batch);
        }
        (summary, packets, trace)
    }
    #[test]
    fn layouts_ownership_and_invalid_input_copy() {
        let _guard = TEST_GUARD.lock().unwrap();
        assert_eq!(std::mem::size_of::<RawPacket>(), 1640);
        assert_eq!(std::mem::offset_of!(RawPacket, bytes), 40);
        assert_eq!(std::mem::size_of::<Trace>(), 96);
        assert_eq!(std::mem::size_of::<Transmission>(), 72);
        assert_eq!(std::mem::size_of::<Summary>(), 80);
        assert_eq!(std::mem::size_of::<Field>(), 24);
        assert_eq!(std::mem::size_of::<Noise>(), 40);
        assert_eq!(std::mem::size_of::<CheckpointInfo>(), 56);
        let mut net = Network::new(0, 1024).unwrap();
        let dev = net.device(DeviceRole::Host).unwrap();
        let port = net.port(dev).unwrap();
        let mut input = [0x45, 0];
        net.inject(port, 1, &input).unwrap();
        input.fill(0xff);
        let packet = net.packet(1).unwrap();
        assert_eq!(net.summary().unwrap().reason, 5);
        assert_eq!(net.fields(1), Err(Error::InvalidPacket));
        assert_eq!(net.device(DeviceRole::Host), Err(Error::State));
        net.close().unwrap();
        assert_eq!(packet.bytes, [0x45, 0]);
    }
    #[test]
    fn raw_registry_rejects_null_and_stale_handles() {
        let _guard = TEST_GUARD.lock().unwrap();
        let _guard = api_lock();
        // SAFETY: nulls are documented error inputs; all other outputs are valid.
        unsafe {
            let mut handle = 0;
            assert_eq!(wb_net_create(1, 0, 0, std::ptr::null_mut()), 1);
            let mut extra = 99;
            assert_eq!(wb_net_create(99, 0, 0, &mut extra), 2);
            assert_eq!(extra, 0);
            // Exhaustion is checked in Zig; Rust tests can own other live networks.
            assert_eq!(wb_net_create(1, 0, 0, &mut handle), 0);
            let old = handle;
            assert_eq!(wb_net_destroy(old), 0);
            assert_eq!(wb_net_create(1, 0, 0, &mut handle), 0);
            assert_ne!(old, handle);
            assert_eq!(wb_net_destroy(old), 3);
            assert_eq!(wb_net_destroy(handle & !(3_u64 << 62)), 3);
            assert_eq!(wb_net_destroy(handle), 0);
        }
    }

    #[test]
    fn checkpoint_restore_and_fork_preserve_active_arp_state_and_replay() {
        let _guard = TEST_GUARD.lock().unwrap();
        let (mut home, source, _) = direct_hosts(0x51);
        home.ping(source, 0x0a000002, 64).unwrap();
        let checkpoint = home.checkpoint().unwrap();
        assert!(home.summary().unwrap().pending_events > 0);

        let mut branch = checkpoint.fork().unwrap();
        drain(&mut home);
        drain(&mut branch);
        let expected = observable(&home);
        assert_eq!(expected, observable(&branch));

        home.restore(&checkpoint).unwrap();
        assert_eq!(
            home.summary().unwrap().pending_events,
            checkpoint.fork().unwrap().summary().unwrap().pending_events
        );
        drain(&mut home);
        assert_eq!(expected, observable(&home));
    }

    #[test]
    fn seeded_jitter_is_replayed_and_loss_is_recorded_as_input_noise() {
        let _guard = TEST_GUARD.lock().unwrap();
        let (mut home, source, _) = direct_hosts(0x1234);
        home.set_noise(0, 10_000).unwrap();
        home.ping(source, 0x0a000002, 64).unwrap();
        let checkpoint = home.checkpoint().unwrap();
        let mut branch = checkpoint.fork().unwrap();
        drain(&mut home);
        drain(&mut branch);
        assert_eq!(observable(&home), observable(&branch));
        assert_eq!(home.noise().unwrap(), branch.noise().unwrap());
        assert!(home.noise().unwrap().draws > 0);

        let (mut lossy, source, _) = direct_hosts(0x1234);
        lossy.set_noise(1_000_000, 0).unwrap();
        lossy.ping(source, 0x0a000002, 64).unwrap();
        drain(&mut lossy);
        let noise = lossy.noise().unwrap();
        assert!(noise.dropped > 0);
        assert_eq!(lossy.summary().unwrap().outcome, 4);
        assert_eq!(lossy.summary().unwrap().pending_events, 0);
    }

    #[test]
    fn impairment_configuration_is_frozen_after_workload_starts() {
        let _guard = TEST_GUARD.lock().unwrap();
        let (mut network, source, _) = direct_hosts(7);
        network.ping(source, 0x0a000002, 64).unwrap();
        assert_eq!(network.set_noise(1, 1), Err(Error::State));
    }

    #[test]
    fn checkpoint_before_workload_survives_source_close_and_branches_diverge() {
        let _guard = TEST_GUARD.lock().unwrap();
        let (source, port, _) = direct_hosts(9);
        let checkpoint = source.checkpoint().unwrap();
        let before = checkpoint.info().unwrap();
        assert_eq!((before.input_kind, before.pending_events), (0, 0));
        source.close().unwrap();
        let mut healthy = checkpoint.fork().unwrap();
        let mut lossy = checkpoint.fork().unwrap();
        lossy.set_noise(1_000_000, 0).unwrap();
        healthy.ping(port, 0x0a000002, 64).unwrap();
        lossy.ping(port, 0x0a000002, 64).unwrap();
        drain(&mut healthy);
        drain(&mut lossy);
        assert_eq!(healthy.summary().unwrap().outcome, 1);
        assert_eq!(lossy.summary().unwrap().outcome, 4);
        assert_eq!(checkpoint.info().unwrap(), before);
        lossy.restore(&checkpoint).unwrap();
        assert_eq!(lossy.noise().unwrap(), Noise::default());
        assert_eq!(lossy.summary().unwrap().trace_total, 0);
        // All restored state is editable again because this checkpoint precedes input.
        lossy.ping(port, 0x0a000002, 64).unwrap();
        drop(checkpoint);
        drain(&mut lossy);
        assert_eq!(observable(&healthy), observable(&lossy));
    }
}
