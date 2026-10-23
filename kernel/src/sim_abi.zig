//! Runtime ABI v1. All calls share the host's process-wide registry lock.
const std = @import("std");
const rt = @import("runtime.zig");
const world_limit = 8;
const tag: u64 = 1 << 63;
const generation_max: u32 = (@as(u32, 1) << (@bitSizeOf(u64) - 32 - 2)) - 1;
const Slot = struct { generation: u32 = 1, world: ?rt.Runtime = null };
var worlds: [world_limit]Slot = @splat(.{});

test "hardening runtime registry retires at namespace boundary and rejects foreign handles" {
    const saved = worlds;
    defer worlds = saved;
    for ([_]u32{ 1, generation_max - 1, generation_max }) |g| {
        worlds = @splat(.{ .generation = 0 });
        worlds[0].generation = g;
        var old: u64 = 0;
        try std.testing.expectEqual(@as(u32, 0), wb_sim_create(1, 0, 0, &old));
        try std.testing.expectEqual(@as(u64, 2), old >> 62);
        for ([_]u64{ 0, 1, 3 }) |namespace| {
            const foreign = (old & ~(@as(u64, 3) << 62)) | (namespace << 62);
            try std.testing.expectEqual(@as(u32, 3), wb_sim_destroy(foreign));
            try std.testing.expect(lookup(old) != null);
        }
        try std.testing.expectEqual(@as(u32, 0), wb_sim_destroy(old));
        try std.testing.expect(lookup(old) == null);
        var next: u64 = 99;
        if (g == generation_max) {
            try std.testing.expectEqual(@as(u32, 4), wb_sim_create(1, 0, 0, &next));
            try std.testing.expectEqual(@as(u64, 0), next);
            try std.testing.expectEqual(@as(u32, 0), worlds[0].generation);
        } else {
            try std.testing.expectEqual(@as(u32, 0), wb_sim_create(1, 0, 0, &next));
            try std.testing.expectEqual(@as(u64, 2), next >> 62);
            try std.testing.expect(lookup(next) != null and lookup(old) == null);
            try std.testing.expectEqual(@as(u32, 0), wb_sim_destroy(next));
        }
    }
}

fn lookup(handle: u64) ?*rt.Runtime {
    if (handle >> 62 != 2) return null;
    const index: u32 = @truncate(handle);
    const generation: u32 = @intCast((handle & ~tag) >> 32);
    if (index == 0 or index > world_limit or generation == 0) return null;
    const slot = &worlds[index - 1];
    if (slot.generation != generation) return null;
    if (slot.world) |*world| return world;
    return null;
}
fn status(err: rt.Error) u32 {
    return switch (err) {
        error.InvalidArgument => 1,
        error.Capacity => 4,
        error.NotFound => 5,
        error.Conflict => 6,
        error.PastTime => 7,
        error.Overflow => 8,
    };
}
export fn wb_sim_version() callconv(.c) u32 {
    return 1;
}
export fn wb_sim_create(version: u32, seed: u64, retention: usize, out_handle: ?*u64) callconv(.c) u32 {
    const out = out_handle orelse return 1;
    out.* = 0;
    if (version != 1) return 2;
    if (retention > rt.journal_limit) return 1;
    for (&worlds, 0..) |*slot, index| {
        if (slot.world == null and slot.generation != 0) {
            slot.world = rt.Runtime.init(seed, retention) catch |err| return status(err);
            out.* = tag | (@as(u64, slot.generation) << 32) | @as(u64, index + 1);
            return 0;
        }
    }
    return 4;
}
export fn wb_sim_destroy(handle: u64) callconv(.c) u32 {
    _ = lookup(handle) orelse return 3;
    const index: u32 = @truncate(handle);
    const slot = &worlds[index - 1];
    slot.world = null;
    slot.generation = if (slot.generation == generation_max) 0 else slot.generation + 1;
    return 0;
}
export fn wb_sim_add(handle: u64, kind: u32, role: u32, a: u64, b: u64, out_id: ?*u64) callconv(.c) u32 {
    const out = out_id orelse return 1;
    out.* = 0;
    const world = lookup(handle) orelse return 3;
    const typed = std.enums.fromInt(rt.graph.Kind, kind) orelse return 1;
    out.* = world.topology.add(typed, role, a, b) catch |err| return status(err);
    return 0;
}
export fn wb_sim_remove(handle: u64, id: u64) callconv(.c) u32 {
    const world = lookup(handle) orelse return 3;
    world.remove(id) catch |err| return status(err);
    return 0;
}
export fn wb_sim_schedule(handle: u64, spec: ?*const rt.Schedule, out_timer: ?*u64) callconv(.c) u32 {
    const out = out_timer orelse return 1;
    out.* = 0;
    const world = lookup(handle) orelse return 3;
    const input = spec orelse return 1;
    out.* = world.schedule(input.*) catch |err| return status(err);
    return 0;
}
export fn wb_sim_cancel(handle: u64, timer: u64) callconv(.c) u32 {
    const world = lookup(handle) orelse return 3;
    world.cancel(timer) catch |err| return status(err);
    return 0;
}
export fn wb_sim_step(handle: u64, out: ?*rt.Progress) callconv(.c) u32 {
    const progress = out orelse return 1;
    progress.* = .{ .reason = 4 };
    const world = lookup(handle) orelse return 3;
    progress.now_ns = world.now;
    const stepped = world.step() catch |err| return status(err);
    progress.* = .{ .now_ns = world.now, .processed = if (stepped) 1 else 0, .reason = if (stepped) 3 else 0 };
    return 0;
}
export fn wb_sim_run_until(handle: u64, horizon: u64, budget: u64, out: ?*rt.Progress) callconv(.c) u32 {
    const progress = out orelse return 1;
    progress.* = .{ .reason = 4 };
    const world = lookup(handle) orelse return 3;
    world.runUntil(horizon, budget, progress) catch |err| return status(err);
    return 0;
}
export fn wb_sim_snapshot(handle: u64, out: ?*rt.Snapshot) callconv(.c) u32 {
    const snapshot = out orelse return 1;
    snapshot.* = std.mem.zeroes(rt.Snapshot);
    const world = lookup(handle) orelse return 3;
    snapshot.* = world.snapshot();
    return 0;
}
export fn wb_sim_entities(handle: u64, out: ?[*]rt.graph.Entity, capacity: usize, out_count: ?*usize) callconv(.c) u32 {
    const count = out_count orelse return 1;
    count.* = 0;
    const world = lookup(handle) orelse return 3;
    if (capacity > rt.graph.limit or (capacity > 0 and out == null)) return 1;
    // Graph snapshots are complete or explicitly rejected; no silent truncation.
    if (capacity < world.topology.count) return 4;
    for (world.topology.entries) |entity| {
        if (entity.id == 0) continue;
        out.?[count.*] = entity;
        count.* += 1;
    }
    return 0;
}
export fn wb_sim_trace(handle: u64, after_ordinal: u64, out: ?[*]rt.Trace, capacity: usize, out_count: ?*usize) callconv(.c) u32 {
    const count = out_count orelse return 1;
    count.* = 0;
    const world = lookup(handle) orelse return 3;
    if (capacity > rt.journal_limit or (capacity > 0 and out == null)) return 1;
    if (capacity > 0) count.* = world.read(after_ordinal, out.?[0..capacity]);
    return 0;
}

test "runtime ABI rejects null, wrong versions, stale worlds and short graph outputs" {
    const t = std.testing;
    var handle: u64 = 99;
    try t.expectEqual(@as(u32, 1), wb_sim_create(1, 0, 1, null));
    try t.expectEqual(@as(u32, 2), wb_sim_create(2, 0, 1, &handle));
    try t.expectEqual(@as(u64, 0), handle);
    try t.expectEqual(@as(u32, 1), wb_sim_create(1, 0, 257, &handle));
    try t.expectEqual(@as(u32, 0), wb_sim_create(1, 0, 1, &handle));
    var id: u64 = 0;
    try t.expectEqual(@as(u32, 0), wb_sim_add(handle, 1, 1, 0, 0, &id));
    try t.expectEqual(@as(u32, 1), wb_sim_schedule(handle, null, &id));
    var count: usize = 123;
    try t.expectEqual(@as(u32, 4), wb_sim_entities(handle, null, 0, &count));
    try t.expectEqual(@as(usize, 0), count);
    try t.expectEqual(@as(u32, 1), wb_sim_trace(handle, 0, null, 1, &count));
    try t.expectEqual(@as(u32, 1), wb_sim_snapshot(handle, null));
    try t.expectEqual(@as(u32, 0), wb_sim_destroy(handle));
    var next: u64 = 0;
    try t.expectEqual(@as(u32, 0), wb_sim_create(1, 0, 0, &next));
    defer _ = wb_sim_destroy(next);
    try t.expect(next != handle);
    try t.expectEqual(@as(u32, 3), wb_sim_destroy(handle));
    try t.expectEqual(@as(u32, 3), wb_sim_destroy(next & ~tag));
}

test "runtime world capacity and release" {
    var handles: [world_limit]u64 = @splat(0);
    defer for (handles) |handle| {
        _ = wb_sim_destroy(handle);
    };
    for (&handles) |*handle| try std.testing.expectEqual(@as(u32, 0), wb_sim_create(1, 0, 0, handle));
    var extra: u64 = 1;
    try std.testing.expectEqual(@as(u32, 4), wb_sim_create(1, 0, 0, &extra));
    try std.testing.expectEqual(@as(u64, 0), extra);
    try std.testing.expectEqual(@as(u32, 0), wb_sim_destroy(handles[0]));
    try std.testing.expectEqual(@as(u32, 0), wb_sim_create(1, 0, 0, &handles[0]));
}
