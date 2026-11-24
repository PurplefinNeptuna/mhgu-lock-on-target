# Lock-on Target: patches

Worked example from `mods/lock-on-target/`. Game facts it rests on are in `mhgu-understanding`.

## Patches

The mod writes clarkekoko's "Lock-on Target MHGU Recomp" cheat and its "No Target Reticle" part as
published, through Forge patches: three in the tap logic `0x27e970(hunter)`, two in the reticle
update `0x61e95c(panel)`. `core/src/lib.rs` holds each patch's original and written words; a host
test compares the written words with the cheat text.

| Offset | Original | Written |
| --- | --- | --- |
| `0x27e9d8` | `ble 0x27eb10` (hold 1 to 18: reopen the window) | `ble 0x27ea78` (the lock block) |
| `0x27e9f8` | `b 0x27eb4c` (after a long hold: window reopen) | `b 0x27eb80` (the shared tail) |
| `0x27ea78` to `0x27ead0` | the window countdown, 22 words | the lock block, 21 words and a literal |
| `0x61e9e0` | `cmp r0, #1` (state 2: draw the reticle) | `cmp r0, #2` |
| `0x61ea28` | `cmp r0, #1` (state 1: draw the reticle) | `cmp r0, #2` |

The lock block uses the tap window f32 at hunter `+0x145c` as an integer flag:

1. `r0 = 0x65f8ec()`, the target-camera button mask, as the original calls it.
2. If hunter `+0x13f8` (pressed this frame) has the button, `flag ^= 1`.
3. Flag 1: lock off, go to `0x27eb80`.
4. `0x18ffc([0x184d750])` (sFestaCamera, through GOT slot `0x1838d90`) is zero: go to `0x27eb80`.
5. Otherwise `0x285ea0(hunter, 0)` raises the single-tap target request (bit 0), then `0x27eb80`.

The flag is 0 (lock on) after the paths that clear the window: a long hold (`0x27e9f4`) and every
frame `0x18e50(sFestaCamera)` != 1 (`0x27ea04`). The countdown words after `0x27ead0` are no
longer reached. Only `0x27ea78` is a branch target inside the block.

## Reticle

`0x61e95c` keeps a state at panel `+0x78` (0, 1, 2). In states 1 and 2 it calls the reticle draw
`0x61ecc0` when `0x18e50(sFestaCamera)` == 1 and `0x190d4(sFestaCamera)` == 1. `0x190d4` returns
1 when the current camera's byte `+0xa7c` is set, else 0, so comparing with 2 never matches.
Those are the only two callers of `0x61ecc0`. With the lock raised every frame, the user reported
the reticle misbehaves without these patches; what the draw does each frame is not read.

## Why r2's hook was replaced

r2 stored 0.001 in the window before the original, so the countdown path ran out every frame.
That path raises bit 0 and also bit 24 (unless session `+0x6f` == 1, `0x27eb38`). The block
raises bit 0 only. The user found r2 ruined aiming and the cheat did not; that bit 24 causes it
is inference.

## Install

| Field | Value |
| --- | --- |
| Gate | The four 64-byte windows in `core/src/signatures.rs` (the GNU build-ID note included), then every patch's original words, all before the first write |
| Entry | Untouched; window `0x27e970` accepts another plugin's Forge entry hook (`e51ff004` plus literal) |
| Write | `Patch::new(address, bytes, false)`; its captured original must equal the checked words; then `enable` and flush (`sync_code`), since Forge's patch is a plain copy |
| Rollback | Any failed write or read-back disables the patches already written, in reverse |
| Lifetime | Patches are forgotten after install; dropping one would restore the site |

Forge calls `forge_onInit` before any hunter update, so no hunter runs the tap logic during the
writes. The plugin exports no `forge_onUpdate`.

On MHGURecomp, `forge_patch_enable` reports the write through `code_changed`
(`runtime/forge.c`, `patch_apply`), so Recomp interprets the tap logic for the session. That is one
call per hunter per frame.
