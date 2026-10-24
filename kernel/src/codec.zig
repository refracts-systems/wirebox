//! Explicit network byte order. Decoders validate lengths before field reads.
const std = @import("std");
pub const max_bytes = 1600;
pub const Error = error{
    TruncatedEthernet,
    UnsupportedEthertype,
    TruncatedArp,
    InvalidArp,
    TruncatedIpv4,
    InvalidIpv4,
    InvalidIpv4Checksum,
    UnsupportedOptions,
    UnsupportedFragment,
    UnsupportedProtocol,
    TruncatedIcmp,
    InvalidIcmpChecksum,
    UnsupportedIcmp,
    UnsupportedMtu,
    UnsupportedVlan,
    InvalidEthernet,
};
pub const Reason = enum(u32) {
    none = 0,
    truncated_ethernet,
    unsupported_ethertype,
    truncated_arp,
    invalid_arp,
    truncated_ipv4,
    invalid_ipv4,
    invalid_ipv4_checksum,
    unsupported_options,
    unsupported_fragment,
    unsupported_protocol,
    truncated_icmp,
    invalid_icmp_checksum,
    unsupported_icmp,
    unsupported_mtu,
    unsupported_vlan,
    invalid_ethernet,
    no_route = 100,
    ttl_expired,
    arp_timeout,
    port_down,
    mac_filter,
    not_router,
    icmp_error_suppressed,
    unsolicited_reply,
    unsupported_address,
    link_loss,
};
pub fn reason(err: Error) Reason {
    return switch (err) {
        error.TruncatedEthernet => .truncated_ethernet,
        error.UnsupportedEthertype => .unsupported_ethertype,
        error.TruncatedArp => .truncated_arp,
        error.InvalidArp => .invalid_arp,
        error.TruncatedIpv4 => .truncated_ipv4,
        error.InvalidIpv4 => .invalid_ipv4,
        error.InvalidIpv4Checksum => .invalid_ipv4_checksum,
        error.UnsupportedOptions => .unsupported_options,
        error.UnsupportedFragment => .unsupported_fragment,
        error.UnsupportedProtocol => .unsupported_protocol,
        error.TruncatedIcmp => .truncated_icmp,
        error.InvalidIcmpChecksum => .invalid_icmp_checksum,
        error.UnsupportedIcmp => .unsupported_icmp,
        error.UnsupportedMtu => .unsupported_mtu,
        error.UnsupportedVlan => .unsupported_vlan,
        error.InvalidEthernet => .invalid_ethernet,
    };
}
pub fn unsupported(r: Reason) bool {
    return switch (r) {
        .unsupported_ethertype, .unsupported_options, .unsupported_fragment, .unsupported_protocol, .unsupported_icmp, .unsupported_mtu, .unsupported_vlan, .unsupported_address => true,
        else => false,
    };
}
pub fn read(bytes: []const u8) u64 {
    std.debug.assert(bytes.len <= 8);
    var value: u64 = 0;
    for (bytes) |b| value = (value << 8) | b;
    return value;
}
pub fn write(bytes: []u8, value: u64) void {
    std.debug.assert(bytes.len <= 8);
    for (bytes, 0..) |*b, i| b.* = @truncate(value >> @as(u6, @intCast((bytes.len - 1 - i) * 8)));
}
pub fn checksum(bytes: []const u8) u16 {
    var sum: u32 = 0;
    var i: usize = 0;
    while (i < bytes.len) : (i += 2) {
        sum += (@as(u32, bytes[i]) << 8) | (if (i + 1 < bytes.len) @as(u32, bytes[i + 1]) else 0);
        sum = (sum & 0xffff) + (sum >> 16);
    }
    while (sum >> 16 != 0) sum = (sum & 0xffff) + (sum >> 16);
    return ~@as(u16, @truncate(sum));
}
pub const Ethernet = struct { dst: u64, src: u64, kind: u16 };
pub fn ethernet(bytes: []const u8) Error!Ethernet {
    // minimum has 14 bytes for ethernet header, 46–1500 byte payload
    if (bytes.len < 14) return error.TruncatedEthernet;
    const kind: u16 = @intCast(read(bytes[12..14]));
    if (kind == 0x8100 or kind == 0x88a8) return error.UnsupportedVlan;
    if (kind != 0x0800 and kind != 0x0806) return error.UnsupportedEthertype;
    const src = read(bytes[6..12]);
    if (src == 0 or src & 0x010000000000 != 0) return error.InvalidEthernet;
    // tidak menyertakan 4-byte FCS/CRC.
    if (bytes.len > 1514) return error.UnsupportedMtu;
    return .{ .dst = read(bytes[0..6]), .src = src, .kind = kind };
}
pub const Arp = struct { op: u16, sha: u64, spa: u32, tha: u64, tpa: u32 };
// Address Resolution Protocol. MAC address dari sebuah IPv4 address di jaringan lokal.
pub fn arp(bytes: []const u8) Error!Arp {
    if (bytes.len < 28) return error.TruncatedArp;
    if (read(bytes[0..2]) != 1 or read(bytes[2..4]) != 0x0800 or bytes[4] != 6 or bytes[5] != 4) return error.InvalidArp;
    const op = read(bytes[6..8]);
    if (op != 1 and op != 2) return error.InvalidArp;
    return .{ .op = @intCast(op), .sha = read(bytes[8..14]), .spa = @intCast(read(bytes[14..18])), .tha = read(bytes[18..24]), .tpa = @intCast(read(bytes[24..28])) };
}
pub const Ip = struct { src: u32, dst: u32, total: usize, ttl: u8, protocol: u8 };
pub fn ipv4(bytes: []const u8) Error!Ip {
    if (bytes.len < 20) return error.TruncatedIpv4;
    if (bytes[0] >> 4 != 4 or bytes[0] & 15 < 5) return error.InvalidIpv4;
    const header = @as(usize, bytes[0] & 15) * 4;
    if (bytes.len < header) return error.TruncatedIpv4;
    const total: usize = @intCast(read(bytes[2..4]));
    if (total < header) return error.InvalidIpv4;
    if (total > bytes.len) return error.TruncatedIpv4;
    if (checksum(bytes[0..header]) != 0) return error.InvalidIpv4Checksum;
    if (header != 20) return error.UnsupportedOptions;
    const flags = read(bytes[6..8]);
    if (flags & 0x8000 != 0) return error.InvalidIpv4;
    if (flags & 0x3fff != 0) return error.UnsupportedFragment;
    if (total > 1500) return error.UnsupportedMtu;
    return .{ .src = @intCast(read(bytes[12..16])), .dst = @intCast(read(bytes[16..20])), .total = total, .ttl = bytes[8], .protocol = bytes[9] };
}
pub fn icmp(bytes: []const u8) Error!void {
    if (bytes.len < 8) return error.TruncatedIcmp;
    if (checksum(bytes) != 0) return error.InvalidIcmpChecksum;
    if (!((bytes[0] == 0 or bytes[0] == 8 or bytes[0] == 3 or bytes[0] == 11) and bytes[1] == 0)) return error.UnsupportedIcmp;
    if ((bytes[0] == 3 or bytes[0] == 11) and bytes.len < 36) return error.TruncatedIcmp;
}
pub fn ipHeader(out: []u8, payload_len: usize, src: u32, dst: u32, ttl: u8, id: u16) void {
    std.debug.assert(out.len >= 20 and payload_len <= 1480);
    @memset(out[0..20], 0);
    out[0] = 0x45;
    write(out[2..4], 20 + payload_len);
    write(out[4..6], id);
    out[8] = ttl;
    out[9] = 1;
    write(out[12..16], src);
    write(out[16..20], dst);
    write(out[10..12], checksum(out[0..20]));
}
pub fn decrement(out: []u8) Error!void {
    const ip = try ipv4(out);
    if (ip.ttl <= 1) return error.InvalidIpv4;
    out[8] -= 1;
    out[10] = 0;
    out[11] = 0;
    write(out[10..12], checksum(out[0..20]));
}
pub fn frame(out: []u8, src: u64, dst: u64, kind: u16, payload: []const u8) usize {
    const len = @max(60, payload.len + 14);
    std.debug.assert(len <= out.len);
    @memset(out[0..len], 0);
    write(out[0..6], dst);
    write(out[6..12], src);
    write(out[12..14], kind);
    @memcpy(out[14..][0..payload.len], payload);
    return len;
}
pub fn arpBytes(out: *[28]u8, msg: Arp) void {
    @memset(out, 0);
    write(out[0..2], 1);
    write(out[2..4], 0x0800);
    out[4] = 6;
    out[5] = 4;
    write(out[6..8], msg.op);
    write(out[8..14], msg.sha);
    write(out[14..18], msg.spa);
    write(out[18..24], msg.tha);
    write(out[24..28], msg.tpa);
}
pub const Field = extern struct { code: u32, byte_offset: u32, bit_offset: u32, bit_width: u32, value: u64 };
pub const View = struct {
    fields: [40]Field = undefined,
    len: usize = 0,
    fn add(self: *View, code: u32, offset: usize, bit: u32, width: u32, value: u64) void {
        self.fields[self.len] = .{ .code = code, .byte_offset = @intCast(offset), .bit_offset = bit, .bit_width = width, .value = value };
        self.len += 1;
    }
};
pub fn view(layer: u32, bytes: []const u8) Error!View {
    var result: View = .{};
    var offset: usize = 0;
    var kind: u16 = 0x0800;
    if (layer == 2) {
        const eth = try ethernet(bytes);
        result.add(1, 0, 0, 48, eth.dst);
        result.add(2, 6, 0, 48, eth.src);
        result.add(3, 12, 0, 16, eth.kind);
        offset = 14;
        kind = eth.kind;
    } else if (layer != 1) return error.UnsupportedEthertype;
    const data = bytes[offset..];
    var payload_end: usize = 0;
    if (kind == 0x0806) {
        const a = try arp(data);
        result.add(10, offset, 0, 16, 1);
        result.add(11, offset + 2, 0, 16, 0x0800);
        result.add(12, offset + 4, 0, 8, 6);
        result.add(13, offset + 5, 0, 8, 4);
        result.add(14, offset + 6, 0, 16, a.op);
        result.add(15, offset + 8, 0, 48, a.sha);
        result.add(16, offset + 14, 0, 32, a.spa);
        result.add(17, offset + 18, 0, 48, a.tha);
        result.add(18, offset + 24, 0, 32, a.tpa);
        payload_end = offset + 28;
    } else {
        const ip = try ipv4(data);
        payload_end = offset + ip.total;
        result.add(30, offset, 0, 4, 4);
        result.add(31, offset, 4, 4, 5);
        result.add(32, offset + 1, 0, 6, data[1] >> 2);
        result.add(33, offset + 1, 6, 2, data[1] & 3);
        result.add(34, offset + 2, 0, 16, ip.total);
        result.add(35, offset + 4, 0, 16, read(data[4..6]));
        result.add(36, offset + 6, 0, 3, data[6] >> 5);
        result.add(37, offset + 6, 3, 13, read(data[6..8]) & 0x1fff);
        result.add(38, offset + 8, 0, 8, ip.ttl);
        result.add(39, offset + 9, 0, 8, ip.protocol);
        result.add(40, offset + 10, 0, 16, read(data[10..12]));
        result.add(41, offset + 12, 0, 32, ip.src);
        result.add(42, offset + 16, 0, 32, ip.dst);
        if (ip.protocol != 1) return error.UnsupportedProtocol;
        const msg = data[20..ip.total];
        try icmp(msg);
        result.add(50, offset + 20, 0, 8, msg[0]);
        result.add(51, offset + 21, 0, 8, msg[1]);
        result.add(52, offset + 22, 0, 16, read(msg[2..4]));
        if (msg[0] == 0 or msg[0] == 8) {
            result.add(53, offset + 24, 0, 16, read(msg[4..6]));
            result.add(54, offset + 26, 0, 16, read(msg[6..8]));
            if (msg.len > 8) result.add(55, offset + 28, 0, @intCast((msg.len - 8) * 8), if (msg.len <= 16) read(msg[8..]) else 0);
        } else result.add(56, offset + 28, 0, @intCast((msg.len - 8) * 8), 0);
    }
    if (layer == 2 and payload_end < bytes.len) result.add(4, payload_end, 0, @intCast((bytes.len - payload_end) * 8), if (bytes.len - payload_end <= 8) read(bytes[payload_end..]) else 0);
    return result;
}
