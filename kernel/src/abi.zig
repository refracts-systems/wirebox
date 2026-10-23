//! Phase-1 bounded transport probe. The registry is not a simulation scheduler.
//! C callers serialize all registry calls; the Rust wrapper enforces this.
const std = @import("std");
comptime {
    _ = @import("sim_abi.zig");
    _ = @import("net_abi.zig");
}
test {
    _ = @import("runtime_tests.zig");
    _ = @import("network_tests.zig");
}
const abi_version: u32 = 1;
const payload_limit = 64;
const batch_limit = 16;
const session_limit = 16;
const generation_bits = @bitSizeOf(u64) - @bitSizeOf(u32) - 2;
// Namespace bits are not generation bits. Retirement prevents an old handle
// becoming valid again; integer wrapping would permit ABA.
const generation_max: u32 = (@as(u32, 1) << generation_bits) - 1;

const Status = enum(u32) { ok, invalid_argument, abi_mismatch, invalid_handle, capacity };
const Event = extern struct {
    id: u64 = 0,
    kind: u32 = 0,
    length: u32 = 0,
    bytes: [payload_limit]u8 = @splat(0),
};
const Context = struct {
    events: [batch_limit]Event = @splat(.{}),
    head: usize = 0,
    count: usize = 0,
    next_id: u64 = 1,
};
const Slot = struct {
    generation: u32 = 1,
    context: ?Context = null,
};
var slots: [session_limit]Slot = @splat(.{});

fn code(status: Status) u32 {
    return @backingInt(status);
}

fn lookup(handle: u64) ?*Slot {
    // Top two bits identify probe/counter/network registries.
    if (handle >> 62 != 0) return null;
    const index: u32 = @truncate(handle);
    const generation: u32 = @intCast(handle >> 32);
    if (index == 0 or index > session_limit or generation == 0) return null;
    const slot = &slots[index - 1];
    if (slot.generation != generation or slot.context == null) return null;
    return slot;
}

export fn wb_abi_version() callconv(.c) u32 {
    return abi_version;
}

export fn wb_create(version: u32, out_handle: ?*u64) callconv(.c) u32 {
    const out = out_handle orelse return code(.invalid_argument);
    out.* = 0;
    if (version != abi_version) return code(.abi_mismatch);
    for (&slots, 0..) |*slot, index| {
        if (slot.context == null and slot.generation != 0) {
            slot.context = .{};
            out.* = (@as(u64, slot.generation) << 32) | @as(u64, index + 1);
            return code(.ok);
        }
    }
    return code(.capacity);
}

export fn wb_destroy(handle: u64) callconv(.c) u32 {
    const slot = lookup(handle) orelse return code(.invalid_handle);
    slot.context = null;
    // Exhausted generations retire a slot instead of aliasing an ancient handle.
    slot.generation = if (slot.generation == generation_max) 0 else slot.generation + 1;
    return code(.ok);
}

export fn wb_probe_submit(handle: u64, data: ?[*]const u8, length: usize) callconv(.c) u32 {
    const slot = lookup(handle) orelse return code(.invalid_handle);
    if (length == 0 or length > payload_limit or data == null) return code(.invalid_argument);
    const context = &slot.context.?;
    if (context.count == batch_limit or context.next_id == std.math.maxInt(u64)) return code(.capacity);
    const tail = (context.head + context.count) % batch_limit;
    var event: Event = .{ .id = context.next_id, .kind = 1, .length = @intCast(length) };
    @memcpy(event.bytes[0..length], data.?[0..length]);
    context.events[tail] = event;
    context.count += 1;
    context.next_id += 1;
    return code(.ok);
}

export fn wb_probe_read(handle: u64, out: ?[*]Event, capacity: usize, out_count: ?*usize) callconv(.c) u32 {
    const count_out = out_count orelse return code(.invalid_argument);
    count_out.* = 0;
    const slot = lookup(handle) orelse return code(.invalid_handle);
    if (capacity > batch_limit or (capacity > 0 and out == null)) return code(.invalid_argument);
    const context = &slot.context.?;
    const count = @min(context.count, capacity);
    for (0..count) |i| out.?[i] = context.events[(context.head + i) % batch_limit];
    context.head = (context.head + count) % batch_limit;
    context.count -= count;
    count_out.* = count;
    return code(.ok);
}

test "ABI layout is fixed and data is copied before the caller changes it" {
    try std.testing.expectEqual(@as(usize, 80), @sizeOf(Event));
    try std.testing.expectEqual(@as(usize, 16), @offsetOf(Event, "bytes"));
    var handle: u64 = 0;
    try std.testing.expectEqual(code(.ok), wb_create(abi_version, &handle));
    defer _ = wb_destroy(handle);
    var data = [_]u8{ 0x12, 0x34 };
    try std.testing.expectEqual(code(.ok), wb_probe_submit(handle, &data, data.len));
    data[0] = 0xff;
    var events: [1]Event = undefined;
    var count: usize = 0;
    try std.testing.expectEqual(code(.ok), wb_probe_read(handle, &events, 1, &count));
    try std.testing.expectEqual(@as(usize, 1), count);
    try std.testing.expectEqualSlices(u8, &.{ 0x12, 0x34 }, events[0].bytes[0..2]);
}

test "destroyed handle cannot access a reused slot" {
    var old: u64 = 0;
    try std.testing.expectEqual(code(.ok), wb_create(abi_version, &old));
    try std.testing.expectEqual(code(.ok), wb_destroy(old));
    var current: u64 = 0;
    try std.testing.expectEqual(code(.ok), wb_create(abi_version, &current));
    defer _ = wb_destroy(current);
    try std.testing.expect(old != current);
    try std.testing.expectEqual(code(.invalid_handle), wb_probe_submit(old, &.{1}, 1));
    try std.testing.expectEqual(code(.invalid_handle), wb_destroy(old));
}

test "session exhaustion is reported and released slots can be used again" {
    var handles: [session_limit]u64 = @splat(0);
    defer for (handles) |handle| {
        if (handle != 0) _ = wb_destroy(handle);
    };
    for (&handles) |*handle| {
        try std.testing.expectEqual(code(.ok), wb_create(abi_version, handle));
    }
    var extra: u64 = 99;
    try std.testing.expectEqual(code(.capacity), wb_create(abi_version, &extra));
    try std.testing.expectEqual(@as(u64, 0), extra);
    try std.testing.expectEqual(code(.ok), wb_destroy(handles[0]));
    handles[0] = 0;
    try std.testing.expectEqual(code(.ok), wb_create(abi_version, &handles[0]));
}

// These tests inject boundary states to avoid a billion create/destroy cycles.
// Save/restore the registry even on assertion failure; no production test hook.
fn expectGenerationBoundary(generation: u32) !void {
    const saved = slots;
    defer slots = saved;
    slots = @splat(.{ .generation = 0 });
    slots[0] = .{ .generation = generation, .context = .{} };
    slots[1] = .{}; // A usable fallback makes retirement observable in create().
    const old = (@as(u64, generation) << 32) | 1;
    const valid = generation > 0 and generation <= generation_max;
    try std.testing.expectEqual(valid, lookup(old) != null);
    const destroyed = wb_destroy(old);
    const actual_generation = slots[0].generation;
    var next: u64 = 0;
    const created = wb_create(abi_version, &next);
    std.debug.print("\ng=0x{x:0>8} old=0x{x:0>16} destroy={d} stored=0x{x:0>8} create={d} next=0x{x:0>16} lookup(next)={}\n", .{
        generation, old, destroyed, actual_generation, created, next, lookup(next) != null,
    });
    try std.testing.expectEqual(if (valid) code(.ok) else code(.invalid_handle), destroyed);
    try std.testing.expectEqual(code(.ok), created);
    // This asserts the design invariants, not merely a preferred +1 policy.
    try std.testing.expect(lookup(next) != null);
    try std.testing.expectEqual(@as(u64, 0), next >> 62);
    if (valid) {
        try std.testing.expect(lookup(old) == null);
        try std.testing.expectEqual(code(.invalid_handle), wb_destroy(old));
        if (generation < generation_max) {
            try std.testing.expectEqual(@as(u32, 1), @as(u32, @truncate(next)));
            try std.testing.expect(next >> 32 > generation);
            try std.testing.expect(next >> 32 <= generation_max);
        } else {
            try std.testing.expectEqual(@as(u32, 0), slots[0].generation);
            try std.testing.expectEqual(@as(u32, 2), @as(u32, @truncate(next)));
        }
    } else {
        // Invalid namespace must not destroy or mutate the injected active slot.
        try std.testing.expectEqual(generation, slots[0].generation);
        try std.testing.expect(slots[0].context != null);
        try std.testing.expectEqual(@as(u32, 2), @as(u32, @truncate(next)));
    }
}

test "handle boundary generation 1" {
    try expectGenerationBoundary(1);
}
test "handle boundary generation 0x3ffffffe" {
    try expectGenerationBoundary(0x3ffffffe);
}
test "handle boundary generation 0x3fffffff" {
    try expectGenerationBoundary(0x3fffffff);
}
test "handle boundary generation 0x40000000" {
    try expectGenerationBoundary(0x40000000);
}
test "handle boundary generation 0x7fffffff" {
    try expectGenerationBoundary(0x7fffffff);
}

test "handle layout rejects zero index generation and every foreign discriminator" {
    const saved = slots;
    defer slots = saved;
    slots = @splat(.{});
    var handle: u64 = 0;
    try std.testing.expectEqual(code(.ok), wb_create(abi_version, &handle));
    try std.testing.expectEqual(@as(usize, 64), @bitSizeOf(@TypeOf(handle)));
    try std.testing.expectEqual(@as(u32, 0x3fffffff), generation_max);
    for ([_]u64{ 0, @as(u64, 1) << 32, 1, (@as(u64, 1) << 32) | (session_limit + 1) }) |invalid| {
        try std.testing.expect(lookup(invalid) == null);
        try std.testing.expectEqual(code(.invalid_handle), wb_destroy(invalid));
    }
    for (1..4) |namespace| {
        const foreign = handle | (@as(u64, namespace) << 62);
        try std.testing.expect(lookup(foreign) == null);
        try std.testing.expectEqual(code(.invalid_handle), wb_destroy(foreign));
        try std.testing.expect(lookup(handle) != null);
    }
}

test "handle stale history stays invalid across reuse and final retirement" {
    const saved = slots;
    defer slots = saved;
    slots = @splat(.{ .generation = 0 });
    slots[0] = .{};
    var history: [256]u64 = undefined;
    for (&history, 0..) |*old, n| {
        try std.testing.expectEqual(code(.ok), wb_create(abi_version, old));
        for (history[0..n]) |stale| try std.testing.expect(lookup(stale) == null);
        try std.testing.expectEqual(code(.ok), wb_destroy(old.*));
        for (history[0 .. n + 1]) |stale| {
            try std.testing.expect(lookup(stale) == null);
            try std.testing.expectEqual(code(.invalid_handle), wb_probe_submit(stale, &.{1}, 1));
        }
    }
    // Jump forward only; emulate the last two lifetimes of this same slot.
    slots[0].generation = generation_max - 1;
    var penultimate: u64 = 0;
    try std.testing.expectEqual(code(.ok), wb_create(abi_version, &penultimate));
    try std.testing.expectEqual(code(.ok), wb_destroy(penultimate));
    var last: u64 = 0;
    try std.testing.expectEqual(code(.ok), wb_create(abi_version, &last));
    try std.testing.expect(lookup(last) != null);
    try std.testing.expectEqual(@as(u64, generation_max), last >> 32);
    try std.testing.expectEqual(code(.ok), wb_destroy(last));
    for (0..32) |_| {
        var out: u64 = 99;
        try std.testing.expectEqual(code(.capacity), wb_create(abi_version, &out));
        try std.testing.expectEqual(@as(u64, 0), out);
        try std.testing.expectEqual(@as(u32, 0), slots[0].generation);
        try std.testing.expect(lookup(penultimate) == null);
        try std.testing.expect(lookup(last) == null);
        for (history) |stale| try std.testing.expect(lookup(stale) == null);
    }
}

test "handle property namespace and reuse over boundary neighborhoods and random generations" {
    const saved = slots;
    defer slots = saved;
    // Fixed seed; deterministic property sampling, no claim of exhaustive u32 fuzzing.
    var random: u64 = 0x123456789abcdef;
    for (0..12000) |n| {
        random = random *% 6364136223846793005 +% 1;
        const generation: u32 = switch (n % 4) {
            0 => @intCast(1 + ((random >> 32) % generation_max)),
            1 => generation_max - @as(u32, @intCast((n / 4) % 512)),
            2 => generation_max + @as(u32, @intCast((n / 4) % 512)),
            else => @truncate(random >> 32),
        };
        slots = @splat(.{ .generation = 0 });
        slots[0] = .{ .generation = generation, .context = .{} };
        const old = (@as(u64, generation) << 32) | 1;
        const valid = generation > 0 and generation <= generation_max;
        const destroyed = wb_destroy(old);
        try std.testing.expectEqual(if (valid) code(.ok) else code(.invalid_handle), destroyed);
        var next: u64 = 99;
        const created = wb_create(abi_version, &next);
        if (valid and generation < generation_max) {
            try std.testing.expectEqual(code(.ok), created);
            try std.testing.expectEqual(@as(u64, 0), next >> 62);
            try std.testing.expect(lookup(next) != null);
            try std.testing.expect(next >> 32 > generation);
            try std.testing.expect(lookup(old) == null);
        } else {
            try std.testing.expectEqual(code(.capacity), created);
            try std.testing.expectEqual(@as(u64, 0), next);
        }
    }
}
