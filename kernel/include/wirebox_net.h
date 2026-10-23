#ifndef WIREBOX_NET_H
#define WIREBOX_NET_H
#include "wirebox_sim.h"
#define WB_NET_VERSION 1u
#define WB_NET_BYTES 1600u
enum { WB_STATE = 9, WB_ALLOCATION = 10, WB_INVALID_PACKET = 11, WB_UNSUPPORTED = 12 };
typedef struct wb_packet {
    uint64_t version_id, packet_id, parent_version, cause_event;
    uint32_t layer, length;
    uint8_t bytes[WB_NET_BYTES];
} wb_packet;
typedef struct wb_net_trace_record {
    uint64_t ordinal, event_id, time_ns, cause_id, entity, packet_id;
    uint64_t version, prior, transmission, detail_a, detail_b;
    uint32_t kind, reason;
} wb_net_trace_record;
typedef struct wb_transmission {
    uint64_t id, version, from_port, to_port, link, start_ns, arrival_ns, event_id, cause_id;
} wb_transmission;
typedef struct wb_net_summary_record {
    uint64_t now_ns, executed, trace_total, trace_dropped, packet_versions, transmissions, failure_entity;
    uint32_t pending_events, pending_arp, outcome, reason, failed, reserved;
} wb_net_summary_record;
typedef struct wb_field {
    uint32_t code, byte_offset, bit_offset, bit_width;
    uint64_t value;
} wb_field;
/* Same pointer/copy/process-wide lock contract as wirebox_sim.h.
   All configuration precedes one ping or injection. IDs are scoped to the network.
   IPv4/MAC are numeric values; wire byte order is encoded explicitly in Zig. */
wb_status wb_net_create(uint32_t version, uint64_t seed, size_t retention, wb_handle *out);
wb_status wb_net_destroy(wb_handle handle);
wb_status wb_net_add(wb_handle handle, uint32_t kind, uint32_t role, uint64_t a, uint64_t b, uint64_t *out);
wb_status wb_net_port(wb_handle handle, uint64_t port, uint64_t mac, uint32_t ip, uint32_t prefix, uint32_t enabled);
wb_status wb_net_link(wb_handle handle, uint64_t link, uint64_t bitrate, uint64_t propagation);
wb_status wb_net_alias(wb_handle handle, uint64_t device, uint32_t ip);
wb_status wb_net_route(wb_handle handle, uint64_t device, uint64_t port, uint32_t network, uint32_t prefix, uint32_t via);
wb_status wb_net_ping(wb_handle handle, uint64_t port, uint32_t destination, uint32_t ttl);
wb_status wb_net_inject(wb_handle handle, uint64_t port, uint32_t layer, const uint8_t *bytes, size_t len);
wb_status wb_net_run(wb_handle handle, uint64_t horizon, uint64_t budget, wb_progress *out);
wb_status wb_net_summary(wb_handle handle, wb_net_summary_record *out);
wb_status wb_net_packet(wb_handle handle, uint64_t version, wb_packet *out);
wb_status wb_net_tx(wb_handle handle, uint64_t id, wb_transmission *out);
/* Trace capacity <=256. Fields capacity <=40, complete view or WB_CAPACITY/error. */
wb_status wb_net_trace(wb_handle handle, uint64_t after_ordinal, wb_net_trace_record *out, size_t capacity, size_t *count);
wb_status wb_net_fields(wb_handle handle, uint64_t version, wb_field *out, size_t capacity, size_t *count);
#endif
