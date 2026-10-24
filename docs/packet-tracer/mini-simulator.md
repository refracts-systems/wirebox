# Mini-simulator: implementation and verification

## Origin of artifact

The prototype is available in C:\Users\ricky\AppData\Local\Temp\opencode\pt_minisim. Main files: ptmini.h, main.cpp, build.bat, and ptmini.exe. Source/executables are not moved to this repository; documentation and verification records are kept.

Source and executables checked on October 19, 2026. Available executables run directly; A rebuild has not been carried out which proves the suitability of the binary to the source. The verification artifact hash is available at [verification/artifact-manifest.json](verification/artifact-manifest.json).

## Scenario

Topology: PC1 — SW1 — R1 — PC2. PC1 is at 10.0.0.0/24 and PC2 is at 10.0.1.0/24. The link in the demo is set at 100 Mb/s with a propagation delay of 1 microsecond.

1. ICMP echo: ARP, MAC learning/flooding, router forwarding, and echo reply.
2. TCP: handshake, simple Nagle buffering, one frame loss, retransmission, then FIN.
3. UDP: sending to registered port, then closed port resulting in ICMP port unreachable.

Application data such as GET /index.html is just a demo payload, not evidence of a full HTTP server implementation.

## Results of local execution

| Metrics | Jitter ON | Jitter OFF |
|---|---:|---:|
| First arrival on SW1, second | 0.0000244705 | 0.00002436 |
| Last event time, seconds | 1.00167 | 1.00163 |
| Event List line | 461 | 461 |
| Record frame arrival | 66 | 66 |
| TCP state transition record | 7 | 7 |
| TCP retransmission record | 1 | 1 |
| Record loses injected frames | 1 | 1 |

The output is stored in [summary.txt](verification/summary.txt) and [selected-trace.txt](verification/selected-trace.txt). This is a scenario demonstration, not a conformance test suite or throughput benchmark.

eventCount() returns the size of eventList_. One scheduler callback can produce multiple traces, so the number of rows should not be labeled as the number of scheduler events executed.

## Checked mechanism in source

The scheduler uses sequential priority_queue (time, seq). Port::send calculates processing delay + transmission time + propagation delay + jitter, then schedules the destination port's receive() call.

runUntil(limit) processes events due to the limit, but this implementation does not advance now() to the limit if there are no events there. The time number above is the last callback time, not automatically the requested horizon duration.

The random generator uses a fixed seed of 20260117. The comments/demo output that mentions jitter ON always being non-reproducible is too broad: with the same seed, input sequence, build, and environment, the run is repeatable. Reproducibility across standard libraries/platforms has not been tested.

## Model limits

- TCP is a teaching model: no SACK, congestion control, or full TIME_WAIT timers; MSS/window and initial sequence simplified.
- Link does not yet model queues, contention, or collisions.
- IPv6, fragmentation, NAT, and full application are not yet available according to the prototype README.
- Realtime mode is available as an enum, but the demo runs stepped mode.
- The similarity of the code name *_FC_* does not prove the similarity of all behavior with PT.

The retransmission timer bug in early versions of the prototype was fixed by arming the timer for SYN/FIN/data and using runUntil. It explains the need for prototype design; does not prove an identical bug in PT.

## Meaning for simulation without hardware

The prototype shows that virtual devices, PDUs, protocol state, and time can run within a local program. The device model is still required. Claims of equivalency with vendor, optical condition, or production hardware require additional model and benchmark testing.

Historical sources: [prototype README](sources/ptmini-README.original.md) and [addendum](sources/REPORT_addendum.original.md).
