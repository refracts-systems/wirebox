#ifndef WIREBOX_SIM_H
#define WIREBOX_SIM_H
#include "wirebox.h"

#define WB_SIM_VERSION 1u
#define WB_ENTITY_LIMIT 128u
#define WB_PENDING_LIMIT 256u
#define WB_TRACE_LIMIT 256u
/* Status 0..4 follows wirebox.h. */
enum { WB_NOT_FOUND = 5, WB_CONFLICT = 6, WB_PAST_TIME = 7, WB_OVERFLOW = 8 };
enum { WB_DEVICE = 1, WB_PORT = 2, WB_LINK = 3 };
enum { WB_HOST = 1, WB_SWITCH = 2, WB_ROUTER = 3 };
enum { WB_ADD_COUNTER = 1, WB_RANDOM_ADD_COUNTER = 2 };
enum { WB_EMPTY = 0, WB_HORIZON = 1, WB_BUDGET = 2, WB_STEPPED = 3, WB_RUN_ERROR = 4 };

typedef struct wb_entity {
    uint64_t id, owner, a, b, counter;
    uint32_t kind, role;
} wb_entity;
typedef struct wb_event {
    uint64_t id, time_ns, target_id, timer_id, cause_id, amount;
    uint32_t action, reserved;
} wb_event;
typedef struct wb_trace {
    wb_event event;
    uint64_t ordinal, before, after;
} wb_trace;
typedef struct wb_schedule {
    uint64_t target_id, time_ns, interval_ns, amount;
    uint32_t action, reserved;
} wb_schedule;
typedef struct wb_progress {
    uint64_t now_ns, processed;
    uint32_t reason, reserved;
} wb_progress;
typedef struct wb_snapshot {
    uint64_t now_ns, seed, rng_state, next_event_id, next_timer_id;
    uint64_t executed, dropped, last_event_id;
    uint32_t entity_count, pending_count, timer_count, journal_count;
} wb_snapshot;

/* Registry calls are serialized process-wide. All buffers are copied, never retained.
   Outputs must be valid, aligned, non-overlapping and sized for capacity.
   Runtime handles and probe handles use disjoint namespaces. */
uint32_t wb_sim_version(void);
wb_status wb_sim_create(uint32_t version, uint64_t seed, size_t retention, wb_handle *out);
wb_status wb_sim_destroy(wb_handle handle);
/* device: role 1..3, a=b=0; port: role=0,a=owner,b=0; link: role=0,a/b=ports. */
wb_status wb_sim_add(wb_handle handle, uint32_t kind, uint32_t role,
                     uint64_t a, uint64_t b, uint64_t *out_id);
wb_status wb_sim_remove(wb_handle handle, uint64_t id);
wb_status wb_sim_schedule(wb_handle handle, const wb_schedule *spec, uint64_t *out_timer);
wb_status wb_sim_cancel(wb_handle handle, uint64_t timer);
wb_status wb_sim_step(wb_handle handle, wb_progress *out);
wb_status wb_sim_run_until(wb_handle handle, uint64_t horizon, uint64_t budget, wb_progress *out);
wb_status wb_sim_snapshot(wb_handle handle, wb_snapshot *out);
/* Complete graph or WB_CAPACITY. Null out allowed only with capacity zero. */
wb_status wb_sim_entities(wb_handle handle, wb_entity *out, size_t capacity, size_t *out_count);
/* Copy retained executions with ordinal > cursor; never consume; zero capacity is a no-op. */
wb_status wb_sim_trace(wb_handle handle, uint64_t after_ordinal, wb_trace *out,
                       size_t capacity, size_t *out_count);
#endif
