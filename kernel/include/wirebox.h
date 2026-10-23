#ifndef WIREBOX_KERNEL_H
#define WIREBOX_KERNEL_H
#include <stddef.h>
#include <stdint.h>

#define WB_ABI_VERSION 1u
#define WB_PAYLOAD_LIMIT 64u
#define WB_BATCH_LIMIT 16u
typedef uint64_t wb_handle;
typedef uint32_t wb_status;
enum { WB_OK = 0, WB_INVALID_ARGUMENT = 1, WB_ABI_MISMATCH = 2,
       WB_INVALID_HANDLE = 3, WB_CAPACITY = 4 };

/* Transport probe only: no packet processing, virtual time or simulation yet. */
typedef struct wb_probe_event {
    uint64_t id;
    uint32_t kind;
    uint32_t length;
    uint8_t bytes[WB_PAYLOAD_LIMIT];
} wb_probe_event;

/* Calls touching the registry must be serialized process-wide by the caller. */
uint32_t wb_abi_version(void);
wb_status wb_create(uint32_t abi_version, wb_handle *out_handle);
wb_status wb_destroy(wb_handle handle);
wb_status wb_probe_submit(wb_handle handle, const uint8_t *data, size_t length);
wb_status wb_probe_read(wb_handle handle, wb_probe_event *out, size_t capacity,
                        size_t *out_count);
#endif
