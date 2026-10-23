const std = @import("std");
pub const graph = @import("graph.zig");
pub const pending_limit = 256;
pub const journal_limit = 256;
pub const Error = graph.Error || error{PastTime};
pub const Action = enum(u32) { add = 1, random_add = 2 };
pub const Event = extern struct {
    id: u64 = 0,
    time_ns: u64 = 0,
    target_id: u64 = 0,
    timer_id: u64 = 0,
    cause_id: u64 = 0,
    amount: u64 = 0,
    action: u32 = 0,
    reserved: u32 = 0,
};
pub const Trace = extern struct {
    event: Event = .{},
    ordinal: u64 = 0,
    before: u64 = 0,
    after: u64 = 0,
};
pub const Schedule = extern struct {
    target_id: u64,
    time_ns: u64,
    interval_ns: u64 = 0,
    amount: u64,
    action: u32 = @backingInt(Action.add),
    reserved: u32 = 0,
};
pub const Progress = extern struct {
    now_ns: u64 = 0,
    processed: u64 = 0,
    // 0 empty (step), 1 horizon reached, 2 budget exhausted, 3 stepped, 4 error.
    reason: u32 = 0,
    reserved: u32 = 0,
};
pub const Snapshot = extern struct {
    now_ns: u64,
    seed: u64,
    rng_state: u64,
    next_event_id: u64,
    next_timer_id: u64,
    executed: u64,
    dropped: u64,
    last_event_id: u64,
    entity_count: u32,
    pending_count: u32,
    timer_count: u32,
    journal_count: u32,
};

pub const Rng = struct {
    state: u64,
    pub fn next(self: *Rng) u64 {
        self.state +%= 0x9e3779b97f4a7c15;
        var z = self.state;
        z = (z ^ (z >> 30)) *% 0xbf58476d1ce4e5b9;
        z = (z ^ (z >> 27)) *% 0x94d049bb133111eb;
        return z ^ (z >> 31);
    }
    pub fn below(self: *Rng, upper: u64) u64 {
        std.debug.assert(upper > 0);
        const threshold = (0 -% upper) % upper;
        while (true) {
            const value = self.next();
            if (value >= threshold) return value % upper;
        }
    }
};

fn less(a: Event, b: Event) bool {
    return a.time_ns < b.time_ns or (a.time_ns == b.time_ns and a.id < b.id);
}

pub const Heap = struct {
    items: [pending_limit]Event = @splat(.{}),
    len: usize = 0,
    fn push(self: *Heap, event: Event) void {
        std.debug.assert(self.len < pending_limit);
        var i = self.len;
        self.len += 1;
        self.items[i] = event;
        while (i > 0) {
            const parent = (i - 1) / 2;
            if (!less(self.items[i], self.items[parent])) break;
            std.mem.swap(Event, &self.items[i], &self.items[parent]);
            i = parent;
        }
    }
    fn down(self: *Heap, start: usize) void {
        var i = start;
        while (2 * i + 1 < self.len) {
            var child = 2 * i + 1;
            if (child + 1 < self.len and less(self.items[child + 1], self.items[child])) child += 1;
            if (!less(self.items[child], self.items[i])) break;
            std.mem.swap(Event, &self.items[i], &self.items[child]);
            i = child;
        }
    }
    fn pop(self: *Heap) Event {
        const event = self.items[0];
        self.len -= 1;
        if (self.len > 0) {
            self.items[0] = self.items[self.len];
            self.down(0);
        }
        return event;
    }
    fn removeTimer(self: *Heap, timer_id: u64) void {
        for (self.items[0..self.len], 0..) |event, i| {
            if (event.timer_id == timer_id) {
                self.len -= 1;
                self.items[i] = self.items[self.len];
                // Rebuild after arbitrary deletion. Bounded O(n), no tombstones.
                var parent = self.len / 2;
                while (parent > 0) {
                    parent -= 1;
                    self.down(parent);
                }
                return;
            }
        }
        unreachable;
    }
};

const Timer = struct { id: u64 = 0, target_id: u64 = 0, interval_ns: u64 = 0 };
pub const Runtime = struct {
    topology: graph.Graph = .{},
    queue: Heap = .{},
    timers: [pending_limit]Timer = @splat(.{}),
    journal: [journal_limit]Trace = @splat(.{}),
    journal_capacity: usize,
    journal_head: usize = 0,
    journal_count: usize = 0,
    now: u64 = 0,
    next_event_id: u64 = 1,
    next_timer_id: u64 = 1,
    seed: u64,
    rng: Rng,
    executed: u64 = 0,
    dropped: u64 = 0,
    last_event_id: u64 = 0,

    pub fn init(seed: u64, retention: usize) Error!Runtime {
        if (retention > journal_limit) return error.InvalidArgument;
        return .{ .seed = seed, .rng = .{ .state = seed }, .journal_capacity = retention };
    }
    fn timer(self: *Runtime, id: u64) ?*Timer {
        if (id == 0) return null;
        for (&self.timers) |*entry| if (entry.id == id) return entry;
        return null;
    }
    pub fn schedule(self: *Runtime, spec: Schedule) Error!u64 {
        if (spec.reserved != 0 or (spec.action != 1 and spec.action != 2) or
            (spec.action == 2 and spec.amount == 0)) return error.InvalidArgument;
        return self.enqueue(spec, 0);
    }
    /// Internal domain events use the same stable heap/timer registry.
    pub fn scheduleDomain(self: *Runtime, target: u64, time: u64, action: u32, token: u64, cause: u64) Error!u64 {
        if (action != 100 and action != 101) return error.InvalidArgument;
        return self.enqueue(.{ .target_id = target, .time_ns = time, .action = action, .amount = token }, cause);
    }
    fn enqueue(self: *Runtime, spec: Schedule, cause: u64) Error!u64 {
        _ = self.topology.find(spec.target_id) orelse return error.NotFound;
        if (spec.time_ns < self.now) return error.PastTime;
        if (self.next_event_id == std.math.maxInt(u64) or self.next_timer_id == std.math.maxInt(u64)) return error.Overflow;
        if (self.queue.len == pending_limit) return error.Capacity;
        for (&self.timers) |*slot| {
            if (slot.id == 0) {
                const id = self.next_timer_id;
                slot.* = .{ .id = id, .target_id = spec.target_id, .interval_ns = spec.interval_ns };
                self.queue.push(.{
                    .id = self.next_event_id,
                    .time_ns = spec.time_ns,
                    .target_id = spec.target_id,
                    .timer_id = id,
                    .amount = spec.amount,
                    .action = spec.action,
                    .cause_id = cause,
                });
                self.next_event_id += 1;
                self.next_timer_id += 1;
                return id;
            }
        }
        return error.Capacity;
    }
    pub fn cancel(self: *Runtime, id: u64) Error!void {
        const entry = self.timer(id) orelse return error.NotFound;
        self.queue.removeTimer(id);
        entry.* = .{};
    }
    pub fn remove(self: *Runtime, id: u64) Error!void {
        try self.topology.remove(id);
        for (&self.timers) |*entry| {
            if (entry.id != 0 and self.topology.find(entry.target_id) == null) {
                self.queue.removeTimer(entry.id);
                entry.* = .{};
            }
        }
    }
    fn append(self: *Runtime, entry: Trace) void {
        if (self.journal_capacity == 0) {
            self.dropped += 1;
            return;
        }
        const index = (self.journal_head + self.journal_count) % self.journal_capacity;
        self.journal[index] = entry;
        if (self.journal_count == self.journal_capacity) {
            self.journal_head = (self.journal_head + 1) % self.journal_capacity;
            self.dropped += 1;
        } else self.journal_count += 1;
    }
    pub fn read(self: *const Runtime, after_ordinal: u64, out: []Trace) usize {
        var count: usize = 0;
        for (0..self.journal_count) |i| {
            const entry = self.journal[(self.journal_head + i) % self.journal_capacity];
            if (entry.ordinal <= after_ordinal) continue;
            if (count == out.len) break;
            out[count] = entry;
            count += 1;
        }
        return count;
    }
    pub fn step(self: *Runtime) Error!bool {
        if (self.queue.len == 0) return false;
        const event = self.queue.items[0];
        if (event.action != 1 and event.action != 2) return error.InvalidArgument;
        const entity = self.topology.find(event.target_id) orelse return error.NotFound;
        const entry = self.timer(event.timer_id) orelse return error.NotFound;
        var next_time: u64 = 0;
        if (entry.interval_ns > 0) {
            next_time = std.math.add(u64, event.time_ns, entry.interval_ns) catch return error.Overflow;
            if (self.next_event_id == std.math.maxInt(u64)) return error.Overflow;
        }
        if (self.executed == std.math.maxInt(u64)) return error.Overflow;
        var next_rng = self.rng;
        const delta = if (event.action == 2) next_rng.below(event.amount) else event.amount;
        const before = entity.counter;
        const after = std.math.add(u64, before, delta) catch return error.Overflow;
        // All fallible work precedes mutation, including the periodic reschedule.
        _ = self.queue.pop();
        self.now = event.time_ns;
        entity.counter = after;
        self.rng = next_rng;
        self.executed += 1;
        self.last_event_id = event.id;
        if (entry.interval_ns > 0) {
            var next = event;
            next.id = self.next_event_id;
            next.time_ns = next_time;
            next.cause_id = event.id;
            self.next_event_id += 1;
            self.queue.push(next);
        } else entry.* = .{};
        self.append(.{ .event = event, .ordinal = self.executed, .before = before, .after = after });
        return true;
    }
    /// Commit scheduling metadata; domain dispatcher owns its own state/error journal.
    pub fn beginDomain(self: *Runtime) Error!Event {
        if (self.queue.len == 0) return error.NotFound;
        const event = self.queue.items[0];
        if (event.action != 100 and event.action != 101) return error.InvalidArgument;
        _ = self.topology.find(event.target_id) orelse return error.NotFound;
        const entry = self.timer(event.timer_id) orelse return error.NotFound;
        if (self.executed == std.math.maxInt(u64)) return error.Overflow;
        _ = self.queue.pop();
        entry.* = .{};
        self.now = event.time_ns;
        self.executed += 1;
        self.last_event_id = event.id;
        self.append(.{ .event = event, .ordinal = self.executed });
        return event;
    }
    pub fn runUntil(self: *Runtime, horizon: u64, budget: u64, progress: *Progress) Error!void {
        progress.* = .{ .now_ns = self.now, .reason = 4 };
        if (horizon < self.now) return error.PastTime;
        while (self.queue.len > 0 and self.queue.items[0].time_ns <= horizon) {
            if (progress.processed == budget) {
                progress.reason = 2;
                return;
            }
            _ = try self.step();
            progress.processed += 1;
            progress.now_ns = self.now;
        }
        self.now = horizon;
        progress.now_ns = horizon;
        progress.reason = 1;
    }
    pub fn snapshot(self: *const Runtime) Snapshot {
        var timers: u32 = 0;
        for (self.timers) |entry| if (entry.id != 0) {
            timers += 1;
        };
        return .{
            .now_ns = self.now,
            .seed = self.seed,
            .rng_state = self.rng.state,
            .next_event_id = self.next_event_id,
            .next_timer_id = self.next_timer_id,
            .executed = self.executed,
            .dropped = self.dropped,
            .last_event_id = self.last_event_id,
            .entity_count = self.topology.count,
            .pending_count = @intCast(self.queue.len),
            .timer_count = timers,
            .journal_count = @intCast(self.journal_count),
        };
    }
};

/// ceil(bytes * 8 * 1e9 / bitrate); wide intermediate avoids false u64 overflow.
pub fn serializationDelay(bytes: u64, bitrate: u64) Error!u64 {
    if (bitrate == 0) return error.InvalidArgument;
    const numerator: u128 = @as(u128, bytes) * 8 * 1_000_000_000;
    const result = (numerator + bitrate - 1) / bitrate;
    if (result > std.math.maxInt(u64)) return error.Overflow;
    return @intCast(result);
}
