# Packet Tracer 9.0.1 — investigation addendum

Target: `C:\Program Files\Cisco Packet Tracer 9.0.1\bin\PacketTracer.exe` (PE x64, MSVC
RelWithDebInfo, 60,005,312 bytes, no PDB).
Method: Ghidra 12.1.4 headless on the pre-analysed project; MSVC RTTI for class names;
string anchoring to name functions; targeted decompilation. REA's native providers
were **not** available in this session (only `native-macos`, `rea-artifact-graph`,
`rea-dotnet-static`), so the equivalent Ghidra project was used instead — noted as a
limitation.

Everything below is either **confirmed** (decompiler output / file bytes shown),
**strong inference** (consistent evidence, no direct proof) or **unknown**.

---

## 1. The scheduler — resolved

I previously reported a discrete-event scheduler but left the container and the
advance loop unknown. The CScheduler destructor settles it.

**`Simulation::CScheduler::~CScheduler` = `FUN_141d1d3e0` (589 bytes) — CONFIRMED**

```
_Memory_00 = *(longlong **)(param_1 + 0x18);      // vector<CNTTimerPointer> {begin,end}
lVar6 = *_Memory_00;
if (lVar6 != _Memory_00[1]) {
  do {
     ... heap sift-down over 16-byte elements ...
     cVar4 = (**(code **)*puVar5)(puVar5, puVar5 + -2);   // virtual comparator
     lVar9 = (longlong)(uVar7 - 1) >> 1;                  // parent index
     puVar5 = (undefined8 *)((lVar8 * 2 + 2) * 0x10 + lVar2); // 2i+2 child
     ...
     FUN_141d1ca60(_Memory_00[1] + -0x10);                // destroy last
     _Memory_00[1] = _Memory_00[1] + -0x10;               // pop_back
  } while (...);
}
```

* Element stride is `0x10` = 16 bytes (a vftable pointer + an 8-byte payload) in a
  `std::vector` at `CScheduler+0x18`.
* The loop is textbook `std::pop_heap` (sift the root's replacement down, comparing
  `2i+1`/`2i+2` children through a **virtual** `operator<`) followed by `pop_back`.

=> **CONFIRMED: the pending-event set is a binary heap over a `std::vector<CNTTimerPointer>`**,
ordered by `CNTTimerPointer::operator<` — which is the `(time, id)` comparison already
recovered from `Simulation::CIpcTimer::vftable[12]` (`FUN_141d1ca70`).

The same destructor also shows the rest of the layout:

```
param_1[0x10] != 0  -> QTimer::stop((QTimer *)(param_1 + 0x58));
                       ...destruct...
                       QTimer::~QTimer(param_1 + 0x58);
                       QSemaphore::~QSemaphore(param_1 + 0x50);
                       QWaitCondition::~QWaitCondition(param_1 + 0x48);
piVar3 = *(int **)(param_1 + 0x70);
   LOCK(); iVar1 = *piVar3; *piVar3 = *piVar3 - 1; UNLOCK();
   if (iVar1 == 1) { FUN_141d1ce60(mem+8, mem+8); free(mem); }   // shared_ptr release
```

| offset | member | reading |
|---|---|---|
| `+0x18` | `std::vector<CNTTimerPointer>` (heap) | ordered pending events |
| `+0x48` | `QWaitCondition` | worker-thread handshake |
| `+0x50` | `QSemaphore` | worker-thread handshake |
| `+0x58` | `QTimer` | *realtime* pacing (a real Qt timer) |
| `+0x70` | `std::shared_ptr<...>` | timer registry, looked up by id |

=> **CONFIRMED: CScheduler runs a background thread** (QSemaphore + QWaitCondition) and
keeps a **QTimer** for realtime mode. **CONFIRMED: there are two containers** — the heap
ordered by time for the next-to-run event, and a shared-pointer registry used to find a
timer by identity.

**`CScheduler::removeIpcTimer` = `FUN_141d1ed90` — CONFIRMED**

```
plVar1 = (longlong *)(param_1 + 0x70);
lVar2 = *plVar1;
FUN_1402a3200(lVar2 + 8, local_70);            // tree lookup (node, +0x19 Isnil byte)
cVar3 = FUN_1402a4130(lVar2 + 8, param_2);     // comparator
...
QString::QString(local_70,"CScheduler::removeIpcTimer delete");   // QDebug warning
FUN_141d1eb00(plVar1, param_2);                // erase
```
plus `*(int *)(DAT_14384a180 + 0x20) += 1` (a global counter). The `DATA_...`-style
node with an `Isnil` byte at `+0x19` and `+0x08`/`+0x30` child links is the MSVC
red-black tree node. => the `+0x70` registry is an **ordered map/set keyed by timer id**.

**`Simulation::CSimulation::getIpcTimer` = `FUN_141d22cf0` — CONFIRMED**

```
lVar2 = __RTDynamicCast(param_1, 0, &Ipc::CParser::RTTI_Type_Descriptor);
...
uVar5  = FUN_141d1e650();                      // scheduler singleton
lVar2  = FUN_141d1e5c0(uVar5, &local_c8);      // look up the timer
if (lVar2 == 0) { FUN_140134b90(&local_c8,"SimulationTimer");
                  _CxxThrowException(...); }   // throws "SimulationTimer"
plVar1 = (longlong *)(lVar2 + 0x50);
uVar3  = (**(code **)(*plVar1 + 0x10))(plVar1, param_2, param_3, 0);   // virtual dispatch
...
throw "getIpcTimer implementation error";      // if the object is not a CIpcTimer
```

=> **CONFIRMED: timers are obtained by name through a lookup that can throw
`"SimulationTimer"`, and the returned object is validated by `dynamic_cast` to
`Simulation::CIpcTimer`.**

**`Simulation::CIpcTimer` — CONFIRMED multiple inheritance**

```
Simulation::CIpcTimer::vftable[0] = FUN_141d1b0b0:
  *param_1 = Simulation::CIpcTimer::vftable;
  param_1[10] = Simulation::CIpcTimer::vftable;   // SECONDARY vftable at +0x50
  FUN_1419aa890();                                 // base construction
  *param_1 = CTimer<Simulation::CIpcTimer>::vftable;
```
The base-class symbol is literally `CTimer<Simulation::CIpcTimer>::vftable`, so
**`CIpcTimer : CTimer<CIpcTimer>`** (and `CTimer<T>` is one of 118 instantiations in
the RTTI dump). The primary vftable's slots 16–19 are `CScheduler`'s entries, so
**`CIpcTimer` also inherits `CScheduler`**.

`CIpcTimer::vftable[4]` (`FUN_141d1c430`, 369 B) constructs an MSVC red-black tree:
`FUN_1427b496c(0xa8)` allocates a 0xA8-byte node, the node points at itself,
`*(undefined2 *)(lVar7 + 0x18) = 0x101` sets the header/Isnil flags, then the usual
min/max fixups. => **a `std::map` lives inside each CIpcTimer instance.**

`CIpcTimer::vftable[5]` (`FUN_141d1c5b0`, 727 B) is a **factory dispatch**: it builds a
`std::string` key, searches the global RB-tree `DAT_14384a160` with `memcmp`, and if a
node is found calls the function pointer stored in the node at `+0xA0`:
`uVar7 = (*(code *)plVar11[0x14])(param_1, param_2, param_3, param_4);`
otherwise `FUN_1419933e0`. => **CONFIRMED: a string-keyed registry of creation
handlers** (the mechanism behind `getIpcTimer` by name).

---

## 2. `CSimulation` and `CNetwork`

**`Simulation::CSimulation` — CONFIRMED**

* vftable[0] `FUN_1400f6570` = `metaObject()` returning `&DAT_1429b3170` → it is a `QObject`.
* vftable[1] `FUN_1401111b0` = `qt_metacast`:
  * `"Simulation::CSimulation"` → itself
  * `"Ipc::CParser"` → **`param_1 + 0x10`** (secondary base at offset 0x10)
  => **`CSimulation : QObject, Ipc::CParser`**.
* vftable[2] `FUN_140103360` = `qt_metacall`: `QMetaObject::activate(param_1, &DAT_1429b3170, 0, 0)`
  → **one signal**, plus a bool getter `FUN_141d26fc0`.

So `CSimulation` is a small QObject facade (one signal, a few properties) over the
per-topology `Ipc::CParser` document — consistent with the earlier finding that
`CNetwork::vftable[1]` serialises the topology into the parser tree.

**`CNetwork` — CONFIRMED (container layout)**

`CNetwork::vftable[1]` = `FUN_1419c0f60` (2,278 B) builds the save document:

* `*(longlong *)(param_1 + 0xf0)` / `+0xf8` are `begin`/`end` of a
  `std::vector<CDevice *>` — the **device list** (the loop calls
  `(**(code **)(**(longlong **)(lVar5 + uVar13 * 8) + 0x58))()`, a virtual returning a
  **device-type code compared against `0x2d`**).
* `param_1 + 0x170` holds an RB-tree (`_Isnil` byte at `+0x19`, `[1]`/`[2]` child links)
  iterated in order → an ordered map of connections.
* Child containers at `+0x48`, `+0x90`, `+0xa8`, `+0xc0`, `+0xd8`, `+0x180`.
* Translation keys: `CNETWORK_NAME_NETWORK`, `CNETWORK_NAME_NETWORK_RESOURCE`,
  `CNETWORK_NAME_REMOTE_NETWORKS`, `CSHAPETEST_NAME_ENCIRCLE_HEAD`, `Instruction`,
  `User Note`.
* Calls into JS: `resourceManager.getGradingNodes()`.

`CNetwork::vftable[2]` = `FUN_1419ba820` (1,312 B) is the merge/diff path: it walks two
parser trees in parallel, matches nodes by translated name
(`CNETWORK_NAME_*`, `"Encircling Tests"`), and requires identical RTTI before calling
`(**(code **)(*(longlong *)(lVar8 + 0x48) + 0x10))(...)` to merge — i.e. topology
merge/compare by node name + runtime type.

---

## 3. Event List vs scheduler queue — resolved

`FUN_142580fc0` is **not** engine code. It belongs to the GUI class `CBaseSimulationPanel`:

```
QMessageBox::setWindowTitle(..., "Buffer Full -- %1");
"The maximum number of events has been reached. \nYou may clear the event list and
 continue from where you left off or adjust the filters to view previous events."
addButton(..., "Clear Event List", 5);
addButton(..., "View Previous Events", 6);
"Buffer Full,\nClearing Events" / "Buffer Full,\nView Previous Events"
```

and `FUN_142582f00` is that panel's setup, defining the widget text:

```
QCoreApplication::translate(..., "CBaseSimulationPanel", "Event List");
QCoreApplication::translate(..., "CBaseSimulationPanel",
   "List of PDU Columns Visibility Time(seconds) Last Device At Device Type");
"Clear Event List" / "Reset Simulation" / "Constant Delay"
"Constant Processing, Transmission, and Propagation Delay"
"Disable random packet transmission delay jitter"
"Captured To:\n\n(no captures)"
```

=> **CONFIRMED: the Simulation Event List is a bounded GUI log with its own buffer and
overflow dialog. It is a separate structure from `CScheduler`'s heap.** The four
columns are exactly `Time(seconds) | Last Device | At Device | Type`. Delay is
constant unless "Disable random packet transmission delay jitter" is unchecked — i.e.
**the engine adds a random jitter to transmission delay by default**, and this option
turns it off. The panel is Qt **Widgets** (`QTabWidget`, `QTableWidget`, `QLabel`,
`QMessageBox`), not QML.

---

## 4. The `.pkt` file format and its encryption

**CONFIRMED — all activity files are encrypted.**

I read the first 16 bytes of 299 `.pkt`/`.pka` files under
`...\Cisco Packet Tracer 9.0.1\saves`:

```
   37303  59 BF F4 06 11 7B 86 A6 26 93 E7 C5 2C E0 8F A9   HotSwappablePower.pkt
  146418  29 74 3C D3 9C 35 87 93 72 2D D1 F8 B5 74 CA DA   tcp_test_app.pkt
    8240  86 CC C2 6C 0F F9 65 57 2C C0 07 4A B5 A7 07 1B   autoconfig_ipv6...pkt
   ...
clear-like headers: 2   encrypted/random headers: 297
```

No zlib (`78 9C/01/DA`), gzip (`1F 8B`), ZIP (`50 4B`) or XML (`3C`) magic anywhere,
and the two "clear-like" hits are coincidental first bytes (they fail immediately
after). `bin\PT.conf` is encrypted the same way.

**CONFIRMED — the cipher is Crypto++ `DataEncryptorWithMAC`/`DataDecryptorWithMAC`.**

RTTI instantiations present in the binary:

```
?AV?$DataEncryptor@VRijndael@CryptoPP@@VSHA256@2@U?$DataParametersInfo@$0BA@$0BA@$0CA@$07$0JME@@2@@CryptoPP@@
?AV?$DataEncryptorWithMAC@VRijndael@CryptoPP@@VSHA256@2@V?$HMAC@VSHA256@CryptoPP@@@2@U?$DataParametersInfo@...@@CryptoPP@@
?AV?$DataDecryptor@VRijndael@CryptoPP@@VSHA256@2@U?$DataParametersInfo@...@@CryptoPP@@
?AV?$DataDecryptorWithMAC@VRijndael@CryptoPP@@VSHA256@2@V?$HMAC@VSHA256@CryptoPP@@@2@U?$DataParametersInfo@...@@CryptoPP@@
?AV?$BlockCipherFinal@$0A@VEnc@Rijndael@CryptoPP@@@CryptoPP@@              // AES encrypt
?AV?$BlockCipherFinal@$00VDec@Rijndael@CryptoPP@@@CryptoPP@@               // AES decrypt
?AV?$CipherModeFinalTemplate_CipherHolder@...VCBC_Encryption@2@@CryptoPP@@ // CBC
?AV?$CipherModeFinalTemplate_CipherHolder@...VCBC_Decryption@2@@CryptoPP@@
?AV?$CMAC@... / ?AV?$HMAC@VSHA256@CryptoPP@@@CryptoPP@@
```

Crypto++ exception strings are embedded:
`"DataDecryptor: cannot decrypt message with this passphrase"` (58 bytes) and
`"DataDecryptorWithMAC: MAC check failed"` (38 bytes). They are built by two small
exception constructors — `CryptoPP::KeyBadErr` (`FUN_141d5c710`, 312 B) and
`CryptoPP::MACBadErr` (`FUN_141d5c880`, 285 B) — both deriving from
`CryptoPP::DataDecryptorErr`.

Crypto++'s own source paths are in the binary too:

```
C:\jenkins\workspace\SGME\SGME_PT901\M_SGME_PT901_Release_Win64\engine\util\cryptopp\*.cpp
```

=> Rijndael/AES in **CBC** mode with **HMAC-SHA256** authentication, driven by
Crypto++'s password-based `DataEncryptorWithMAC` family. Under Crypto++ these classes
**derive both the AES key and the IV deterministically from the passphrase**, which is
why a wrong passphrase yields either "cannot decrypt ... with this passphrase" (padding)
or "MAC check failed".

**CONFIRMED — the decryption routine (`FUN_141d5d870`, 895 B).** This is the function
that turns a passphrase into a usable AES context. Its decompiled body shows, in order:

```c
_SHA256_ = FUN_141d58ed0(&sha, 1);                                 // construct SHA256
sha.vftable = CryptoPP::SHA256::vftable;
FUN_141d700f0(&sha, *(char **)(this + 0xb8), *(size_t *)(this + 0xb0)); // SHA256.Update(passphrase, len)
FUN_141d700f0(&sha, salt, 8);                                      // SHA256.Update(salt, 8)   <-- 8-byte salt
(*(sha.vt + 0x20))(&sha, digest32, 0x20);                          // SHA256.Final -> 32 bytes

key  = FUN_141d546a0(0x10);                                        // 16-byte AES key   <-- AES-128
iv   = FUN_141d546a0(0x10);                                        // 16-byte IV

FUN_141d5b8d0(pass, passLen, salt, 8);                             // KDF init(passphrase, salt)
uVar5 = (*(*(this + 0xc0) + 0x40))(this + 0xc0);                   // KDF parameter (itr / derived length)
FUN_141d5a9e0(this + 0xc0, key, 0x10, iv);                         // KDF -> derive key(16) + IV(16)

cipher = FUN_1427b496c(0xe0);                                      // 224-byte AES-CBC object
cipher = FUN_141d61230(cipher, this + 200, 0, 5);                  // construct (mode selector 5)
(*(cipher.vt + 0x30))(cipher, key, 0x10, 0);                       // SetKey(key, 16)
(*(cipher.vt + 0x90))(cipher, iv, 0x10);                           // SetIV(iv, 16)
FUN_141d646a0(this, cipher);                                       // store the cipher on the decryptor

if (FUN_141d702b0(digest32, digest32 + 0x10) == 0) {               // verify
    *(this + 0x98) = 2;
    if (*(this + 0x1a0)) throw KeyBadErr("DataDecryptor: cannot decrypt ... passphrase");
} else *(this + 0x98) = 1;
```

Reading the constants:

* the passphrase is carried on the object as **`char* at +0xb8` and `size_t length at
  +0xb0`** — i.e. it is supplied by the caller, not compiled in;
* the salt is **8 bytes** and is passed in as an argument (so it is per-file data);
* the derived AES key is **16 bytes** => **AES-128**, not AES-256 — the `Rijndael`
  template instantiation is shared but the usable key length here is 128 bits;
* key *and* IV are both derived, which is why the wrong passphrase fails cleanly;
* failures raise `CryptoPP::KeyBadErr` / `CryptoPP::MACBadErr`.

**CONFIRMED — the passphrase is (at least optionally) a user password, and PT asks the
GUI/JS layer for it.** `FUN_1419aba00` (≈340 callers — it is the script/IPC bridge)
embeds:

```
Activity file requires password, call ipc.appWindow().getActiveFile().confirmPassword(passwordString)
```

and the JS bridge name `confirmPassword`, handled in the GUI at `FUN_1423d7f10`.
PT's own file classes are `pt::CActivityFile` plus `File::CFile`, `File::CFileManager`,
`File::CFileSystem`, `Ui::BaseDeviceFileManager`.

**UNKNOWN / not completed**

* The passphrase used for activity files that are *not* password-protected — a built-in
  default must exist (because `bin\PT.conf` is encrypted and is readable with no user
  prompt), but it was **not located**. The passphrase is passed in from outside the
  decryptor (`+0xb0`/`+0xb8`), so it lives in whatever PT code calls
  `FUN_141d5d870`/`FUN_141d5dd70`, which was not reached in this pass.
* The exact KDF internals (`FUN_141d5b8d0` / `FUN_141d5a9e0` — iteration count and
  whether this is Crypto++'s `RandomPool` scheme or a PBKDF2-style derivation) were
  **not decompiled**; only the derivation *interface* and the output sizes (16-byte key,
  16-byte IV) are confirmed.
* Whether a `.pkt` can be decrypted offline without user input — **not demonstrated**.

What *is* established is enough to attack it: the scheme, cipher/mode/MAC, the exact
key/IV sizes, the 8-byte salt, the source path, and the three failure modes
("cannot decrypt message with this passphrase", "MAC check failed", and the decoy
string used by the script bridge).

---

## 4b. The key derivation - RESOLVED (decompiled)

> **⚠ CORRECTION - read §7 first.** The Crypto++ `DataDecryptor<Rijndael,SHA256,...>`
> cluster analysed in §4b/**§4c below is real, and the decompilation is correct, but it
> does NOT decrypt a `.pkt` document.** Live instrumentation shows that no
> `DataDecryptor*` function is ever entered while a `.pkt` is opened; that cluster serves
> the `Ipc::CParser` *encrypted-attribute* helper (base64 attribute + passphrase stored
> beside it), a different subsystem. The document path - which needs **no passphrase at
> all** - is the `EAX<Twofish>` + zlib pipeline fully recovered in **§7**.

The whole crypto cluster sits at `0x141d5xxx`-`0x141d7xxxx`: this is Crypto++ compiled
directly into `PacketTracer.exe` (the same `.rdata` region carries
`...\engine\util\cryptopp\donna_32.cpp`). Three functions carry the keying.

### `FUN_141d5c070` - the Crypto++ decryptor constructor (340 B)

It installs a vftable whose RTTI descriptor was read straight out of `.rdata`:

```
vftable base VA 0x142BA40A8
class  .?AV?$DataDecryptor@VRijndael@CryptoPP@@VSHA256@2@
         U?$DataParametersInfo@$0BA@$0BA@$0CA@$07$0JME@@2@@CryptoPP@@
    -> CryptoPP::DataDecryptor<Rijndael, SHA256,
                                 DataParametersInfo<16, 16, 32, 7, 2500>>
```

(The MSVC literal alphabet maps `A`->0 with positional base-16 digits, so `$0BA@`=16,
`$0CA@`=32, `$0JME@`=0x9C4=**2500** - a reading independently corroborated by the
`0x9c4` literal sitting at the KDF call site, see below.)

The only two references to that vftable base are at `141d5c14e` and `141d5c155`, both
inside `FUN_141d5c070`, so that function *is* the constructor.

### `FUN_141d5ba10` (1,033 B) - the KDF core

It is **not PBKDF2 and not HMAC-based**. It is Crypto++'s **`Mash`** construction,
identified by its own verbatim error string in `.rdata`:

```
02ba2df8  HMAC(
02ba2e00  Mash: output length too large
02b9ad50  RoundUpToMultipleOf: integer overflow
```

The decompiled algorithm, exactly:

```c
/* FUN_141d5ba10(input, inputLen, out, outLen, iterations) */
_Size = RoundUpToMultipleOf32(outLen);      /* outLen is 32 here            */
_Src = malloc(_Size); _Dst = malloc(_Size);

local_170 = CryptoPP::SHA256::vftable;      /* one SHA256 context, reused   */
FUN_141d748b0(...);                         /* Restart                      */

/* pass 1 - one 32-byte block per counter value */
for (off = 0; off < outLen; off += 0x20) {
    local_1c8[0] = (byte)(off >> 8);        /* 2-byte big-endian counter    */
    local_1c7    = (byte)(off);
    SHA256_Update(&h, local_1c8, 2);
    SHA256_Update(&h, input, inputLen);
    n = SHA256_DigestSize();                /* 32                           */
    SHA256_Final(&h, _Src + off);
}

/* passes 2..iterations - re-hash the whole buffer each time */
for (p = 1; p < iterations; ++p) {
    memcpy(_Dst, _Src, _Size);
    for (off = 0; off < _Size; off += 0x20) {
        SHA256_Update(&h, BE16(off), 2);
        SHA256_Update(&h, _Dst, _Size);
        SHA256_Final(&h, _Src + off);
    }
}
memcpy(out, _Src, outLen);
/* every buffer is zeroised with explicit memset loops before free() */
```

i.e.

```
block_i    = SHA256( bigendian16(32*i) || DATA )     for each 32-byte block
then the whole buffer is re-hashed (iterations - 1) more times
out[0..32) = the buffer
```

### `FUN_141d5b8d0` (309 B) - the key/IV split

```c
/* FUN_141d5b8d0(pass, passLen, salt, saltLen, iterations, outKey, outIV) */
buff = malloc(passLen + saltLen);
memcpy(buff,           pass, passLen);
memcpy(buff + passLen, salt, saltLen);      /* input = passphrase || salt  */
m = malloc(0x20);
FUN_141d5ba10(buff, passLen + saltLen, m, 0x20, iterations);
*outKey = m[0]; outKey[1] = m[1];           /* bytes  0..15 -> AES key     */
*outIV  = m[2]; outIV[1]  = m[3];           /* bytes 16..31 -> CBC IV      */
```

### `FUN_141d5d870` (895 B) - the decryptor's initialiser

This is where every number becomes concrete:

```c
FUN_141d5d870(this, saltPtr, sink)
{
    /* passphrase lives on the object at +0xb8 (ptr) / +0xb0 (len) */
    SHA256 h; h.Update(this[0xb8], this[0xb0]);   /* passphrase          */
              h.Update(saltPtr, 8);               /* 8-byte salt         */
    digest = h.Final();                           /* 32 bytes            */

    key = malloc(0x10);  iv = malloc(0x10);
    local_1e8 = 0x9c4;                            /* >>> ITERATIONS = 2500 */
    FUN_141d5b8d0(this[0xb8], this[0xb0], saltPtr, 8);   /* -> key, iv    */

    aes = new (0xe0) AES-CBC object;              /* FUN_141d61230       */
    aes->vftable[0x30](aes, sink, 0x10, 0);       /* SetKey(.., 16)      */
    aes->vftable[0x90](aes, digest + 0x10, 0x10); /* SetIV (16 bytes)    */
    FUN_141d646a0(this, aes);                     /* attach              */

    ok = FUN_141d702b0(digest, digest + 0x10);    /* 16-byte compare     */
    if (!ok) { this[0x98] = 2; if (this[0x1a0]) throw CryptoPP::KeyBadErr; }
    else     { this[0x98] = 1; }
}
```

Facts locked in by this function:

| parameter | value | how it is known |
|---|---|---|
| cipher | Rijndael / AES | RTTI class name |
| mode | CBC | `SetIV` plus `DataParametersInfo` BLOCKSIZE |
| key length | **16 bytes (AES-128)** | `malloc(0x10)` and `SetKey(...,0x10)` |
| IV length | **16 bytes** | `malloc(0x10)` and `SetIV(...,0x10)` |
| salt length | **8 bytes** | `h.Update(saltPtr, 8)`, `FUN_141d5b8d0(..., 8)` |
| iterations | **2500** | the literal `0x9c4` at the call site (and `$0JME@` in the RTTI name) |
| integrity | SHA-256 over the processed data, compared as two 16-byte halves (`FUN_141d702b0`) | the final compare |
| failure | `CryptoPP::KeyBadErr` -> *"DataDecryptor: cannot decrypt message with this passphrase"* | exception ctor `FUN_141d5c710` |

**So the earlier description - "SHA-256 of passphrase+salt producing a 16-byte key and a
16-byte IV" - was only half right.** The digest of the passphrase *is* computed, but it
serves as the **verification value**; the key and IV come from the **2500-iteration
`Mash`** derivation of `passphrase || salt`.

## 4c. Where the passphrase comes from - mechanism known, value not

`FUN_141d5dd70` (9 bytes) is a virtual thunk:

```
4C 8D 42 08      lea  r8, [rdx+8]
E9 F7 FA FF FF   jmp  FUN_141d5d870        ; tail call, salt = rdx
```

It is **slot 54 of the `DataDecryptor<...>` vftable** - a byte search over the whole
image found exactly one pointer to it, at `.rdata 0x142BA4258`.

Walking the reverse call graph with raw *references-to-entry-point* (Ghidra's own xref
database is incomplete on this binary, so `getCallingFunctions` returned nothing and the
references had to be enumerated directly) gives:

```
FUN_142445ea0  (9,705 B)
   -> FUN_1424448a0 / FUN_142444b40 / FUN_14244fa60 / FUN_142448570 / FUN_1424488a0
        -> FUN_141d7a5e0   (464 B)
             -> FUN_141d5c070    the DataDecryptor ctor
```

`FUN_141d7a5e0` is unambiguous about the interface:

```c
/* FUN_141d7a5e0(QByteArray *out, QByteArray *ciphertext, QString *passphrase) */
obj  = new (0xb0);                                   /* the decryptor object */
sink = new (0x20); *sink = StringSinkTemplate<std::string>::vftable;
sink[3] = &plaintextString;                          /* output goes here     */
FUN_141d5c070(obj, passphrase->data(), passphrase->size(), sink);   /* passphrase! */
src  = SourceTemplate<StringStore>(ciphertext->constData(), ciphertext->size());
*out = QByteArray(plaintextString.data(), plaintextString.size());
```

and its caller `FUN_1424448a0` shows *where that `QString` comes from*:

```c
FUN_1424448a0(node /* Ipc::CParser */, flag, parserNode)
{
    pass = FUN_1419923e0(parserNode, &x, 1);      /* child #1 = the passphrase */
    val  = FUN_1419923e0(parserNode, &y, 0);      /* child #0 = the ciphertext */
    if (FUN_141987bf0(flag) == 0) plaintext = "";                  /* not encrypted */
    else {
        raw   = QByteArray::fromBase64(val);                       /* base64 first  */
        plain = FUN_141d7a5e0(raw, pass);
    }
    FUN_141992bf0(parserNode, plain);             /* write plaintext back      */
}
```

**Conclusion (strong inference, not confirmed):** the chain that was reached is PT's
*generic `Ipc::CParser` encrypted-attribute* helper - it decrypts a **base64 string
attribute whose passphrase is stored beside it in the same parse tree**
(`pt::CActivityFile`, activity passwords, `bin\PT.conf`). The `.pkt` **document** is
encrypted from a *different* call site that this pass did not reach, so the default
passphrase for a password-less `.pkt` remains **unknown**.

What is settled:

* There is **no compile-time passphrase constant** at the call site found; the
  passphrase is always a `QString` argument, never a literal.
* The file is **not merely compressed**: all 293 samples fail a zlib check
  (`78 01/5E/9C/DA`), inflating them yields invalid data, and the byte entropy is
  **7.9987 bits/byte** - so `.pkt` really is encrypted.
* Reading a `.pkt` offline still needs the right passphrase, but the derivation, the
  sizes, the iteration count, the cipher, the mode and the failure modes are now fully
  known - which is everything else required.

---

## 5. Practical consequence

* A `CNet`-like simulator needs **one heap ordered by (time, sequence)** and nothing
  more for correctness; the registry map and the worker thread exist for *realtime*
  pacing and for cancelling named timers, not for the simulation semantics.
* The Event List is presentation state and must be bounded separately from the queue.
* Devices live in a `std::vector` on the topology object; connections live in an
  ordered map; both are serialised into a typed node tree keyed by translated names.
* The document format is **not** AES-CBC + HMAC and needs **no passphrase** - see §7.
  It is `zlib` over a position-dependent XOR; newer saves add Crypto++ `EAX<Twofish>`
  with compile-time key/nonce constants. Reading `.pkt` offline works: **293/293** shipped
  `.pkt` samples decode to well-formed XML.

---

## 6. The mini-simulator, extended (`pt_minisim\`)

`ptmini.h` + `main.cpp` now cover TCP and UDP in addition to the original
Ethernet/ARP/IP/ICMP path, and model the GUI's jitter switch.

### 6.1 Layer-3 demultiplexing

The process chain is still built linearly - `Eth -> Arp -> HostIp -> Icmp -> Tcp -> Udp` -
but a received packet no longer walks *up* that chain. `HostIp`/`RoutingProcess` hand a
locally-addressed packet to a protocol-number map, which is how the real
`CHostIp`/`CRoutingProcess` "deliver locally" branch behaves:

```cpp
std::map<uint8_t, Process*> upperByProto;   // 1 ICMP, 6 TCP, 17 UDP
void Device::deliverUp(Pdu& p) {
    auto it = upperByProto.find(p.proto);
    if (it == upperByProto.end()) { /* CHOSTIP_FC_NO_HIGHER_PROCESS */ return; }
    it->second->processReceive(p);
}
```

Only the *send* path uses the linear chain, and every L4 process forwards anything that
is not its own protocol (`TcpProcess::processSend` lets a UDP PDU pass straight through).
That mirrors the real design, where each `CProcess` is both a link in a chain and a
demultiplexer.

### 6.2 TCP - `TcpProcess` + `TcpConnection`

Packet Tracer splits TCP into one `Tcp::CTcpProcess` per device (sockets) and one
`CTcpConnection` per flow (the state machine). The mini-sim does the same and emits the
real step codes:

| area | step codes |
|---|---|
| socket / listen | `CTCPPROCESS_FC_CONNECT`, `_PORT_ALREADY_IN_USE`, `_NO_MORE_PORTS` |
| handshake | `CTCPCONNECTION_FC_SET_STATE`, `_ACCEPT_CONNECTION`, `_CON_SUCCESS`, `_REFUSE_CONNECTION`, `CTcpConnection_fc_connection_refused` |
| send side | `_SEND_BUFFER`, `_NOT_EXCEED_WINDOW_SIZE`, `_BUFFER_DATA`, `CTcpConnection_fc_buffer_data_nagle_alg`, `_TAKE_OUT_FROM_BUFFER` |
| receive side | `_DEENCAP`, `_ENCAP`, `_EXPECTED_SEQUENCE`, `_UNEXPECTED_SEQUENCE`, `_DUPLICATE`, `CTcpConnection_fc_receive_data`, `CTcpConnection_fc_receive_ack` |
| retransmission | `CTcpConnection_fc_retransmit`, `_RETRANSMIT_EXPIRES`, `_MAX_RETRANSMIT`, `CTcpConnection_fc_connection_reset` |
| teardown | `CTcpConnection_fc_close_connection`, `CTcpConnection_fc_close_server`, `CTcpConnection_fc_disconnected`, `_DROP_DATA_NOT_IN_ESTABLISHED` |

The retransmission timer is scheduled on the same `Scheduler` heap as everything else and
is invalidated by a token when the ACK arrives - the timer never has to be *cancelled*,
which is exactly why the real `CScheduler` can be a plain heap.

A worked trace from the demo (one deliberately dropped frame on PC1's uplink):

```
0.00068 PC1 CTCPCONNECTION_FC_ENCAP   ... ACK PSH seq=1001 ... data="GET /index.html"
0.00069 PC1 CTcpConnection_fc_buffer_data_nagle_alg  Nagle: "HEAD /logo.png" buffered
1.00099 PC1 CTcpConnection_fc_retransmit             attempt 1
1.00107 PC2 CTcpConnection_fc_receive_data           Data received: "POST /log"
```

**A bug worth recording.** The first version armed the retransmission timer for *every*
segment, including bare ACKs. Because the timer re-arms itself, `sim.run()` (drain) then
never terminated: it drove the connection through `MAX_RETRANSMIT` to `CLOSED` before the
application ever sent data. The fix is both a code fix (arm only for SYN/FIN/data) and an
API one (`Scheduler::peekTime()` + `Simulation::runUntil(t)`), so a caller can advance to
a time instead of to quiescence. The real engine carries the same hazard, which is why
`CScheduler` exposes a named-timer registry and a `removeIpcTimer` path rather than
relying on drain.

### 6.3 UDP - `UdpProcess`

Connectionless and stateless, matching `Udp::CUdpProcess`:

| area | step codes |
|---|---|
| socket | `CUDPPROCESS_FC_CREATE_SEGMENT` |
| send | `CUDPPROCESS_FC_ENCAP` |
| receive | `CUDPPROCESS_FC_DECAP`, `CUDPPROCESS_FC_NOT_FOUND_PROCESS` |
| error | `CUDPPROCESS_FC_SEND_ICMP_UNREACHABLE` -> ICMP type 3 / code 3 |

### 6.4 Jitter

The Simulation panel's *"Disable random packet transmission delay jitter"* is
**unchecked by default**, so transmission is jittered. The mini-sim reproduces it in
`Port::send`:

```cpp
Time arrive = dev->sim->now() + dev->procDelay + tx + linkDelay + dev->sim->jitter();
```

`main()` runs the whole scenario twice and compares - the cheapest possible
demonstration of the flag:

```
                          jitter ON        jitter OFF
first frame at SW1 (s)   2.44705e-05      2.436e-05
virtual time (s)         1.00167          1.00163
events                   461              461
```

### 6.5 Injected loss

To exercise the retransmission path deterministically the mini-sim has a test hook the
real engine gets from its link-error features:

```cpp
int Port::dropNext = 0;   // silently lose the next N frames, with a CPORT_FC_FRAME_DROPPED step
```

**Final state of the demo: 461 events, 66 frame arrivals, 7 TCP state transitions,
1 retransmission, 1 injected frame loss, virtual time ~1.0017 s.**

---

## 7. `.pkt` documents — SOLVED (fully readable offline)

> **This section supersedes the passphrase framing of §4b/§4c.** A `.pkt` needs **no
> passphrase**. `CryptoPP::DataDecryptor<Rijndael,SHA256,...>` is genuine code in the
> binary, but it belongs to a *different* subsystem (`Ipc::CParser` encrypted attributes);
> live instrumentation confirms it is never entered while a `.pkt` is opened.

**Status: confirmed end-to-end** by three independent methods — decompilation of the
loader, live instrumentation of the running process with Frida, and a sweep of **411**
shipped files, of which **400 decode to XML** and **all 293 `.pkt` decode 100 %**.

### 7.1 The chain

```
CAppWindow vftable[56]  (slot 0x1C0)          FUN_141fae9d0      ".pkt open" document loader
   FUN_141d79e20(content, out, path)                            document decoder
      |  if the input already parses as XML  ->  out = content   (passthrough)
      |
      |  tmp = FUN_141d77b00(content, 0x89, 0x10)               "step 1"
      |        tmp[i] = (byte)(n*(1-i)) ^ content[n-1-i]        n = |content|
      |        tmp     = FUN_141d77560(tmp, 16 x 0x89, 16 x 0x10)
      |                   CryptoPP::EAX_Final<CryptoPP::Twofish,0>  (decrypt,
      |                   trailing 16-byte tag consumed -> |tmp| = n - 16)
      |
      |  if (|tmp| == 0)  tmp = content                          fallback => "variant A"
      |  n = |tmp|
      |  for i: tmp[i] ^= ((n & 0xFF) - (i & 0xFF)) & 0xFF       position XOR
      |  out = qUncompress(tmp)                                  zlib inflate
      |  out.replace(<4 mojibake sequences>)                      character cleanup
      |
      -> QXmlStreamReader root must be one of
         PACKETTRACER5 | PACKETTRACER | PACKETTRACER_ACTIVITY | PACKETTRACER5_ACTIVITY
```

How the cipher was pinned down: `FUN_141d77560` loads the

```
??_7?$EAX_Final@VTwofish@CryptoPP@@$0A@@CryptoPP@@6B@
```

vftable (`CryptoPP::EAX_Final<CryptoPP::Twofish,0>`) and calls
`FUN_141d5a9e0(cipher, keyPtr, 0x10, ivPtr)` (the Crypto++ `SetKeyWithIV`),
`FUN_141d78730` (`StringSource`), and `FUN_141d60100(obj, cipher, sink, 0x10)`
(the `AuthenticatedDecryptionFilter` constructor with a 16-byte MAC), collecting the
plaintext into a `std::string` and copying it into the output `QByteArray`. The output
length is always `input - 16`, i.e. the EAX tag is verified and dropped.

### 7.2 Two on-disk variants, one boolean apart

| variant | condition | pipeline |
|---|---|---|
| **B** (new saves, PT ≥ ~6) | `step 1` produces `n-16` bytes | `raw -> reverseXor -> EAX<Twofish> -> positionXor(using n-16) -> zlib` |
| **A** (legacy saves, e.g. `<VERSION>5.2.0.0068</VERSION>`) | EAX rejects the input, loader falls back to the raw bytes | `raw -> positionXor(using n) -> zlib` |

`positionXor` is `y[i] = x[i] ^ (((n & 0xFF) - (i & 0xFF)) & 0xFF)` for **both**
variants; only `n` differs (`n-16` vs `n`). Getting `n` wrong silently corrupts every
file longer than 255 bytes.

### 7.3 The keystream is a compile-time constant

EAX is a CTR-mode construction: `keystream = Twofish_ECB(counter blocks)`, and the tag
only authenticates. Both the key and the nonce are built from constant fills
(`FUN_1427ae010` writes 16 x `0x89` and 16 x `0x10`), so **there is exactly one keystream
for every variant-B file**, independent of contents, path or name. That was verified
three ways:

* the keystream derived from `HMI.ptd` decoded `Generic Sensor.ptd` correctly, and
* the keystream derived from a 3.5 MB `.pkt` decoded that same `Generic Sensor.ptd`, and
* both derivations produce the identical first bytes `eb f6 f3 0c c5 40 ba 6b 82 74 29 4b ...`.

Consequence: one capture yields an offline reader for every file up to the captured
length. This is why no passphrase exists — there is nothing secret in the *document*
path, only obfuscation built from fixed bytes.

### 7.4 Verification

| check | result |
|---|---|
| `sample_b.pkt` (23,965 B) live: step 1 output | 23,949 B = `size - 16` |
| same file, live decryptor plaintext | 142,286 B, `<PACKETTRACER5>`, `<VERSION>6.1.0.0073</VERSION>` |
| 3,486,271 B `.pkt` live decryptor plaintext | **7,320,358 B** |
| same file, offline reader | **7,320,358 B** — exact match, `<PACKETTRACER5>`, `<VERSION>9.0.0.0000</VERSION>` |
| sweep of 411 shipped files | 400 decode; methods: B 344, A 50, plain-XML 6, undecoded 11 |
| `.pkt` | **293 / 293**, all root `PACKETTRACER5` |
| `.ptd` (device templates) | **88 / 88**, all root `PACKETTRACER` |
| `.pka` (activities) | **6 / 6**, all root `PACKETTRACER5_ACTIVITY` |
| `.conf` | 13 / 13 (some are plain XML / INI-style) |
| `sample.pkt` 4,899 B (variant A) | inflates to 34,065 B XML, `<VERSION>5.2.0.0068</VERSION>`, contains `0050.0F5E.DD01`, `10.10.11.1` |

The 3.5 MB match between the live `AuthenticatedDecryptionFilter` output and the purely
offline computation is the decisive evidence: same length, same bytes, same XML.

### 7.5 Why 11 files still fail — and it is not a bug

`FUN_141d77b00` is the *generic* decode step and it has only **four** callers, all
reachable through two thin wrappers that supply different constants
(`ghidra_pt\out\xrefs.txt`):

| call site | constants | used for |
|---|---|---|
| `FUN_141d79e20` | `(0x89, 0x10)` | `.pkt` documents |
| `FUN_141d7a400` | `(0x89, 0x10)` | other document-ish loads |
| `FUN_141d7a3e0` | **`(0xBA, 0xBE)`** | a second, independent family |

Because the key/nonce are the only parameters, a `(0xBA,0xBE)` call would produce a
*different* keystream, so files encrypted that way cannot decode with the document
keystream.

The undecoded files are exactly the extension containers — 6 `.pts`, 4 `.pkp`, 1 `.pta`
(extension metadata, not network topologies):

```
Marvel.pts             885,047 B   fb 9e 51 c9 69 68 49 52 ...   no ZIP, no zlib magic
ActivitySequencer.pts 14,973,237 B  dd a2 ac 6c 41 a1 54 71 ...   no ZIP, no zlib magic
mqtt_broker.pkp        101,628 B   c1 4d 67 22 3b 88 64 57 ...   no ZIP, no zlib magic
```

**Status: CONFIRMED — and the `(0xBA,0xBE)` pair is *not* the extension family.**

A second probe (`probe14.js`) hooked the EAX wrapper `FUN_141d77560` itself and read the
two 16-byte buffers it is handed. Two live calls used

```
call 1-2 : key = 16 x 0xBA   nonce = 16 x 0xBE
call 3+  : key = 16 x 0x89   nonce = 16 x 0x10     (every document/template load)
```

so the key/nonce *bytes* are no longer inferred: each is the fill byte of the constant,
repeated 16 times — the same two fills are also the 16-byte `keyA`/`keyB` used inside
step 1. Matching the two inputs (54 B, 278 B) against the disk finds exactly one file
each:

```
C:\Users\ricky\Cisco Packet Tracer 9.0.1\uuid    54 B
C:\Users\ricky\Cisco Packet Tracer 9.0.1\nets   278 B
```

The `(0xBA,0xBE)` family therefore protects PT's own installation-UUID and nets metadata,
**not** the `.pts`/`.pkp`/`.pta` containers. The earlier guess that `(0xBA,0xBE)` was the
extension family is retracted here.

What the containers are **not** is now settled:

* all 13 of them were pushed through the variant-B path with the document keystream →
  **0 bytes** of zlib output in every case;
* across the whole instrumented startup the decryptor is never handed an output larger
  than the 3,486,255-byte topology (the containers are 5.9 MB and 14.9 MB), and PT never
  opens them at startup at all.

So they are not decoded by `FUN_141d77b00` / `FUN_141d77560` under **either** constant
pair.

**Status for the containers: unknown.** Confirmed: flat/high entropy, no ZIP, no zlib
magic, and not this scheme. Not confirmed: what it actually is — most likely a separate
container format handled by another library with its own key material, which would need
one extension to actually be loaded before it can be observed.

Keystream provenance caveat: the keystream is a **captured** artifact, not a derived one.
An attempt was made to reproduce the cipher offline (Twofish-128 in Crypto++'s
`EAX_Final`, 131,072 structural variants, judged by a key-independent invariant — in
EAX-CTR, consecutive keystream blocks must decrypt to counter values that differ by
exactly +1 — for both key candidates); **no variant matched**, so `pktread.ps1` still
needs the 3.5 MB blob (or a fresh capture). The q0/q1 nibble tables used in that attempt
*do* reproduce the published reference table heads, so the error is in the round
skeleton, not the S-boxes. Also note: the same 24 KB keystream re-derived from a
completely different file (`sample_b.pkt`) has the identical head
`eb f6 f3 0c c5 40 ba 6b 82 74 29 4b 63 b5 4c c1 7f 34 03 4f 32 76 c6 5d`, which is a
third independent confirmation of universality.

Remaining caveat: a file **larger** than the captured keystream (3,486,255 B) must be
captured first — the regional `.pts` files reach 15 MB. Everything in the shipped
network-document tree decodes today.

### 7.6 Tools delivered

| file | purpose |
|---|---|
| `pt_frida\pktread.ps1` | offline reader: single file or whole-tree batch, no PT, no Frida. `-KsFile` points at the keystream. |
| `pt_frida\ks\ks_universal.bin` | the recovered 3,486,255-byte EAX keystream (derived data, not a secret). |
| `pt_frida\probe12.js` | Frida capture that dumps the step-1 output of every decode (`ks\<id>_<insize>_<outsize>.bin`). |
| `pt_frida\probe14.js` | reads the two 16-byte key/nonce buffers handed to the EAX wrapper — how the constants were confirmed. |
| `pt_frida\kswork.ps1` | **re-derives the keystream**: given any `(raw file, step-1 dump)` pair it rebuilds `ks_universal.bin` (`-SrcFile -DumpFile -OutKs`). `-SkipTest` skips the container probe. |
| `pt_frida\derived\ks_universal.bin` | the same keystream, regenerated and re-verified in this session. |

Keystream re-derivation recipe (no reverse engineering needed, ~1 min):

```powershell
# 1. capture: spawn PT on any variant-B file with probe12.js loaded, wait for boot
# 2. rebuild:  ks[i] = tmp[i] ^ dump[i],  tmp = reverse-xor of the raw file
powershell -File pt_frida\kswork.ps1 -SrcFile <file.pkt> `
          -DumpFile pt_frida\ks\<id>_<insize>_<outsize>.bin -OutKs pt_frida\ks\ks_universal.bin -SkipTest
```
| `ghidra_pt\out\decomp3.txt` | decompilation of the loader, the decoder, step 1 and the EAX wrapper. |
| `ghidra_pt\out\xrefs.txt`, `xrefs2.txt` | call sites and their constants. |
| `pt_frida\decoded_final2\` | the decoded corpus (~400 XML documents). |

Usage:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass `
  -File pt_frida\pktread.ps1 -File "C:\...\saves\...\foo.pkt" `
  -KsFile pt_frida\ks\ks_universal.bin -OutDir .\out
```

### 7.7 What this means for the reverse-engineering question

The `.pkt` format is **obfuscated, not protected**: fixed key, fixed nonce, no
passphrase, no per-file secret. Anyone who recovers the two constants (they are visible
as immediates at the call site) or captures the keystream once can read and, with the
same primitives, write these files. Practically, that confirms the addendum's original
thrust — the *interesting* engineering in Packet Tracer is the simulation engine
(§1–§3, §6), and the container format is a thin obfuscation layer on top of
`zlib` + `Twofish-EAX`.
