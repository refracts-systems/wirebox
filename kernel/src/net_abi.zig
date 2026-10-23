const std = @import("std");
pub const net = @import("network.zig");
const rt = net.rt;
const tag: u64 = 3 << 62;
const generation_max: u32 = (@as(u32, 1) << (@bitSizeOf(u64) - 32 - 2)) - 1;
const Slot = struct { generation: u32 = 1, world: ?*net.Network = null };
var slots: [4]Slot = @splat(.{});

test "hardening network registry retires at namespace boundary without leaking allocation" {
    const saved = slots;
    defer slots = saved;
    // Cleanup test-owned allocations even if an assertion fails.
    slots = @splat(.{ .generation = 0 });
    defer for (&slots) |*slot| {
        if (slot.world) |world| std.heap.page_allocator.destroy(world);
    };
    for ([_]u32{ 1, generation_max - 1, generation_max }) |g| {
        slots[0].generation = g;
        var old: u64 = 0;
        try std.testing.expectEqual(@as(u32, 0), wb_net_create(1, 0, 0, &old));
        try std.testing.expectEqual(@as(u64, 3), old >> 62);
        for (0..3) |namespace| {
            const foreign = (old & ~tag) | (@as(u64, namespace) << 62);
            try std.testing.expectEqual(@as(u32, 3), wb_net_destroy(foreign));
            try std.testing.expect(lookup(old) != null);
        }
        try std.testing.expectEqual(@as(u32, 0), wb_net_destroy(old));
        try std.testing.expect(lookup(old) == null);
        var next: u64 = 99;
        if (g == generation_max) {
            try std.testing.expectEqual(@as(u32, 4), wb_net_create(1, 0, 0, &next));
            try std.testing.expectEqual(@as(u64, 0), next);
            try std.testing.expectEqual(@as(u32, 0), slots[0].generation);
        } else {
            try std.testing.expectEqual(@as(u32, 0), wb_net_create(1, 0, 0, &next));
            try std.testing.expectEqual(@as(u64, 3), next >> 62);
            try std.testing.expect(lookup(next) != null and lookup(old) == null);
            try std.testing.expectEqual(@as(u32, 0), wb_net_destroy(next));
        }
    }
}
fn lookup(handle: u64) ?*net.Network {
    if (handle >> 62 != 3) return null;
    const index: u32 = @truncate(handle);
    const generation: u32 = @intCast((handle & ~tag) >> 32);
    if (index == 0 or index > slots.len or generation == 0) return null;
    const slot = &slots[index - 1];
    return if (slot.generation == generation) slot.world else null;
}
fn status(err: net.Error) u32 {
    return switch (err) {
        error.InvalidArgument => 1,
        error.Capacity => 4,
        error.NotFound => 5,
        error.Conflict => 6,
        error.PastTime => 7,
        error.Overflow => 8,
        error.State => 9,
        error.OutOfMemory => 10,
    };
}
fn failed(world: *net.Network, err: net.Error) u32 {
    if (world.started and (err == error.Capacity or err == error.Overflow or err == error.OutOfMemory)) world.failed = true;
    return status(err);
}
export fn wb_net_create(version: u32, seed: u64, retention: usize, out: ?*u64) callconv(.c) u32 {
    const handle = out orelse return 1;
    handle.* = 0;
    if (version != 1) return 2;
    if (retention > net.trace_limit) return 1;
    for (&slots, 0..) |*slot, index| if (slot.world == null and slot.generation != 0) {
        const world = std.heap.page_allocator.create(net.Network) catch return 10;
        world.init(seed, retention) catch |err| {
            std.heap.page_allocator.destroy(world);
            return status(err);
        };
        slot.world = world;
        handle.* = tag | (@as(u64, slot.generation) << 32) | @as(u64, index + 1);
        return 0;
    };
    return 4;
}
export fn wb_net_destroy(handle: u64) callconv(.c) u32 {
    const world = lookup(handle) orelse return 3;
    std.heap.page_allocator.destroy(world);
    const index: u32 = @truncate(handle);
    const slot = &slots[index - 1];
    slot.world = null;
    slot.generation = if (slot.generation == generation_max) 0 else slot.generation + 1;
    return 0;
}
export fn wb_net_add(handle: u64, kind: u32, role: u32, a: u64, b: u64, out: ?*u64) callconv(.c) u32 {
    const id = out orelse return 1;
    id.* = 0;
    const world = lookup(handle) orelse return 3;
    const typed = std.enums.fromInt(rt.graph.Kind, kind) orelse return 1;
    id.* = world.add(typed, role, a, b) catch |err| return status(err);
    return 0;
}
export fn wb_net_port(handle: u64, id: u64, mac: u64, ip: u32, prefix: u32, enabled: u32) callconv(.c) u32 {
    const world = lookup(handle) orelse return 3;
    if (enabled > 1) return 1;
    world.setPort(id, mac, ip, prefix, enabled == 1) catch |err| return status(err);
    return 0;
}
export fn wb_net_link(handle: u64, id: u64, bitrate: u64, propagation: u64) callconv(.c) u32 {
    const world = lookup(handle) orelse return 3;
    world.setLink(id, bitrate, propagation) catch |err| return status(err);
    return 0;
}
export fn wb_net_alias(handle: u64, device: u64, ip: u32) callconv(.c) u32 {
    const world = lookup(handle) orelse return 3;
    world.addAlias(device, ip) catch |err| return status(err);
    return 0;
}
export fn wb_net_route(handle: u64, device: u64, port: u64, network: u32, prefix: u32, via: u32) callconv(.c) u32 {
    const world = lookup(handle) orelse return 3;
    world.addRoute(device, port, network, prefix, via) catch |err| return status(err);
    return 0;
}
export fn wb_net_ping(handle: u64, port: u64, destination: u32, ttl: u32) callconv(.c) u32 {
    const world = lookup(handle) orelse return 3;
    world.ping(port, destination, ttl) catch |err| return failed(world, err);
    return 0;
}
export fn wb_net_inject(handle: u64, port: u64, layer: u32, bytes: ?[*]const u8, len: usize) callconv(.c) u32 {
    const world = lookup(handle) orelse return 3;
    if (bytes == null or len > net.c.max_bytes) return 1;
    world.inject(port, layer, bytes.?[0..len]) catch |err| return failed(world, err);
    return 0;
}
export fn wb_net_run(handle: u64, horizon: u64, budget: u64, out: ?*rt.Progress) callconv(.c) u32 {
    const progress = out orelse return 1;
    progress.* = .{ .reason = 4 };
    const world = lookup(handle) orelse return 3;
    world.run(horizon, budget, progress) catch |err| return failed(world, err);
    return 0;
}
export fn wb_net_summary(handle: u64, out: ?*net.Summary) callconv(.c) u32 {
    const summary = out orelse return 1;
    summary.* = std.mem.zeroes(net.Summary);
    const world = lookup(handle) orelse return 3;
    summary.* = world.summary();
    return 0;
}
export fn wb_net_packet(handle: u64, version: u64, out: ?*net.Packet) callconv(.c) u32 {
    const packet = out orelse return 1;
    packet.* = .{};
    const world = lookup(handle) orelse return 3;
    packet.* = (world.packet(version) catch |err| return status(err)).*;
    return 0;
}
export fn wb_net_tx(handle: u64, id: u64, out: ?*net.Tx) callconv(.c) u32 {
    const tx = out orelse return 1;
    tx.* = .{};
    const world = lookup(handle) orelse return 3;
    if (id == 0 or id > world.tx_count) return 5;
    tx.* = world.txs[@intCast(id - 1)];
    return 0;
}
export fn wb_net_trace(handle: u64, after: u64, out: ?[*]net.Trace, capacity: usize, out_count: ?*usize) callconv(.c) u32 {
    const count = out_count orelse return 1;
    count.* = 0;
    const world = lookup(handle) orelse return 3;
    if (capacity > 256 or (capacity > 0 and out == null)) return 1;
    if (capacity > 0) count.* = world.readTrace(after, out.?[0..capacity]);
    return 0;
}
export fn wb_net_fields(handle: u64, version: u64, out: ?[*]net.c.Field, capacity: usize, out_count: ?*usize) callconv(.c) u32 {
    const count = out_count orelse return 1;
    count.* = 0;
    const world = lookup(handle) orelse return 3;
    if (capacity > 40 or (capacity > 0 and out == null)) return 1;
    const packet = world.packet(version) catch |err| return status(err);
    const view = net.c.view(packet.layer, packet.data()) catch |err| return if (net.c.unsupported(net.c.reason(err))) 12 else 11;
    if (capacity < view.len) return 4;
    if (view.len > 0) @memcpy(out.?[0..view.len], view.fields[0..view.len]);
    count.* = view.len;
    return 0;
}

test "network ABI allocation capacity, pointer errors and failed resource state" {
    const t = std.testing;
    var handles: [4]u64 = @splat(0);
    defer for (handles) |handle| {
        _ = wb_net_destroy(handle);
    };
    for (&handles) |*handle| try t.expectEqual(@as(u32, 0), wb_net_create(1, 0, 1024, handle));
    var extra: u64 = 99;
    try t.expectEqual(@as(u32, 4), wb_net_create(1, 0, 0, &extra));
    try t.expectEqual(@as(u64, 0), extra);
    const handle = handles[0];
    var dev: u64 = 0;
    var port: u64 = 0;
    try t.expectEqual(@as(u32, 0), wb_net_add(handle, 1, 1, 0, 0, &dev));
    try t.expectEqual(@as(u32, 0), wb_net_add(handle, 2, 0, dev, 0, &port));
    try t.expectEqual(@as(u32, 0), wb_net_port(handle, port, 0x020000000001, 0x0a000001, 24, 1));
    try t.expectEqual(@as(u32, 1), wb_net_inject(handle, port, 1, null, 2));
    try t.expectEqual(@as(u32, 1), wb_net_packet(handle, 1, null));
    var count: usize = 99;
    try t.expectEqual(@as(u32, 1), wb_net_trace(handle, 0, null, 1, &count));
    try t.expectEqual(@as(usize, 0), count);
    lookup(handle).?.packet_count = net.packet_limit;
    try t.expectEqual(@as(u32, 4), wb_net_ping(handle, port, 0x0a000002, 64));
    var summary: net.Summary = undefined;
    try t.expectEqual(@as(u32, 0), wb_net_summary(handle, &summary));
    try t.expectEqual(@as(u32, 1), summary.failed);
    try t.expectEqual(@as(u32, 0), summary.outcome);
    var progress: rt.Progress = .{};
    try t.expectEqual(@as(u32, 9), wb_net_run(handle, 1, 1, &progress));
}
