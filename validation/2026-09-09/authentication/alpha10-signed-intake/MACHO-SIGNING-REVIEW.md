---
title: Apple Silicon alpha10 signing provenance comparison
date: 2026-09-09
status: passed-bounded-artifact-comparison
tags: [prisma-airs-harness, authentication, macos, signing, provenance]
---

The signed intake contains the same executable code and application data as the
original Apple Silicon artifact. The complete byte comparison found changes only
inside the signature allocation and three load-command size fields. This passes
the bounded signing-transformation check; it is not macOS signature-trust or
application acceptance.

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| Original `/var/tmp/airs-alpha10-macos-b012/verified/airs-harness` | 311,478,320 | `3393285d9230f53cfd63702881b253e56f2f864e58acd7c407266167c875c5f0` |
| Signed `/var/tmp/airs-alpha10-signed-intake/airs-harness` | 309,667,632 | `2099b66324eef6d1c63a6ba06da681163f60031767081833888e34626a4dee2b` |

Both files are thin arm64 Mach-O executables. Their headers, all section metadata,
and 28 of 30 load commands are identical. The remaining commands differ only in:

| Field | File offset | Original | Signed |
| --- | ---: | ---: | ---: |
| `__LINKEDIT.vmsize` | 2,112 | 60,473,344 | 58,654,720 |
| `__LINKEDIT.filesize` | 2,128 | 60,459,056 | 58,648,368 |
| `LC_CODE_SIGNATURE.datasize` | 3,644 | 2,432,704 | 622,016 |

The signature starts at the same offset, 309,045,616, and occupies the remainder
of each file. Excluding those three fields, every preceding byte compares equal.
The complete range from the end of load commands, offset 3,648, through the start
of the signature is identical: 309,041,968 bytes, SHA-256
`3460be265bcb28c7d217a640af8caa47ec27b2fcf01cd5368215de344fc6c4db`.
This includes code, data, strings, unwind information, symbol/linker data and
padding. All 19 file-backed section records, including the empty section, have
matching offsets, sizes, flags and hashes. The main `__TEXT,__text` section alone
is 187,692,996 bytes with matching SHA-256
`8e0f40ddd0c31ea6983b4588cf94b49c63a8aac982c33051184a8649d7f5e621`.
The entire `__TEXT` segment cannot have an identical raw hash because it also
contains the changed load-command fields; its executable section bytes do.

The original already has an ad-hoc signature (`0x2` CodeDirectory flags), rather
than no signature. The signed intake has `0x10000` runtime flags, a larger
requirements blob and a nonempty CMS signature-wrapper payload. Its CodeDirectory
uses 16 KiB pages and 18,863 SHA-256 code slots instead of 4 KiB pages and 75,451
slots. This accounts for the substantially smaller hash table; the total signature
allocation shrank by exactly the whole-file reduction of 1,810,688 bytes. Both
CodeDirectories cover the same prefix, and every stored code-page hash was
independently recomputed and matched. The original has 18,010 zero trailing bytes
after its SuperBlob; the signed allocation has 9,021 trailing bytes that are not
all zero. Their hashes are retained, but their meaning is not established by this
parser. They lie entirely within the declared signature allocation, beyond the
CodeDirectory-covered prefix. These format interpretations follow Apple's
[Mach-O declarations](https://raw.githubusercontent.com/apple-oss-distributions/xnu/main/EXTERNAL_HEADERS/mach-o/loader.h)
and [code-signing structures](https://raw.githubusercontent.com/apple-oss-distributions/xnu/main/osfmk/kern/cs_blobs.h).

This Linux-host check used read-only mappings and did not modify either binary.
The complete measurements are in [MACHO-SIGNING-DIFF.json](MACHO-SIGNING-DIFF.json).
The exact one-off parser is retained as
[macho-comparison.py.txt](macho-comparison.py.txt); its hash is in the receipt.
To reproduce, copy that text file to a temporary `.py` path and run Python with
the original path, signed path and a new output JSON path as its three arguments.

No certificate chain, CMS signature, timestamp, Gatekeeper decision, notarization
ticket, Keychain behavior or macOS execution was verified here. The owner-reported
Apple Accepted submission ID `dc835ddf-8841-49bc-a7ee-2f370dcb3457` is recorded as
reported evidence only. Native trust checks and end-user acceptance remain
separate gates. No private key or credentials were accessed.
