# ptmini — a miniature Packet Tracer simulation core

A small, self-contained C++17 program that reproduces the **shape** of Cisco Packet
Tracer's simulation engine. It is meant to be read together with the reverse
engineering notes: every class and every trace line is modelled on a structure
recovered from `PacketTracer.exe` 9.0.1.

It is **not** a clone. It is the skeleton you would write first if you were building a
Packet-Tracer-like network simulator yourself.

```
       10.0.0.0/24            10.0.0.0/24           10.0.1.0/24
   PC1 ────────────── SW1 ────────────── R1 ────────────── PC2
  .2                (L2)            .1        .1            .2
```

## Build and run

```
build.bat            :: MSVC (Visual Studio 2022 Build Tools)
ptmini.exe           :: full trace
ptmini.exe -q        :: summary only
```

or with any C++17 compiler:

```
g++ -std=c++17 -O2 main.cpp -o ptmini
```

## What the run demonstrates

The scenario is executed twice — once with transmission jitter **on** (Packet
Tracer's default) and once **off** — and the timelines are compared.

| phase | what it exercises |
|---|---|
| 1. ICMP echo | ARP resolution + buffering, switch CAM learning and flooding, the router's forwarding decision, ICMP echo request/reply |
| 2. TCP | three-way handshake, Nagle buffering, send-window accounting, an **injected lost frame** recovered by the retransmission timer, a full FIN exchange |
| 3. UDP | a datagram to a bound socket, then to an unbound port, which makes the host generate **ICMP port unreachable** |

Sample trace (jitter on, abridged — real `*_FC_*` step codes throughout):

```
0.000610845 PC1 CTCPCONNECTION_FC_CON_SUCCESS      Three-way handshake complete with 10.0.1.2:80.
0.000685937 PC1 CTCPCONNECTION_FC_ENCAP            TCP segment encapsulated: ACK PSH seq=1001 ... data="GET /index.html".
0.000685937 PC1 CTcpConnection_fc_buffer_data_nagle_alg
                                                   Nagle: "HEAD /logo.png" buffered until the outstanding segment is acknowledged.
0.000763364 PC2 CTcpConnection_fc_receive_data     Data received: "GET /index.html".
0.000837513 PC1 CTCPCONNECTION_FC_TAKE_OUT_FROM_BUFFER
                                                   The window opened; sending buffered data "HEAD /logo.png".
0.000989673 PC1 CPORT_FC_FRAME_DROPPED             Frame lost on port 0 (injected link error).
1.00099     PC1 CTcpConnection_fc_retransmit       Segment not acknowledged; retransmitting (attempt 1).
1.00107     PC2 CTcpConnection_fc_receive_data     Data received: "POST /log".
1.00114     PC1 CTCPCONNECTION_FC_SET_STATE        Connection state = FIN WAIT 1.
1.00121     PC2 CTcpConnection_fc_close_connection The peer sent FIN; state = CLOSE WAIT.
1.00159     PC2 CUDPPROCESS_FC_SEND_ICMP_UNREACHABLE
                                                   ICMP port unreachable sent to 10.0.0.2.
1.00167     PC1 CICMPPROCESS_FC_ICMP_HOST_UNREACHABLE
                                                   ICMP destination unreachable (from 10.0.1.2).
```

Summary of the last run: **461 events, 66 frame arrivals, 7 TCP state transitions,
1 retransmission, 1 injected loss**, virtual time ≈ 1.0017 s.

## Architecture → Packet Tracer evidence

| ptmini | Packet Tracer (recovered) | Evidence |
|---|---|---|
| `Scheduler` | `Simulation::CScheduler` | QObject; a `std::vector<CNTTimerPointer>` maintained as a **binary heap** (push_heap/pop_heap sift-down visible in the destructor `FUN_141d1d3e0`); plus a `std::shared_ptr` timer registry at `+0x70`, a `QWaitCondition` (`+0x48`) / `QSemaphore` (`+0x50`) pair for a worker thread, and a `QTimer` (`+0x58`) for realtime pacing. |
| event ordering by `(time, seq)` | `Simulation::CIpcTimer::vftable[12]` comparator | decompiled: compares primary+secondary time, then an id. |
| `Scheduler::peekTime` / `runUntil` | the "next event" query | needed because a retransmission timer **re-arms itself**; a full drain would never terminate. |
| `Process` chain (`higher_`/`lower_`) | `CProcess::init()` + `addEntryToMapTable()` | error strings recovered verbatim: *"no higher process connection."*, *"There is more than one lower process."* |
| `Device::deliverUp` (IP proto → L4) | `CHostIp` / `CRoutingProcess` "deliver locally" | the router's decision order comes from the `CROUTINGPROCESS_FC_*` enumeration. |
| `Pdu` (layered headers, `encap`/`decap`) | `Traffic::CPdu` / `CPduGroup` / `CFrameInstance` | `getDecapsulationInfo()`, `signal_type`. A PDU is a plain C++ object, never a wire packet. |
| `Port` with transmission time + link delay + **jitter** | `Port::CPort`, `Link::CLink`/`CCable` | GUI options *"Constant Processing, Transmission, and Propagation Delay"* and *"Disable random packet transmission delay jitter"*. |
| `EthProcess` MAC match/DECAP | `CEncapProcess` | `CETHERNETENCAPPROCESS_FC_MAC_MATCH`, `..._MAC_NOT_MATCH`, `..._ENCAP`. |
| `ArpProcess` | `Arp::CArpProcess` + `Arp::CArpLookup` | `CARPPROCESS_FC_*` (38 steps) + `CARPLOOKUP_FC_NOT_FOUND_BUFFER`. |
| `HostIpProcess` | `Ip::CHostIp` | `CHOSTIP_FC_SET_TTL`, `..._DEST_ADD_VALID`, `..._SET_NEXT_HOP`, `..._DEFAULT_GATEWAY_NOT_SET`. |
| `IcmpProcess` | `Icmp::CIcmpProcess` | `CICMPPROCESS_FC_ICMP_ECHO_REQUEST` / `..._ECHO_REPLY` / `..._ICMP_HOST_UNREACHABLE`. |
| `TcpProcess` + `TcpConnection` | `Tcp::CTcpProcess` + `CTcpConnection` | `CTCPPROCESS_FC_*` (7 steps), `CTCPCONNECTION_FC_*` (45 steps), `CTcpConnection_fc_buffer_data_nagle_alg`, `..._retransmit`, `..._close_connection`. |
| `UdpProcess` | `Udp::CUdpProcess` | `CUDPPROCESS_FC_CREATE_SEGMENT` / `_ENCAP` / `_DECAP` / `_NOT_FOUND_PROCESS` / `_SEND_ICMP_UNREACHABLE`. |
| `RoutingProcess` | `Routing::CRoutingProcess` (`FUN_14094c530`, 26,901 bytes) | decision order taken from the `CROUTINGPROCESS_FC_*` enumeration (39 steps). |
| `MacSwitcher` | `Switching::CMacSwitcher` | `CMACSWITCHER_FC_*` (42 steps). |
| bounded event list + `maxEvents_` | `CBaseSimulationPanel` | *"The maximum number of events has been reached."*, *"Buffer Full -- %1"*, *"Clear Event List"*, *"View Previous Events"*. The **GUI log**, separate from the scheduler queue. |
| `Simulation::step()` == "Capture / Forward" | `CSimulation` (`QObject` + `Ipc::CParser`) | one scheduler pop per press. |

## The router decision order (as implemented in `RoutingProcess`)

Taken from the `CROUTINGPROCESS_FC_*` flow-chart enumeration recovered from the binary:

```
CHECK_PORT_IP                  -> packet addressed to this router? deliver locally
CHECK_MULTICAST / BROADCAST    -> drop non-local broadcast
CHECK_TTL_ZERO                 -> TTL <= 1 ? ICMP time exceeded, drop
LOOKUP_TABLE_IN                -> longest-prefix lookup in the RIB
MATCHING_ENTRY / NO_MATCHING   -> found? else USE_DEFAULT_GATEWAY, else ICMP host unreachable
CACHED_CEF_*                   -> fast-path cache
DECREMENT_TTL                  -> TTL -= 1
ENCAP_TO_PACKET                -> re-encapsulate for the egress link
( ARP lookup -> CARPLOOKUP_FC_NOT_FOUND_BUFFER if the next hop is unresolved )
```

## Files

| file | contents |
|---|---|
| `ptmini.h` | the whole engine (header-only): scheduler, PDU, process chain, devices, Ethernet/ARP/IP/ICMP/TCP/UDP, builders |
| `main.cpp` | the `PC1 -- SW1 -- R1 -- PC2` topology and the three phases above |
| `build.bat` | MSVC build script |
| `README.md` | this file |
| `REPORT_addendum.md` | the reverse-engineering findings behind every claim above |

## Reading order

1. `Scheduler` / `Event` — the discrete-event core.
2. `Process` — the protocol stack abstraction.
3. `Device::receive` / `l2Forward` / `ipForward` / `deliverUp` — how a PDU moves.
4. `ArpProcess`, `HostIpProcess`, `RoutingProcess`, `MacSwitcher`, `IcmpProcess`,
   `TcpProcess` (`TcpConnection`), `UdpProcess` — the state machines.
5. `main()` — a topology and three traffic phases.

## Deliberate simplifications

* One PDU is delivered per link; no link queueing, contention or collision model.
* TCP is a teaching model: no SACK, no congestion control, no TIME_WAIT timer,
  fixed MSS/window, and a fixed initial sequence number per side.
* The retransmission timer is armed only for SYN/FIN/data segments (never for a
  bare ACK) — which is what the real `CTcpConnection` does, and a bug in this
  skeleton reproduced the opposite behaviour until it was fixed.
* ARP/ND are merged; IPv6, fragmentation, NAT and the application layer are omitted.
* The RIB is a flat vector with a popcount prefix length, not a radix trie.
* `realtime` mode exists as an enum but the demo always runs stepped.

## Bonus - reading real `.pkt` files offline

`ptmini` does not read `.pkt`; that work is separate and now complete
(`REPORT_addendum.md` §7). The reader is:

```
powershell -NoProfile -ExecutionPolicy Bypass `
  -File ..\pt_frida\pktread.ps1 `
  -File "C:\Program Files\Cisco Packet Tracer 9.0.1\saves\...\foo.pkt" `
  -KsFile ..\pt_frida\ks\ks_universal.bin -OutDir .\out
```

It needs no Packet Tracer process and no passphrase: the format is zlib over a
position-dependent XOR, and newer saves add `CryptoPP::EAX<Twofish>` under a
compile-time-constant key/nonce, so the keystream is a universal constant byte string.

Verified by a 411-file sweep (**293/293 `.pkt`**, **88/88 `.ptd`**, **6/6 `.pka`**, 400
total decoded) and by a byte-exact match with the live process on a 3.5 MB topology
(7,320,358 bytes of XML produced both by PT's own `AuthenticatedDecryptionFilter` and by
the offline reader).

