const std = @import("std");
pub const rt = @import("runtime.zig");
pub const c = @import("codec.zig");
pub const Error = rt.Error || error{ State, OutOfMemory };
pub const packet_limit = 128;
pub const tx_limit = 256;
pub const trace_limit = 10_000;
pub const Packet = extern struct {
    version_id: u64 = 0,
    packet_id: u64 = 0,
    parent_version: u64 = 0,
    cause_event: u64 = 0,
    layer: u32 = 0,
    len: u32 = 0,
    bytes: [c.max_bytes]u8 = @splat(0),
    pub fn data(self: *const Packet) []const u8 {
        return self.bytes[0..self.len];
    }
};
pub const Trace = extern struct {
    ordinal: u64 = 0,
    event_id: u64 = 0,
    time_ns: u64 = 0,
    cause_id: u64 = 0,
    entity: u64 = 0,
    packet_id: u64 = 0,
    version: u64 = 0,
    prior: u64 = 0,
    transmission: u64 = 0,
    detail_a: u64 = 0,
    detail_b: u64 = 0,
    kind: u32 = 0,
    reason: u32 = 0,
};
pub const Tx = extern struct {
    id: u64 = 0,
    version: u64 = 0,
    from: u64 = 0,
    to: u64 = 0,
    link: u64 = 0,
    start_ns: u64 = 0,
    arrival_ns: u64 = 0,
    event_id: u64 = 0,
    cause_id: u64 = 0,
};
pub const Summary = extern struct {
    now_ns: u64,
    executed: u64,
    trace_total: u64,
    trace_dropped: u64,
    packet_versions: u64,
    transmissions: u64,
    failure_entity: u64,
    pending_events: u32,
    pending_arp: u32,
    outcome: u32,
    reason: u32,
    failed: u32,
    reserved: u32 = 0,
};
pub const Port = struct { id: u64 = 0, mac: u64 = 0, ip: u32 = 0, prefix: u32 = 0, enabled: bool = true };
const Link = struct { id: u64 = 0, bitrate: u64 = 0, propagation: u64 = 0 };
const Route = struct { device: u64 = 0, port: u64 = 0, network: u32 = 0, prefix: u32 = 0, via: u32 = 0 };
const Alias = struct { device: u64 = 0, ip: u32 = 0 };
const ArpEntry = struct { port: u64 = 0, ip: u32 = 0, mac: u64 = 0 };
const MacEntry = struct { device: u64 = 0, mac: u64 = 0, port: u64 = 0 };
const Pending = struct {
    active: bool = false,
    port: u64 = 0,
    ip: u32 = 0,
    attempts: u32 = 0,
    timer: u64 = 0,
    count: usize = 0,
    versions: [8]u64 = @splat(0),
};
const Pick = struct { port: u64, next_hop: u32, prefix: u32, connected: bool };
pub fn mask(prefix: u32) u32 {
    return if (prefix == 0) 0 else @as(u32, std.math.maxInt(u32)) << @as(u5, @intCast(32 - prefix));
}
pub fn unicastMac(mac: u64) bool {
    return mac > 0 and mac < (1 << 48) and mac & 0x010000000000 == 0;
}
fn ordinaryIp(ip: u32) bool {
    return ip != 0 and ip >> 24 != 0 and ip >> 24 != 127 and ip < 0xe0000000;
}

pub const Network = struct {
    runtime: rt.Runtime,
    ports: [128]Port,
    links: [128]Link,
    next_free: [128]u64,
    routes: [32]Route,
    aliases: [16]Alias,
    arp_cache: [64]ArpEntry,
    mac_cache: [64]MacEntry,
    pending: [16]Pending,
    packets: [packet_limit]Packet,
    packet_count: usize,
    next_packet: u64,
    txs: [tx_limit]Tx,
    tx_count: usize,
    trace: [trace_limit]Trace,
    retention: usize,
    trace_total: u64,
    current: rt.Event,
    started: bool,
    failed: bool,
    outcome: u32,
    why: c.Reason,
    failure_entity: u64,
    source_device: u64,
    source_ip: u32,
    destination: u32,
    request_version: u64,
    next_ip_id: u16,

    pub fn init(self: *Network, seed: u64, retention: usize) Error!void {
        if (retention > trace_limit) return error.InvalidArgument;
        // Initialize in place; no large Network value or caller-owned allocator crosses ABI.
        @memset(std.mem.asBytes(self), 0);
        self.runtime = try rt.Runtime.init(seed, 0);
        self.retention = retention;
        self.next_packet = 1;
        self.next_ip_id = 1;
        for (&self.ports) |*port| port.enabled = true;
    }
    fn config(self: *Network) Error!void {
        if (self.started or self.failed) return error.State;
    }
    fn portIndex(self: *Network, id: u64) Error!usize {
        for (self.ports, 0..) |port, i| if (port.id == id and id != 0) return i;
        return error.NotFound;
    }
    fn owner(self: *Network, port: u64) u64 {
        return self.runtime.topology.find(port).?.owner;
    }
    pub fn add(self: *Network, kind: rt.graph.Kind, role: u32, a: u64, b: u64) Error!u64 {
        try self.config();
        const id = try self.runtime.topology.add(kind, role, a, b);
        if (kind == .port) {
            for (&self.ports) |*port| if (port.id == 0) {
                port.id = id;
                break;
            };
        }
        return id;
    }
    pub fn setPort(self: *Network, id: u64, mac: u64, ip: u32, prefix: u32, enabled: bool) Error!void {
        try self.config();
        const index = try self.portIndex(id);
        if (prefix > 32 or (mac != 0 and !unicastMac(mac)) or (ip != 0 and (!ordinaryIp(ip) or mac == 0))) return error.InvalidArgument;
        // MAC/IP uniqueness avoids ambiguous fixture identities.
        for (self.ports) |p| if (p.id != id and p.id != 0 and ((mac != 0 and p.mac == mac) or (ip != 0 and p.ip == ip))) return error.Conflict;
        for (self.aliases) |a| if (ip != 0 and a.device != 0 and a.ip == ip) return error.Conflict;
        self.ports[index] = .{ .id = id, .mac = mac, .ip = ip, .prefix = prefix, .enabled = enabled };
    }
    pub fn setLink(self: *Network, id: u64, bitrate: u64, propagation: u64) Error!void {
        try self.config();
        const entity = self.runtime.topology.find(id) orelse return error.NotFound;
        if (entity.kind != 3 or bitrate == 0) return error.InvalidArgument;
        for (&self.links) |*link| if (link.id == id or link.id == 0) {
            link.* = .{ .id = id, .bitrate = bitrate, .propagation = propagation };
            return;
        };
        return error.Capacity;
    }
    pub fn addAlias(self: *Network, device: u64, ip: u32) Error!void {
        try self.config();
        const entity = self.runtime.topology.find(device) orelse return error.NotFound;
        if (entity.kind != 1 or !ordinaryIp(ip)) return error.InvalidArgument;
        for (self.ports) |p| if (p.ip == ip) return error.Conflict;
        for (self.aliases) |a| if (a.device != 0 and a.ip == ip) return error.Conflict;
        for (&self.aliases) |*a| if (a.device == 0) {
            a.* = .{ .device = device, .ip = ip };
            return;
        };
        return error.Capacity;
    }
    pub fn addRoute(self: *Network, device: u64, port: u64, network: u32, prefix: u32, via: u32) Error!void {
        try self.config();
        const p = self.ports[try self.portIndex(port)];
        if (self.owner(port) != device or p.ip == 0 or prefix > 32) return error.InvalidArgument;
        if (network & mask(prefix) != network or (via != 0 and (!ordinaryIp(via) or via & mask(p.prefix) != p.ip & mask(p.prefix)))) return error.InvalidArgument;
        for (self.routes) |r| if (r.device == device and r.network == network and r.prefix == prefix) return error.Conflict;
        for (&self.routes) |*r| if (r.device == 0) {
            r.* = .{ .device = device, .port = port, .network = network, .prefix = prefix, .via = via };
            return;
        };
        return error.Capacity;
    }
    fn validate(self: *Network) Error!void {
        // Configuration is staged: setPort may invalidate an earlier addRoute.
        // Recheck cross-object constraints before ping/inject commits started=true.
        // Like validating a transaction, failure here must not consume IDs or timers.
        for (self.routes) |r| {
            if (r.device == 0) continue;
            const p = self.ports[try self.portIndex(r.port)];
            if (p.ip == 0 or (r.via != 0 and r.via & mask(p.prefix) != p.ip & mask(p.prefix))) return error.InvalidArgument;
        }
        for (self.runtime.topology.entries) |entity| {
            if (entity.kind != 3) continue;
            var found = false;
            for (self.links) |link| if (link.id == entity.id) {
                found = true;
                break;
            };
            if (!found) return error.InvalidArgument;
        }
    }
    pub fn packet(self: *Network, id: u64) Error!*const Packet {
        if (id == 0 or id > self.packet_count) return error.NotFound;
        return &self.packets[@intCast(id - 1)];
    }
    fn store(self: *Network, layer: u32, bytes: []const u8, parent: u64, same_packet: bool) Error!u64 {
        if (bytes.len > c.max_bytes or self.packet_count == packet_limit) return error.Capacity;
        const packet_id = if (same_packet) (try self.packet(parent)).packet_id else self.next_packet;
        if (!same_packet) self.next_packet += 1; // bounded by packet_limit
        const id = self.packet_count + 1;
        const out = &self.packets[self.packet_count];
        out.* = .{ .version_id = id, .packet_id = packet_id, .parent_version = parent, .cause_event = self.current.id, .layer = layer, .len = @intCast(bytes.len) };
        @memcpy(out.bytes[0..bytes.len], bytes);
        self.packet_count += 1;
        return id;
    }
    fn emit(self: *Network, input: Trace) void {
        self.trace_total += 1; // bounded packet/tx/event work in this model
        if (self.retention == 0) return;
        var row = input;
        row.ordinal = self.trace_total;
        row.event_id = self.current.id;
        row.time_ns = self.runtime.now;
        row.cause_id = self.current.cause_id;
        if (row.version > 0) row.packet_id = self.packets[@intCast(row.version - 1)].packet_id;
        self.trace[@intCast((row.ordinal - 1) % self.retention)] = row;
    }
    pub fn readTrace(self: *Network, after: u64, out: []Trace) usize {
        const count = @min(self.trace_total, self.retention);
        if (after >= self.trace_total) return 0;
        // Seek directly into the ring. Reading in batches must not rescan old rows.
        var ordinal = @max(self.trace_total - count + 1, after + 1);
        var n: usize = 0;
        while (ordinal <= self.trace_total and n < out.len) : (ordinal += 1) {
            if (ordinal <= after) continue;
            out[n] = self.trace[@intCast((ordinal - 1) % self.retention)];
            n += 1;
        }
        return n;
    }
    fn reject(self: *Network, device: u64, version: u64, reason: c.Reason) void {
        self.emit(.{ .kind = 17, .entity = device, .version = version, .reason = @backingInt(reason) });
        self.outcome = if (c.unsupported(reason)) 6 else 5;
        self.why = reason;
        self.failure_entity = device;
    }
    fn local(self: *Network, device: u64, ip: u32) bool {
        for (self.ports) |p| if (p.id != 0 and p.enabled and p.ip == ip and self.owner(p.id) == device) return true;
        for (self.aliases) |a| if (a.device == device and a.ip == ip) return true;
        return false;
    }
    fn broadcastIp(self: *Network, ip: u32) bool {
        if (ip == 0xffffffff or ip >= 0xe0000000) return true;
        for (self.ports) |p| if (p.id != 0 and p.ip != 0 and p.prefix < 31 and ip == (p.ip | ~mask(p.prefix))) return true;
        return false;
    }
    pub fn route(self: *Network, device: u64, destination: u32) ?Pick {
        var best: ?Pick = null;
        for (self.ports) |p| {
            if (p.id == 0 or p.ip == 0 or !p.enabled or self.owner(p.id) != device) continue;
            if (destination & mask(p.prefix) == p.ip & mask(p.prefix) and (best == null or p.prefix > best.?.prefix))
                best = .{ .port = p.id, .next_hop = destination, .prefix = p.prefix, .connected = true };
        }
        for (self.routes) |r| {
            if (r.device != device or !self.ports[self.portIndex(r.port) catch unreachable].enabled) continue;
            if (destination & mask(r.prefix) == r.network and (best == null or r.prefix > best.?.prefix))
                best = .{ .port = r.port, .next_hop = if (r.via == 0) destination else r.via, .prefix = r.prefix, .connected = false };
        }
        return best;
    }
    fn transmit(self: *Network, from: u64, version: u64) Error!void {
        const pi = try self.portIndex(from);
        if (!self.ports[pi].enabled) {
            self.emit(.{ .kind = 17, .entity = from, .version = version, .reason = 103 });
            return;
        }
        var peer: u64 = 0;
        var link_id: u64 = 0;
        for (self.runtime.topology.entries) |e| {
            if (e.kind != 3) continue;
            if (e.a == from) {
                peer = e.b;
                link_id = e.id;
                break;
            }
            if (e.b == from) {
                peer = e.a;
                link_id = e.id;
                break;
            }
        }
        if (peer == 0) {
            self.emit(.{ .kind = 17, .entity = from, .version = version, .reason = 103 });
            return;
        }
        var config_link: Link = .{};
        for (self.links) |l| if (l.id == link_id) {
            config_link = l;
            break;
        };
        if (config_link.bitrate == 0) return error.InvalidArgument;
        if (self.tx_count == tx_limit) return error.Capacity;
        const bytes = (try self.packet(version)).data();
        const start = @max(self.runtime.now, self.next_free[pi]);
        const end = std.math.add(u64, start, try rt.serializationDelay(bytes.len, config_link.bitrate)) catch return error.Overflow;
        const arrival = std.math.add(u64, end, config_link.propagation) catch return error.Overflow;
        const tx_id = self.tx_count + 1;
        const event_id = self.runtime.next_event_id;
        _ = try self.runtime.scheduleDomain(peer, arrival, 100, tx_id, self.current.id);
        self.txs[self.tx_count] = .{ .id = tx_id, .version = version, .from = from, .to = peer, .link = link_id, .start_ns = start, .arrival_ns = arrival, .event_id = event_id, .cause_id = self.current.id };
        self.tx_count += 1;
        self.next_free[pi] = end;
        self.emit(.{ .kind = 10, .entity = from, .version = version, .transmission = tx_id, .detail_a = peer, .detail_b = arrival });
    }
    fn encapsulate(self: *Network, port: u64, version: u64, dst_mac: u64) Error!void {
        const p = self.ports[try self.portIndex(port)];
        const bytes = (try self.packet(version)).data();
        var frame_bytes: [c.max_bytes]u8 = undefined;
        const len = c.frame(&frame_bytes, p.mac, dst_mac, 0x0800, bytes);
        const wrapped = try self.store(2, frame_bytes[0..len], version, true);
        self.emit(.{ .kind = 1, .entity = port, .version = wrapped, .prior = version });
        try self.transmit(port, wrapped);
    }
    fn arpFrame(self: *Network, port: u64, message: c.Arp, parent: u64) Error!void {
        var payload: [28]u8 = undefined;
        c.arpBytes(&payload, message);
        var bytes: [60]u8 = undefined;
        _ = c.frame(&bytes, message.sha, if (message.op == 1) 0xffffffffffff else message.tha, 0x0806, &payload);
        const version = try self.store(2, &bytes, parent, false);
        self.emit(.{ .kind = if (message.op == 1) 4 else 6, .entity = port, .version = version, .prior = parent, .detail_a = message.tpa });
        try self.transmit(port, version);
    }
    fn arpAttempt(self: *Network, index: usize) Error!void {
        const pending = &self.pending[index];
        const p = self.ports[try self.portIndex(pending.port)];
        pending.attempts += 1;
        try self.arpFrame(p.id, .{ .op = 1, .sha = p.mac, .spa = p.ip, .tha = 0, .tpa = pending.ip }, pending.versions[0]);
        const due = std.math.add(u64, self.runtime.now, 1_000_000_000) catch return error.Overflow;
        pending.timer = try self.runtime.scheduleDomain(p.id, due, 101, index + 1, self.current.id);
    }
    fn resolve(self: *Network, port: u64, next_hop: u32, version: u64) Error!void {
        self.emit(.{ .kind = 3, .entity = port, .version = version, .detail_a = next_hop });
        for (self.arp_cache) |a| if (a.port == port and a.ip == next_hop) {
            self.emit(.{ .kind = 20, .entity = port, .version = version, .detail_a = next_hop, .detail_b = a.mac });
            try self.encapsulate(port, version, a.mac);
            return;
        };
        for (&self.pending) |*p| if (p.active and p.port == port and p.ip == next_hop) {
            if (p.count == p.versions.len) return error.Capacity;
            p.versions[p.count] = version;
            p.count += 1;
            self.emit(.{ .kind = 19, .entity = port, .version = version, .detail_a = next_hop });
            return;
        };
        for (&self.pending, 0..) |*p, i| if (!p.active) {
            p.* = .{ .active = true, .port = port, .ip = next_hop, .count = 1 };
            p.versions[0] = version;
            self.emit(.{ .kind = 19, .entity = port, .version = version, .detail_a = next_hop });
            try self.arpAttempt(i);
            return;
        };
        return error.Capacity;
    }
    fn sendIp(self: *Network, device: u64, version: u64) Error!void {
        const ip = c.ipv4((try self.packet(version)).data()) catch unreachable;
        const chosen = self.route(device, ip.dst) orelse {
            self.emit(.{ .kind = 17, .entity = device, .version = version, .reason = 100 });
            if (self.outcome == 0) {
                self.outcome = 2;
                self.why = .no_route;
                self.failure_entity = device;
            }
            return;
        };
        self.emit(.{ .kind = 2, .entity = device, .version = version, .detail_a = chosen.port, .detail_b = chosen.next_hop, .reason = if (chosen.connected) 200 else 201 });
        try self.resolve(chosen.port, chosen.next_hop, version);
    }
    fn receiveArp(self: *Network, port: u64, version: u64, eth: c.Ethernet) Error!void {
        const data = (try self.packet(version)).data();
        const a = c.arp(data[14..]) catch |err| {
            self.reject(self.owner(port), version, c.reason(err));
            return;
        };
        const p = self.ports[try self.portIndex(port)];
        if (a.sha != eth.src or !unicastMac(a.sha) or !ordinaryIp(a.spa)) {
            self.reject(self.owner(port), version, .invalid_arp);
            return;
        }
        if (p.ip == 0 or a.tpa != p.ip or (a.op == 2 and a.tha != p.mac)) return;
        var cache: ?*ArpEntry = null;
        for (&self.arp_cache) |*entry| if (entry.port == port and entry.ip == a.spa) {
            cache = entry;
            break;
        };
        if (cache == null) for (&self.arp_cache) |*entry| if (entry.port == 0) {
            cache = entry;
            break;
        };
        const entry = cache orelse return error.Capacity;
        entry.* = .{ .port = port, .ip = a.spa, .mac = a.sha };
        self.emit(.{ .kind = 5, .entity = port, .version = version, .detail_a = a.spa, .detail_b = a.sha });
        if (a.op == 1) try self.arpFrame(port, .{ .op = 2, .sha = p.mac, .spa = p.ip, .tha = a.sha, .tpa = a.spa }, version);
        for (&self.pending) |*pending| if (pending.active and pending.port == port and pending.ip == a.spa) {
            if (pending.timer != 0) try self.runtime.cancel(pending.timer);
            pending.active = false;
            for (pending.versions[0..pending.count]) |buffered| try self.encapsulate(port, buffered, a.sha);
            pending.count = 0;
            pending.timer = 0;
        };
    }
    fn switchFrame(self: *Network, port: u64, version: u64, eth: c.Ethernet) Error!void {
        const device = self.owner(port);
        if (!unicastMac(eth.src)) {
            self.reject(device, version, .invalid_ethernet);
            return;
        }
        var cache: ?*MacEntry = null;
        for (&self.mac_cache) |*entry| if (entry.device == device and entry.mac == eth.src) {
            cache = entry;
            break;
        };
        if (cache == null) for (&self.mac_cache) |*entry| if (entry.device == 0) {
            cache = entry;
            break;
        };
        const source = cache orelse return error.Capacity;
        if (source.port != port) self.emit(.{ .kind = 7, .entity = device, .version = version, .detail_a = eth.src, .detail_b = port });
        source.* = .{ .device = device, .mac = eth.src, .port = port };
        if (unicastMac(eth.dst)) for (self.mac_cache) |entry| {
            if (entry.device == device and entry.mac == eth.dst) {
                self.emit(.{ .kind = 9, .entity = device, .version = version, .detail_a = entry.port });
                if (entry.port != port) try self.transmit(entry.port, version);
                return;
            }
        };
        self.emit(.{ .kind = 8, .entity = device, .version = version });
        for (self.ports) |p| if (p.id != 0 and p.id != port and p.enabled and self.owner(p.id) == device) try self.transmit(p.id, version);
    }
    fn newIp(self: *Network, src: u32, dst: u32, ttl: u8, payload: []const u8, parent: u64) Error!u64 {
        if (payload.len > 1480) return error.Capacity;
        var bytes: [1500]u8 = undefined;
        c.ipHeader(&bytes, payload.len, src, dst, ttl, self.next_ip_id);
        self.next_ip_id +%= 1;
        @memcpy(bytes[20..][0..payload.len], payload);
        return self.store(1, bytes[0 .. 20 + payload.len], parent, false);
    }
    fn errorIcmp(self: *Network, device: u64, ingress: u64, version: u64, typ: u8, broadcast: bool) Error!void {
        const bytes = (try self.packet(version)).data();
        const ip = c.ipv4(bytes) catch unreachable;
        const error_msg = ip.protocol == 1 and ip.total > 20 and (bytes[20] == 3 or bytes[20] == 4 or bytes[20] == 5 or bytes[20] == 11 or bytes[20] == 12);
        if (broadcast or self.broadcastIp(ip.dst) or !ordinaryIp(ip.src) or self.broadcastIp(ip.src) or error_msg) {
            self.emit(.{ .kind = 17, .entity = device, .version = version, .reason = 106 });
            if (self.outcome == 0) {
                self.outcome = 7;
                self.why = .icmp_error_suppressed;
            }
            return;
        }
        var msg: [36]u8 = @splat(0);
        msg[0] = typ;
        @memcpy(msg[8..36], bytes[0..28]);
        c.write(msg[2..4], c.checksum(&msg));
        const src = self.ports[try self.portIndex(ingress)].ip;
        const reply = try self.newIp(src, ip.src, 64, &msg, version);
        self.emit(.{ .kind = 15, .entity = device, .version = reply, .prior = version, .detail_a = typ });
        try self.sendIp(device, reply);
    }
    fn receiveIp(self: *Network, port: u64, version: u64, broadcast: bool) Error!void {
        const device = self.owner(port);
        const bytes = (try self.packet(version)).data();
        const ip = c.ipv4(bytes) catch |err| {
            self.reject(device, version, c.reason(err));
            return;
        };
        if (ip.protocol != 1) {
            self.reject(device, version, .unsupported_protocol);
            return;
        }
        const msg = bytes[20..ip.total];
        c.icmp(msg) catch |err| {
            self.reject(device, version, c.reason(err));
            return;
        };
        if (self.broadcastIp(ip.dst) or self.broadcastIp(ip.src) or !ordinaryIp(ip.dst) or !ordinaryIp(ip.src)) {
            self.reject(device, version, .unsupported_address);
            return;
        }
        if (self.local(device, ip.dst)) {
            if (msg[0] == 8) {
                if (broadcast) {
                    self.emit(.{ .kind = 17, .entity = device, .version = version, .reason = 106 });
                    return;
                }
                var reply_msg: [1480]u8 = undefined;
                @memcpy(reply_msg[0..msg.len], msg);
                reply_msg[0] = 0;
                reply_msg[2] = 0;
                reply_msg[3] = 0;
                c.write(reply_msg[2..4], c.checksum(reply_msg[0..msg.len]));
                const reply = try self.newIp(ip.dst, ip.src, 64, reply_msg[0..msg.len], version);
                self.emit(.{ .kind = 14, .entity = device, .version = reply, .prior = version });
                try self.sendIp(device, reply);
            } else {
                var matches = false;
                if (self.request_version != 0 and device == self.source_device and ip.dst == self.source_ip) {
                    const original = (try self.packet(self.request_version)).data();
                    if (msg[0] == 0) {
                        matches = ip.src == self.destination and msg.len == original.len - 20 and std.mem.eql(u8, msg[4..], original[24..]);
                    } else {
                        const quote = msg[8..36];
                        matches = c.read(quote[12..16]) == self.source_ip and c.read(quote[16..20]) == self.destination and quote[9] == 1 and std.mem.eql(u8, quote[24..28], original[24..28]);
                    }
                }
                if (matches) {
                    self.outcome = if (msg[0] == 0) 1 else if (msg[0] == 3) 2 else 3;
                    self.emit(.{ .kind = 16, .entity = device, .version = version, .detail_a = msg[0], .detail_b = msg[1] });
                } else self.emit(.{ .kind = 17, .entity = device, .version = version, .reason = 107 });
            }
            return;
        }
        if (self.runtime.topology.find(device).?.role != 3) {
            self.emit(.{ .kind = 17, .entity = device, .version = version, .reason = 105 });
            return;
        }
        if (ip.ttl <= 1) {
            self.why = .ttl_expired;
            self.failure_entity = device;
            self.emit(.{ .kind = 17, .entity = device, .version = version, .reason = 101 });
            try self.errorIcmp(device, port, version, 11, broadcast);
            return;
        }
        const chosen = self.route(device, ip.dst) orelse {
            self.why = .no_route;
            self.failure_entity = device;
            self.emit(.{ .kind = 17, .entity = device, .version = version, .reason = 100 });
            try self.errorIcmp(device, port, version, 3, broadcast);
            return;
        };
        self.emit(.{ .kind = 2, .entity = device, .version = version, .detail_a = chosen.port, .detail_b = chosen.next_hop, .reason = if (chosen.connected) 200 else 201 });
        var changed: [1500]u8 = undefined;
        @memcpy(changed[0..ip.total], bytes[0..ip.total]);
        c.decrement(changed[0..ip.total]) catch unreachable;
        const forwarded = try self.store(1, changed[0..ip.total], version, true);
        self.emit(.{ .kind = 13, .entity = device, .version = forwarded, .prior = version, .detail_a = ip.ttl, .detail_b = ip.ttl - 1 });
        try self.resolve(chosen.port, chosen.next_hop, forwarded);
    }
    fn receive(self: *Network, port: u64, version: u64, tx: u64) Error!void {
        self.emit(.{ .kind = 11, .entity = port, .version = version, .transmission = tx });
        const p = self.ports[try self.portIndex(port)];
        if (!p.enabled) {
            self.emit(.{ .kind = 17, .entity = port, .version = version, .reason = 103 });
            return;
        }
        const bytes = (try self.packet(version)).data();
        const eth = c.ethernet(bytes) catch |err| {
            self.reject(self.owner(port), version, c.reason(err));
            return;
        };
        if (self.runtime.topology.find(self.owner(port)).?.role == 2) {
            try self.switchFrame(port, version, eth);
            return;
        }
        if (eth.dst != p.mac and eth.dst != 0xffffffffffff) {
            self.emit(.{ .kind = 17, .entity = port, .version = version, .reason = 104 });
            return;
        }
        if (eth.kind == 0x0806) return self.receiveArp(port, version, eth);
        const payload = bytes[14..];
        const ip = c.ipv4(payload) catch |err| {
            self.reject(self.owner(port), version, c.reason(err));
            return;
        };
        const decap = try self.store(1, payload[0..ip.total], version, true);
        self.emit(.{ .kind = 12, .entity = port, .version = decap, .prior = version });
        try self.receiveIp(port, decap, eth.dst == 0xffffffffffff);
    }
    pub fn ping(self: *Network, port: u64, destination: u32, ttl: u32) Error!void {
        try self.config();
        const p = self.ports[try self.portIndex(port)];
        if (!p.enabled or p.ip == 0 or ttl == 0 or ttl > 255 or !ordinaryIp(destination) or self.broadcastIp(destination)) return error.InvalidArgument;
        try self.validate();
        self.started = true;
        self.source_device = self.owner(port);
        self.source_ip = p.ip;
        self.destination = destination;
        var msg = [_]u8{ 8, 0, 0, 0, 0x12, 0x34, 0, 1 };
        c.write(msg[2..4], c.checksum(&msg));
        const version = try self.newIp(p.ip, destination, @intCast(ttl), &msg, 0);
        self.request_version = version;
        self.emit(.{ .kind = 1, .entity = self.source_device, .version = version });
        if (self.local(self.source_device, destination)) try self.receiveIp(port, version, false) else try self.sendIp(self.source_device, version);
    }
    pub fn inject(self: *Network, port: u64, layer: u32, bytes: []const u8) Error!void {
        try self.config();
        _ = try self.portIndex(port);
        if (layer != 1 and layer != 2 or bytes.len > c.max_bytes) return error.InvalidArgument;
        try self.validate();
        self.started = true;
        const version = try self.store(layer, bytes, 0, false);
        self.emit(.{ .kind = 1, .entity = port, .version = version });
        if (layer == 1) try self.receiveIp(port, version, false) else try self.receive(port, version, 0);
    }
    fn dispatch(self: *Network, event: rt.Event) Error!void {
        if (event.action == 100) {
            const tx = self.txs[@intCast(event.amount - 1)];
            try self.receive(tx.to, tx.version, tx.id);
        } else {
            const index: usize = @intCast(event.amount - 1);
            const pending = &self.pending[index];
            pending.timer = 0;
            if (!pending.active) return error.State;
            if (pending.attempts < 3) return self.arpAttempt(index);
            for (pending.versions[0..pending.count]) |version| self.emit(.{ .kind = 18, .entity = self.owner(pending.port), .version = version, .detail_a = pending.ip, .detail_b = pending.attempts, .reason = 102 });
            pending.active = false;
            pending.count = 0;
            if (self.outcome == 0) {
                self.outcome = 4;
                self.why = .arp_timeout;
                self.failure_entity = self.owner(pending.port);
            }
        }
    }
    pub fn run(self: *Network, horizon: u64, budget: u64, progress: *rt.Progress) Error!void {
        progress.* = .{ .now_ns = self.runtime.now, .reason = 4 };
        if (!self.started or self.failed) return error.State;
        if (horizon < self.runtime.now) return error.PastTime;
        while (self.runtime.queue.len > 0 and self.runtime.queue.items[0].time_ns <= horizon) {
            if (progress.processed == budget) {
                progress.reason = 2;
                return;
            }
            self.current = try self.runtime.beginDomain();
            self.dispatch(self.current) catch |err| {
                self.failed = true;
                progress.now_ns = self.runtime.now;
                return err;
            };
            progress.processed += 1;
            progress.now_ns = self.runtime.now;
        }
        self.runtime.now = horizon;
        progress.now_ns = horizon;
        progress.reason = 1;
    }
    pub fn summary(self: *Network) Summary {
        var pending_count: u32 = 0;
        for (self.pending) |p| if (p.active) {
            pending_count += 1;
        };
        return .{
            .now_ns = self.runtime.now,
            .executed = self.runtime.executed,
            .trace_total = self.trace_total,
            .trace_dropped = self.trace_total - @min(self.retention, self.trace_total),
            .packet_versions = self.packet_count,
            .transmissions = self.tx_count,
            .failure_entity = self.failure_entity,
            .pending_events = @intCast(self.runtime.queue.len),
            .pending_arp = pending_count,
            .outcome = self.outcome,
            .reason = @backingInt(self.why),
            .failed = @intFromBool(self.failed),
        };
    }
};

test "trace retention wraps without trimming scheduler or aliasing packet versions" {
    const net = try std.testing.allocator.create(Network);
    defer std.testing.allocator.destroy(net);
    try net.init(1, trace_limit);
    const version = try net.store(1, &.{ 0x45, 0x00 }, 0, false);
    // Synthetic journal stress: this tests storage, not 10,257 protocol events.
    for (0..10_257) |i| net.emit(.{ .kind = 1, .version = version, .detail_a = i });
    const summary = net.summary();
    try std.testing.expectEqual(@as(u64, 257), summary.trace_dropped);
    try std.testing.expectEqual(@as(u64, 0), summary.executed);
    try std.testing.expectEqual(@as(u32, 0), summary.pending_events);
    var rows: [127]Trace = undefined;
    var after: u64 = 0;
    var count: usize = 0;
    while (true) {
        const n = net.readTrace(after, &rows);
        if (n == 0) break;
        for (rows[0..n]) |row| {
            try std.testing.expectEqual(@as(u64, 258) + count, row.ordinal);
            try std.testing.expectEqual(version, row.version);
            try std.testing.expectEqual(row.ordinal - 1, row.detail_a);
            count += 1;
            after = row.ordinal;
        }
    }
    try std.testing.expectEqual(@as(usize, 10_000), count);
    try std.testing.expectEqual(@as(usize, 0), net.readTrace(std.math.maxInt(u64), &rows));
    try std.testing.expectEqualSlices(u8, &.{ 0x45, 0x00 }, net.packets[0].data());
}
