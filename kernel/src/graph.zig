const std = @import("std");
pub const limit = 128;
pub const Error = error{ InvalidArgument, NotFound, Conflict, Capacity, Overflow };
pub const Kind = enum(u32) { device = 1, port = 2, link = 3 };
pub const Entity = extern struct {
    id: u64 = 0,
    owner: u64 = 0,
    a: u64 = 0,
    b: u64 = 0,
    counter: u64 = 0,
    kind: u32 = 0,
    role: u32 = 0,
};

pub const Graph = struct {
    entries: [limit]Entity = @splat(.{}),
    next_id: u64 = 1,
    count: u32 = 0,

    pub fn find(self: *Graph, id: u64) ?*Entity {
        if (id == 0) return null;
        for (&self.entries) |*entity| if (entity.id == id) return entity;
        return null;
    }

    pub fn add(self: *Graph, kind: Kind, role: u32, a: u64, b: u64) Error!u64 {
        var entity: Entity = .{ .kind = @backingInt(kind), .role = role };
        switch (kind) {
            .device => if (role < 1 or role > 3 or a != 0 or b != 0) return error.InvalidArgument,
            .port => {
                if (role != 0 or b != 0) return error.InvalidArgument;
                const owner = self.find(a) orelse return error.NotFound;
                if (owner.kind != @backingInt(Kind.device)) return error.InvalidArgument;
                entity.owner = a;
            },
            .link => {
                if (role != 0 or a == b) return error.InvalidArgument;
                const first = self.find(a) orelse return error.NotFound;
                const second = self.find(b) orelse return error.NotFound;
                if (first.kind != @backingInt(Kind.port) or second.kind != @backingInt(Kind.port)) return error.InvalidArgument;
                for (self.entries) |existing| {
                    if (existing.kind == @backingInt(Kind.link) and
                        (existing.a == a or existing.a == b or existing.b == a or existing.b == b)) return error.Conflict;
                }
                entity.a = a;
                entity.b = b;
            },
        }
        if (self.next_id == std.math.maxInt(u64)) return error.Overflow;
        for (&self.entries) |*slot| {
            if (slot.id == 0) {
                entity.id = self.next_id;
                slot.* = entity;
                self.next_id += 1;
                self.count += 1;
                return entity.id;
            }
        }
        return error.Capacity;
    }

    /// The runtime cancels timers targeting removed IDs immediately after this.
    pub fn remove(self: *Graph, id: u64) Error!void {
        _ = self.find(id) orelse return error.NotFound;
        var removed: [limit]bool = @splat(false);
        for (self.entries, 0..) |entity, i| {
            removed[i] = entity.id == id or (entity.id != 0 and entity.owner == id);
        }
        for (self.entries, 0..) |entity, i| {
            if (entity.kind != @backingInt(Kind.link)) continue;
            for (self.entries, 0..) |endpoint, j| {
                if (removed[j] and (entity.a == endpoint.id or entity.b == endpoint.id)) {
                    removed[i] = true;
                    break;
                }
            }
        }
        for (&self.entries, removed) |*entity, remove_it| {
            if (remove_it) {
                entity.* = .{};
                self.count -= 1;
            }
        }
    }
};
