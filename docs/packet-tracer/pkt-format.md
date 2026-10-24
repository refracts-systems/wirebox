# .pkt document format: results and corrections

## Last status in report

The document file path was reportedly successfully traced from the CAppWindow loader via FUN_141d79e20. If the input can be parsed as XML, the decoder can perform passthrough.

| Variant | Reported pipeline |
|---|---|
| B, newer document in sample | reverse/XOR → Twofish/EAX with constant material → XOR position → qUncompress → XML |
| A, legacy on sample | XOR position → qUncompress → XML |
| Plain | Direct XML |

qUncompress includes Qt framing; New parsers should check framing details before using regular zlib inflaters. A standalone implementation of the decoder is not written in this documentation work.

According to reports, variant B uses a 16 byte key containing 0x89 and a 16 byte nonce containing 0x10. Hook FUN_141d77560 reportedly reads that buffer directly. There is no per-file passphrase on the tested document envelope. Activity passwords and encrypted attributes are different scopes.

## Reported test evidence

- One 3,486,271 byte file produces 7,320,358 bytes of XML, byte-identical between the capture process output and the offline reader.
- Sweep 411 files: 400 decoded, consisting of 344 B variants, 50 A variants, and 6 plain XML; 11 is not decoded on that sweep.
- Document categories called: 293/293 .pkt, 88/88 .ptd, and 6/6 .pka produce well-formed XML.
- Keystreams from different inputs are reported to have identical prefixes, corroborating the use of the same stream across the document family.

The numbers per extension category mentioned do not detail all 400 outputs. The follow-up notes mention 13 extension containers, different from 11 in the previous sweep. An inventory/manifest corpus is needed to unify this scope of figures.

## Reader limit

The reader still uses a captured keystream of 3,486,255 bytes. Attempts to reproduce Twofish/EAX independently have not been suitable. Therefore, the historical phrase 'fully recovered' must be read alongside this limit.

Sizes that exceed stream coverage are not yet supported by the artifact. The success of capture/plaintext also does not prove that checking authentication tags, creating tags for writers, or round-trips received by PT. That need remains open.

This result does not mean Wyrlex requires .pkt to run. The importer is an additional input line; The engine can accept its own topology format.

## Cryptographic path correction

Findings of AES-CBC, SHA-256/HMAC, and Mash 2500 iterations on DataDecryptor<Rijndael,...> should not be used as explanations for the .pkt envelope. Addendum §7 replaces hypotheses §4b/§4c for document containers.

FUN_141d5c710/FUN_141d5c880 which was originally called the decryptor constructor was corrected as an exception constructor. Details of legacy cryptographic functions remain on file for context, not as final decoder specifications.

## Metadata and extensions

Key/nonce pairs 16 × 0xBA and 16 × 0xBE are reported to be used in uuid/nets. The hypothesis that the couple was an extended family has been withdrawn.

The tested .pts/.pkp/.pta still cannot be decoded successfully. Failing a particular trial and not being seen in startup observations does not prove that all possible decoder paths have been eliminated. High entropy and the absence of any known magic have not identified the cipher or its format.

## Original artifact

The report mentions pktread.ps1, ks_universal.bin, probe12.js, probe14.js, kswork.ps1, as well as Ghidra output and decoded XML. Artifact executables, key streams, and corpus XML are not copied in this documentation work.

Sources: [final addendum summary](sources/04-pkt-addendum.txt) and [full report §7](sources/REPORT_addendum.original.md).
