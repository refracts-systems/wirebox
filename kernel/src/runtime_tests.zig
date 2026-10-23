const std = @import("std");
const t = std.testing;
const rt = @import("runtime.zig");
const max = std.math.maxInt(u64);

fn host(world: *rt.Runtime) !u64 {
    return world.topology.add(.device, 1, 0, 0);
}
fn add(world: *rt.Runtime, target: u64, time: u64, amount: u64, interval: u64) !u64 {
    return world.schedule(.{ .target_id = target, .time_ns = time, .amount = amount, .interval_ns = interval });
}

test "runtime C layouts" {
    try t.expectEqual(@as(usize, 48), @sizeOf(rt.graph.Entity));
    try t.expectEqual(@as(usize, 56), @sizeOf(rt.Event));
    try t.expectEqual(@as(usize, 80), @sizeOf(rt.Trace));
    try t.expectEqual(@as(usize, 56), @offsetOf(rt.Trace, "ordinal"));
    try t.expectEqual(@as(usize, 40), @sizeOf(rt.Schedule));
    try t.expectEqual(@as(usize, 24), @sizeOf(rt.Progress));
    try t.expectEqual(@as(usize, 80), @sizeOf(rt.Snapshot));
}

test "graph validates references and endpoint exclusivity; cascade cancels every removed owner" {
    var world = try rt.Runtime.init(0, 16);
    const h1 = try host(&world);
    const h2 = try host(&world);
    const p1 = try world.topology.add(.port, 0, h1, 0);
    const p2 = try world.topology.add(.port, 0, h2, 0);
    try t.expectError(error.NotFound, world.topology.add(.port, 0, 999, 0));
    try t.expectError(error.InvalidArgument, world.topology.add(.port, 0, p1, 0));
    try t.expectError(error.InvalidArgument, world.topology.add(.link, 0, p1, p1));
    try t.expectError(error.InvalidArgument, world.topology.add(.link, 0, h1, p2));
    const link = try world.topology.add(.link, 0, p1, p2);
    try t.expectError(error.Conflict, world.topology.add(.link, 0, p2, p1));
    for ([_]u64{ h1, p1, link, h2, p2 }) |id| _ = try add(&world, id, 10, 1, 1);
    try world.remove(h1);
    try t.expect(world.topology.find(h1) == null and world.topology.find(p1) == null and world.topology.find(link) == null);
    try t.expectEqual(@as(u32, 2), world.topology.count);
    try t.expectEqual(@as(usize, 2), world.queue.len);
    const fresh = try host(&world);
    try t.expect(fresh > link);
    try t.expectError(error.NotFound, add(&world, h1, 20, 1, 0));
    var progress: rt.Progress = .{};
    try world.runUntil(10, 20, &progress);
    try t.expectEqual(@as(u64, 2), progress.processed);
    try t.expectEqual(@as(u64, 0), world.topology.find(fresh).?.counter);
    try world.remove(p2);
    try t.expect(world.topology.find(h2) != null);
    try t.expectEqual(@as(usize, 1), world.queue.len);
    try t.expectError(error.NotFound, world.remove(p2));
}

test "stable heap order matches independent linear selection after arbitrary cancellations" {
    var world = try rt.Runtime.init(11, 256);
    const id = try host(&world);
    var source = rt.Rng{ .state = 31 };
    var expected: [200]rt.Event = undefined;
    var alive: [200]bool = @splat(true);
    for (&expected, 0..) |*event, i| {
        const time = source.below(30);
        const timer = try add(&world, id, time, 1, 0);
        event.* = .{ .id = i + 1, .time_ns = time, .timer_id = timer };
    }
    for (expected, 0..) |event, i| {
        if (i % 3 == 0) {
            try world.cancel(event.timer_id);
            alive[i] = false;
        }
    }
    var ordinal: u64 = 0;
    while (true) {
        var selected: ?usize = null;
        for (expected, 0..) |event, i| {
            if (!alive[i]) continue;
            if (selected == null or event.time_ns < expected[selected.?].time_ns or
                (event.time_ns == expected[selected.?].time_ns and event.id < expected[selected.?].id)) selected = i;
        }
        const index = selected orelse break;
        try t.expect(try world.step());
        var trace: [1]rt.Trace = undefined;
        try t.expectEqual(@as(usize, 1), world.read(ordinal, &trace));
        try t.expectEqual(expected[index].id, trace[0].event.id);
        try t.expectEqual(expected[index].time_ns, trace[0].event.time_ns);
        alive[index] = false;
        ordinal += 1;
    }
    const now = world.now;
    try t.expect(!try world.step());
    try t.expectEqual(now, world.now);
    try t.expectEqual(@as(u64, 133), world.executed);
}

test "same-time recurrence keeps causal IDs and yields at event budget" {
    var world = try rt.Runtime.init(0, 16);
    const id = try host(&world);
    const timer = try add(&world, id, 10, 1, 5);
    _ = try add(&world, id, 10, 10, 0);
    _ = try add(&world, id, 15, 100, 0);
    var progress: rt.Progress = .{};
    try world.runUntil(100, 0, &progress);
    try t.expectEqual(@as(u32, 2), progress.reason);
    try t.expectEqual(@as(u64, 0), world.now);
    try world.runUntil(100, 4, &progress);
    try t.expectEqual(@as(u32, 2), progress.reason);
    try t.expectEqual(@as(u64, 15), world.now);
    try t.expectEqual(@as(u64, 112), world.topology.find(id).?.counter);
    var trace: [4]rt.Trace = undefined;
    try t.expectEqual(@as(usize, 4), world.read(0, &trace));
    for (trace, 1..) |event, expected| try t.expectEqual(@as(u64, expected), event.event.id);
    try t.expectEqual(@as(u64, 1), trace[3].event.cause_id);
    try world.cancel(timer);
    try t.expectError(error.NotFound, world.cancel(timer));
    try world.runUntil(100, 0, &progress);
    try t.expectEqual(@as(u32, 1), progress.reason);
    try t.expectEqual(@as(u64, 100), world.now);
    try t.expect(!try world.step());
    try t.expectError(error.PastTime, world.runUntil(99, 1, &progress));
    try t.expectError(error.PastTime, add(&world, id, 99, 1, 0));
    _ = try add(&world, id, 100, 1, 0);
    try t.expect(try world.step());
}

test "capacity and cancellation reclaim space without consuming IDs on failure" {
    var world = try rt.Runtime.init(0, 0);
    const id = try host(&world);
    var first: u64 = 0;
    for (0..rt.pending_limit) |i| {
        const timer = try add(&world, id, 1, 1, 0);
        if (i == 0) first = timer;
    }
    const snapshot = world.snapshot();
    try t.expectError(error.Capacity, add(&world, id, 1, 1, 0));
    try t.expectEqualDeep(snapshot, world.snapshot());
    try world.cancel(first);
    _ = try add(&world, id, 1, 1, 0);
    for (1..rt.graph.limit) |_| _ = try host(&world);
    const next = world.topology.next_id;
    try t.expectError(error.Capacity, host(&world));
    try t.expectEqual(next, world.topology.next_id);
}

test "overflow is atomic for time, counter, RNG, IDs and execution count" {
    var world = try rt.Runtime.init(0, 16);
    const id = try host(&world);
    const timer = try add(&world, id, max, 1, 1);
    const before = world.snapshot();
    try t.expectError(error.Overflow, world.step());
    try t.expectEqualDeep(before, world.snapshot());
    try t.expectEqual(@as(u64, 0), world.topology.find(id).?.counter);
    try world.cancel(timer);
    world.topology.find(id).?.counter = max;
    const random = try world.schedule(.{ .target_id = id, .time_ns = 1, .amount = 2, .action = 2 });
    const before_random = world.snapshot();
    // SplitMix64 seed 0 first sample is odd, so adding modulo 2 overflows.
    try t.expectError(error.Overflow, world.step());
    try t.expectEqualDeep(before_random, world.snapshot());
    try world.cancel(random);
    world.topology.find(id).?.counter = 0;
    _ = try add(&world, id, 1, 1, 0);
    world.executed = max;
    try t.expectError(error.Overflow, world.step());
    world.next_event_id = max;
    try t.expectError(error.Overflow, add(&world, id, 1, 1, 0));
    world.next_event_id = 10;
    world.next_timer_id = max;
    try t.expectError(error.Overflow, add(&world, id, 1, 1, 0));
    world.topology.next_id = max;
    try t.expectError(error.Overflow, host(&world));
}

test "failed later event exposes successful run prefix; future event remains queued" {
    var world = try rt.Runtime.init(0, 8);
    const id = try host(&world);
    _ = try add(&world, id, 1, max, 0);
    const bad = try add(&world, id, 2, 1, 0);
    _ = try add(&world, id, 4, 0, 0);
    var progress: rt.Progress = .{};
    try t.expectError(error.Overflow, world.runUntil(3, 10, &progress));
    try t.expectEqual(@as(u64, 1), progress.processed);
    try t.expectEqual(@as(u64, 1), progress.now_ns);
    try t.expectEqual(@as(u32, 4), progress.reason);
    try world.cancel(bad);
    try world.runUntil(3, 10, &progress);
    try t.expectEqual(@as(u64, 3), world.now);
    try t.expectEqual(@as(usize, 1), world.queue.len);
}

test "pinned SplitMix64 golden values and explicit state continuation" {
    var rng = rt.Rng{ .state = 0 };
    const golden = [_]u64{ 0xe220a8397b1dcdaf, 0x6e789e6aa1b965f4, 0x06c45d188009454f, 0xf88bb8a8724c81ec };
    for (golden) |value| try t.expectEqual(value, rng.next());
    var copy = rng;
    for (0..20) |_| try t.expectEqual(rng.below(0x8000000000000001), copy.below(0x8000000000000001));
    try t.expectEqual(@as(u64, 0), rng.below(1));
}

test "retention and read cadence do not change state, RNG, IDs or execution order" {
    var baseline: ?rt.Snapshot = null;
    var counter: u64 = 0;
    for ([_]usize{ 0, 1, 7, 256 }) |retention| {
        var world = try rt.Runtime.init(20260117, retention);
        const id = try host(&world);
        const timer = try world.schedule(.{ .target_id = id, .time_ns = 0, .interval_ns = 1, .amount = 1000, .action = 2 });
        var progress: rt.Progress = .{};
        try world.runUntil(299, 300, &progress);
        try world.cancel(timer);
        var trace: [256]rt.Trace = undefined;
        const count = world.read(0, &trace);
        try t.expectEqual(retention, count);
        try t.expectEqual(@as(u64, 300 - retention), world.dropped);
        if (count > 0) {
            try t.expectEqual(@as(u64, 301 - retention), trace[0].ordinal);
            try t.expectEqual(@as(u64, 300), trace[count - 1].ordinal);
            try t.expectEqual(@as(usize, 0), world.read(300, &trace));
        }
        var snapshot = world.snapshot();
        snapshot.dropped = 0;
        snapshot.journal_count = 0;
        if (baseline) |expected| {
            try t.expectEqualDeep(expected, snapshot);
            try t.expectEqual(counter, world.topology.find(id).?.counter);
        } else {
            baseline = snapshot;
            counter = world.topology.find(id).?.counter;
        }
    }
    // Read after each step vs once at the end, with the same seeded event stream.
    var a = try rt.Runtime.init(5, 256);
    var b = try rt.Runtime.init(5, 256);
    const ia = try host(&a);
    const ib = try host(&b);
    for (0..40) |i| {
        _ = try a.schedule(.{ .target_id = ia, .time_ns = 40 - i, .amount = 100, .action = 2 });
        _ = try b.schedule(.{ .target_id = ib, .time_ns = 40 - i, .amount = 100, .action = 2 });
    }
    var incremental: [40]rt.Trace = undefined;
    for (&incremental, 0..) |*entry, i| {
        _ = try a.step();
        try t.expectEqual(@as(usize, 1), a.read(i, @as(*[1]rt.Trace, @ptrCast(entry))));
    }
    var progress: rt.Progress = .{};
    try b.runUntil(40, 40, &progress);
    var batch: [40]rt.Trace = undefined;
    try t.expectEqual(@as(usize, 40), b.read(0, &batch));
    try t.expectEqualDeep(incremental, batch);
    try t.expectEqualDeep(a.snapshot(), b.snapshot());
}

test "serialization delay rounds up with checked wide arithmetic" {
    try t.expectEqual(@as(u64, 5120), try rt.serializationDelay(64, 100_000_000));
    try t.expectEqual(@as(u64, 2_666_666_667), try rt.serializationDelay(1, 3));
    try t.expectEqual(@as(u64, 0), try rt.serializationDelay(0, 1));
    try t.expectError(error.InvalidArgument, rt.serializationDelay(1, 0));
    try t.expectError(error.Overflow, rt.serializationDelay(max, 1));
    try t.expectEqual(@as(u64, 8_000_000_000), try rt.serializationDelay(max, max));
}

test "hardening interleaved schedule cancel step matches linear reference model" {
    for ([_]u64{ 0, 1, 31, 20260117, max }) |seed| {
        var world = try rt.Runtime.init(seed, 1);
        const target = try host(&world);
        var rng = rt.Rng{ .state = seed };
        var model: [96]?rt.Event = @splat(null);
        var expected_counter: u64 = 0;
        for (0..4000) |_| {
            const choice = rng.below(3);
            var free: ?usize = null;
            var first: ?usize = null;
            var count: usize = 0;
            for (model, 0..) |entry, i| {
                if (entry) |e| {
                    count += 1;
                    if (first == null or e.time_ns < model[first.?].?.time_ns or
                        (e.time_ns == model[first.?].?.time_ns and e.id < model[first.?].?.id)) first = i;
                } else free = i;
            }
            if (choice == 0 and free != null) {
                const event_id = world.next_event_id;
                const time = world.now + rng.below(8);
                const timer = try add(&world, target, time, 1, 0);
                model[free.?] = .{ .id = event_id, .time_ns = time, .timer_id = timer };
                count += 1;
            } else if (choice == 1 and first != null) {
                // Delete a different array slot than the min when possible.
                var selected = first.?;
                for (model, 0..) |entry, i| if (entry != null and rng.below(2) == 0) {
                    selected = i;
                    break;
                };
                try world.cancel(model[selected].?.timer_id);
                model[selected] = null;
                count -= 1;
            } else if (first) |i| {
                const expected = model[i].?;
                try t.expect(try world.step());
                try t.expectEqual(expected.id, world.last_event_id);
                try t.expectEqual(expected.time_ns, world.now);
                expected_counter += 1;
                model[i] = null;
                count -= 1;
            } else try t.expect(!try world.step());
            const snapshot = world.snapshot();
            try t.expectEqual(count, world.queue.len);
            try t.expectEqual(@as(u32, @intCast(count)), snapshot.timer_count);
            try t.expectEqual(expected_counter, world.topology.find(target).?.counter);
        }
    }
}
