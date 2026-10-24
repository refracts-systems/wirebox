#include "wirebox.h"
#include "wirebox_sim.h"
#include "wirebox_net.h"
_Static_assert(sizeof(wb_probe_event) == 80, "C event size differs from Zig/Rust");
_Static_assert(_Alignof(wb_probe_event) == 8, "C event alignment differs");
_Static_assert(offsetof(wb_probe_event, bytes) == 16, "C payload offset differs");
_Static_assert(sizeof(wb_handle) == 8, "handle width differs");
_Static_assert(sizeof(wb_entity) == 48, "entity layout");
_Static_assert(offsetof(wb_entity, kind) == 40, "entity kind offset");
_Static_assert(sizeof(wb_event) == 56, "scheduler event layout");
_Static_assert(offsetof(wb_event, action) == 48, "action offset");
_Static_assert(sizeof(wb_trace) == 80, "trace layout");
_Static_assert(offsetof(wb_trace, ordinal) == 56, "trace ordinal offset");
_Static_assert(sizeof(wb_schedule) == 40, "schedule layout");
_Static_assert(sizeof(wb_progress) == 24, "progress layout");
_Static_assert(sizeof(wb_snapshot) == 80, "snapshot layout");
_Static_assert(offsetof(wb_snapshot, entity_count) == 64, "snapshot count offset");
_Static_assert(sizeof(wb_packet) == 1640, "packet copy layout");
_Static_assert(offsetof(wb_packet, bytes) == 40, "packet bytes offset");
_Static_assert(sizeof(wb_net_trace_record) == 96, "network trace layout");
_Static_assert(sizeof(wb_transmission) == 72, "transmission layout");
_Static_assert(sizeof(wb_net_summary_record) == 80, "network summary layout");
_Static_assert(sizeof(wb_net_noise_record) == 40, "network noise layout");
_Static_assert(sizeof(wb_checkpoint_info) == 56, "checkpoint info layout");
_Static_assert(sizeof(wb_field) == 24, "field layout");
