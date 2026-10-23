const std = @import("std");
const n = @import("network.zig");
const c = n.c;
const t = std.testing;

test "codec golden header decrement preserves payload and validates checksum" {
    var bytes = [_]u8{ 0x45, 0, 0, 0x1c, 0, 1, 0, 0, 0x40, 1, 0x65, 0xdd, 10, 0, 0, 2, 10, 0, 1, 2, 8, 0, 0xf7, 0xff, 0, 0, 0, 0 };
    const expected = [_]u8{ 0x45, 0, 0, 0x1c, 0, 1, 0, 0, 0x3f, 1, 0x66, 0xdd, 10, 0, 0, 2, 10, 0, 1, 2 };
    try c.decrement(&bytes);
    try t.expectEqualSlices(u8, &expected, bytes[0..20]);
    try t.expectEqual(@as(u16, 0), c.checksum(bytes[0..20]));
    try t.expectEqual(@as(u16, 0), c.checksum(bytes[20..]));
    const view = try c.view(1, &bytes);
    var ttl_found = false;
    for (view.fields[0..view.len]) |field| if (field.code == 38) {
        try t.expectEqual(@as(u32, 8), field.byte_offset);
        try t.expectEqual(@as(u64, 63), field.value);
        ttl_found = true;
    };
    try t.expect(ttl_found);
}

test "truncated inputs never read outside packet; unsupported stays explicit" {
    var bytes: [1600]u8 = @splat(0);
    for (0..20) |len| try t.expectError(error.TruncatedIpv4, c.ipv4(bytes[0..len]));
    for (0..14) |len| try t.expectError(error.TruncatedEthernet, c.ethernet(bytes[0..len]));
    for (0..28) |len| try t.expectError(error.TruncatedArp, c.arp(bytes[0..len]));
    for (0..8) |len| try t.expectError(error.TruncatedIcmp, c.icmp(bytes[0..len]));
    c.ipHeader(&bytes, 8, 0x0a000002, 0x0a000102, 64, 1);
    bytes[8] = 63;
    try t.expectError(error.InvalidIpv4Checksum, c.ipv4(bytes[0..28]));
    c.ipHeader(&bytes, 8, 1, 2, 64, 1);
    bytes[6] = 0x20;
    bytes[10] = 0;
    bytes[11] = 0;
    c.write(bytes[10..12], c.checksum(bytes[0..20]));
    try t.expectError(error.UnsupportedFragment, c.ipv4(bytes[0..28]));
    bytes[12] = 0x81;
    bytes[13] = 0;
    try t.expectError(error.UnsupportedVlan, c.ethernet(bytes[0..60]));
}

test "hardening codec mutation corpus bounds every successful field view" {
    var rng = n.rt.Rng{ .state = 0x57495245424f58 };
    var successes: usize = 0;
    for (0..4000) |iteration| {
        var bytes: [1600]u8 = @splat(0);
        c.ipHeader(&bytes, 8, 0x0a000001, 0x0a000002, 64, 1);
        bytes[20] = 8;
        c.write(bytes[22..24], c.checksum(bytes[20..28]));
        var len: usize = 28;
        switch (iteration % 4) {
            0 => {},
            1 => bytes[@intCast(rng.below(28))] ^= @as(u8, 1) << @as(u3, @intCast(rng.below(8))),
            2 => len = @intCast(rng.below(29)),
            else => {
                len = @intCast(rng.below(1601));
                for (bytes[0..len]) |*b| b.* = @truncate(rng.next());
            },
        }
        for ([_]u32{ 1, 2 }) |layer| {
            const view = c.view(layer, bytes[0..len]) catch continue;
            successes += 1;
            try t.expect(view.len <= view.fields.len);
            for (view.fields[0..view.len]) |f| {
                try t.expect(f.bit_offset < 8 and f.bit_width > 0);
                try t.expect(@as(u64, f.byte_offset) * 8 + f.bit_offset + f.bit_width <= len * 8);
            }
        }
    }
    try t.expect(successes >= 1000); // Corpus includes known-valid controls.
}

fn direct(world: *n.Network) ![2]u64 {
    const a = try world.add(.device, 1, 0, 0);
    const b = try world.add(.device, 1, 0, 0);
    const pa = try world.add(.port, 0, a, 0);
    const pb = try world.add(.port, 0, b, 0);
    try world.setPort(pa, 0x020000000001, 0x0a000001, 24, true);
    try world.setPort(pb, 0x020000000002, 0x0a000002, 24, true);
    const link = try world.add(.link, 0, pa, pb);
    try world.setLink(link, 100_000_000, 1000);
    return .{ pa, pb };
}

test "hardening staged port changes must revalidate routes before workload starts" {
    const world = try t.allocator.create(n.Network);
    defer t.allocator.destroy(world);
    try world.init(0, 0);
    const ports = try direct(world);
    const device = world.runtime.topology.find(ports[0]).?.owner;
    try world.addRoute(device, ports[0], 0, 0, 0x0a000002);
    // Each edit is legal alone; together they leave an off-link gateway.
    try world.setPort(ports[0], 0x020000000001, 0x0a000101, 24, true);
    try t.expectError(error.InvalidArgument, world.ping(ports[0], 0xcb007101, 64));
    try t.expect(!world.started and !world.failed);
    try t.expectEqual(@as(usize, 0), world.packet_count);
    try t.expectEqual(@as(usize, 0), world.runtime.queue.len);
    try t.expectEqual(@as(u64, 0), world.trace_total);
    // Validation failure leaves configuration editable, unlike dispatch failure.
    try world.setPort(ports[0], 0x020000000001, 0x0a000001, 24, true);
    try world.ping(ports[0], 0xcb007101, 64);
    try t.expect(world.started);
}
test "network echo runs through shared scheduler and immutable packet versions" {
    const world = try t.allocator.create(n.Network);
    defer t.allocator.destroy(world);
    try world.init(0, 1024);
    const ports = try direct(world);
    try world.ping(ports[0], 0x0a000002, 64);
    var progress: n.rt.Progress = .{};
    try world.run(5_000_000_000, 1000, &progress);
    try t.expectEqual(@as(u32, 1), world.summary().outcome);
    try t.expectEqual(@as(u32, 0), world.summary().pending_arp);
    try t.expectEqual(@as(u32, 0), world.summary().pending_events);
    try t.expectEqual(@as(u64, 5800), world.txs[0].arrival_ns);
    const original = try world.packet(1);
    try t.expectEqual(@as(u8, 8), original.bytes[20]);
    try t.expectEqual(@as(u8, 64), original.bytes[8]);
}

test "switch flood branches share bytes with distinct transmissions and later unicast" {
    const world = try t.allocator.create(n.Network);
    defer t.allocator.destroy(world);
    try world.init(0, 1024);
    const sw = try world.add(.device, 2, 0, 0);
    var ports: [3]u64 = undefined;
    for (&ports, 0..) |*port, i| {
        const dev = try world.add(.device, 1, 0, 0);
        port.* = try world.add(.port, 0, dev, 0);
        const sp = try world.add(.port, 0, sw, 0);
        try world.setPort(port.*, 0x020000000001 + i, @intCast(0x0a000001 + i), 24, true);
        const link = try world.add(.link, 0, sp, port.*);
        try world.setLink(link, 100_000_000, 1000);
    }
    try world.ping(ports[0], 0x0a000002, 64);
    var progress: n.rt.Progress = .{};
    try world.run(5_000_000_000, 1000, &progress);
    try t.expectEqual(@as(u32, 1), world.outcome);
    // First ARP request is transmitted host->switch, then to both other ports.
    try t.expectEqual(world.txs[0].version, world.txs[1].version);
    try t.expectEqual(world.txs[0].version, world.txs[2].version);
    try t.expect(world.txs[1].id != world.txs[2].id and world.txs[1].to != world.txs[2].to);
    try t.expectEqual(world.txs[0].event_id, world.txs[1].cause_id);
    var rows: [1024]n.Trace = undefined;
    const count = world.readTrace(0, &rows);
    var unicasts: usize = 0;
    for (rows[0..count]) |row| if (row.kind == 9) {
        unicasts += 1;
    };
    try t.expect(unicasts >= 3);
}

test "static LPM overrides connected route; disabled egress and off-link gateway rejected" {
    const world = try t.allocator.create(n.Network);
    defer t.allocator.destroy(world);
    try world.init(0, 0);
    const router = try world.add(.device, 3, 0, 0);
    const a = try world.add(.port, 0, router, 0);
    const b = try world.add(.port, 0, router, 0);
    try world.setPort(a, 0x020000000001, 0x0a000001, 24, true);
    try world.setPort(b, 0x020000000101, 0x0a000101, 24, true);
    try world.addRoute(router, a, 0, 0, 0x0a000002);
    try world.addRoute(router, a, 0x0a000102, 32, 0x0a000002);
    try t.expectEqual(a, world.route(router, 0x0a000102).?.port);
    try t.expectEqual(b, world.route(router, 0x0a000103).?.port);
    try t.expectEqual(a, world.route(router, 0xcb007101).?.port);
    try t.expectError(error.Conflict, world.addRoute(router, b, 0x0a000102, 32, 0));
    try t.expectError(error.InvalidArgument, world.addRoute(router, a, 0xc6336400, 24, 0x0a000102));
    try world.setPort(a, 0x020000000001, 0x0a000001, 24, false);
    try t.expectEqual(b, world.route(router, 0x0a000102).?.port);
    try t.expect(world.route(router, 0xcb007101) == null);
}

test "echo preserves odd-length data and field values" {
    const world = try t.allocator.create(n.Network);
    defer t.allocator.destroy(world);
    try world.init(0, 1024);
    const ports = try direct(world);
    var bytes: [31]u8 = @splat(0);
    c.ipHeader(&bytes, 11, 0x0a000001, 0x0a000002, 64, 1);
    bytes[20] = 8;
    bytes[24] = 0x12;
    bytes[25] = 0x34;
    bytes[27] = 1;
    @memcpy(bytes[28..], &[_]u8{ 0xab, 0xcd, 0xef });
    c.write(bytes[22..24], c.checksum(bytes[20..]));
    try world.inject(ports[1], 1, &bytes);
    const reply = try world.packet(2);
    try t.expectEqualSlices(u8, bytes[24..], reply.bytes[24..31]);
    try t.expectEqual(@as(u8, 0), reply.bytes[20]);
    try t.expectEqual(@as(u16, 0), c.checksum(reply.bytes[20..31]));
    const view = try c.view(1, reply.data());
    var found = false;
    for (view.fields[0..view.len]) |field| if (field.code == 55) {
        try t.expectEqual(@as(u64, 0xabcdef), field.value);
        found = true;
    };
    try t.expect(found);
}

test "ICMP error suppression for error-of-error and L2 broadcast" {
    const world = try t.allocator.create(n.Network);
    defer t.allocator.destroy(world);
    for ([_]bool{ false, true }) |broadcast| {
        try world.init(0, 1024);
        const ports = try direct(world);
        const device = world.runtime.topology.find(ports[1]).?.owner;
        world.runtime.topology.find(device).?.role = 3;
        var bytes: [56]u8 = @splat(0);
        c.ipHeader(&bytes, if (broadcast) 8 else 36, 0x0a000001, 0xcb007101, if (broadcast) 1 else 64, 1);
        bytes[20] = if (broadcast) 8 else 3;
        c.write(bytes[22..24], c.checksum(bytes[20..if (broadcast) 28 else 56]));
        if (broadcast) {
            var frame: [60]u8 = undefined;
            _ = c.frame(&frame, 0x020000000001, 0xffffffffffff, 0x0800, bytes[0..28]);
            try world.inject(ports[1], 2, &frame);
        } else try world.inject(ports[1], 1, &bytes);
        try t.expectEqual(@as(u32, 7), world.outcome);
        try t.expectEqual(@as(usize, 0), world.tx_count);
        try t.expectEqual(@as(usize, 0), world.runtime.queue.len);
        var rows: [16]n.Trace = undefined;
        const count = world.readTrace(0, &rows);
        for (rows[0..count]) |row| try t.expect(row.kind != 15);
    }
}
