# Reported simulation architecture

## Methods and limits

The report target is PacketTracer.exe 9.0.1 x64. The initial stage uses PE, string, RTTI MSVC inspection, and resource extraction. The next stage uses the Ghidra 12.1.4 headless project. The function name FUN_* is the analysis label on the binary; Certain method names are reconstructed via RTTI and string anchoring, without PDB.

REA reportedly does not provide a native PE decompiler provider in the original investigation session. Decompilation is done via Ghidra; Frida is used in runtime investigation of document formats. This is not a claim that REA itself decompiles targets.

## Simulation world

| Object | Role | Evidence in the report |
|---|---|---|
| CNetwork / CNetworkFile | Document topology and serialization | RTTI, serializer, parser |
| CWorkspaceList | Add/remove devices and links | Operation name and associated references |
| Device::CDevice | Polymorphic devices | RTTI/vftable, descriptor, factory |
| Port::CPort | Send/receive point | Port class and send/receive labels |
| Link::CLink / CCable | Delayed connection and delivery | RTTI, source-path string, cable timer |
| CProcess | Protocol processing | processSend/processReceive/processNoConnection, higher/lower vectors |
| Traffic::CPdu | In-memory package representation | RTTI, header class, getDecapsulationInfo |
| CFrameInstance | Instance representation for views/animations | RTTI and animated slots |

CNetwork reportedly uses a CDevice* vector at +0xF0/+0xF8 and an ordered connection map at +0x170. This offset is the result of target version analysis, not a layout contract for Wyrlex.

## Scheduler

The addendum reports CScheduler::destructor FUN_141d1d3e0 with a shift-down loop as well as pop_heap/pop_back operations.

| Relative location | Reported content | Interpretation |
|---|---|---|
| +0x18 | std::vector<CNTTimerPointer>, 16 byte element | Binary heap storage for next timer |
| +0x48 / +0x50 | QWaitCondition / QSemaphore | Synchronization primitives; worker lifecycle details still need direct inspection |
| +0x58 | QTimer | Regarding realtime pacing according to reports |
| +0x70 | shared_ptr to registry ordered map | Lookup timer based on ID |

CIpcTimer comparator reportedly sorts primary + secondary time then ID. Heap answers the next timer; registry answers search/deletion based on ID. The registry form does not require event queues to use multimap.

getIpcTimer, FUN_141d22cf0, reportedly performs a lookup and dynamic_cast to CIpcTimer, with a SimulationTimer error or getIpcTimer implementation error if it fails.

The report also mentions multiple inheritance of CIpcTimer, secondary vftable +0x50, as well as CSimulation as QObject and Ipc::CParser. This inheritance detail is not required to copy the scheduler pattern to a new engine.

## Packet processing chain

Common models supported by the report:

1. The sending process forms a PDU.
2. The encapsulation process prepares the link header.
3. Port and link schedule delivery.
4. The destination port hands over the PDU to the receiving process.
5. The process of decapsulating and dispatching to the next protocol.

init() and addEntryToMapTable() connect higher/lower processes. The error string in CHdlcProcess::init and other protocol methods reinforces this structure. The order of each path and all special conditions have not been restored.

## Routing and instrumentation

FUN_14094c530, identified as CRoutingProcess, reportedly contains RIB lookup, local receive, TTL expiry, encapsulation failure, and ICMP generation.

The codes CROUTINGPROCESS_FC_*, CARPPROCESS_FC_*, CMACSWITCHER_FC_*, and CTcpConnection_FC_* help map decisions. Ghidra's stage report mentions 1,404 step codes in all. The existence of a step name does not prove that all transitions or protocol behavior have been reimplemented.

PDUDefinition.json is approximately 183,638 bytes containing approximately 170 display layouts; resource notes mention over 250 implementations of getDecapsulationInfo. View resources are not the same as a complete wire parser implementation.

## Time and GUI

Simulation time is separated from the wall clock. The GUI has paused/stepped and realtime modes. The delay/jitter settings provide evidence of the timing model parameters, but do not reveal the entire formula or random PT distribution.

CBaseSimulationPanel's Event List is a delimited GUI log. FUN_142580fc0/FUN_142582f00 is associated with buffer full, list control, and delay settings. The scheduler queue must be able to continue to be managed separately from the log retention policy.

The Back button and animationDone slot are not enough to prove the restore of the entire state engine. The relationship of a Capture/Forward to a pop scheduler remains partial inference to the original report.

Sources: [Ghidra report](sources/03-ghidra-architecture.txt), [full addendum](sources/REPORT_addendum.original.md), and [evidence ledger](evidence-ledger.md).
