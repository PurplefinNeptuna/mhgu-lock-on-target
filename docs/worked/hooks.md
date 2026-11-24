# Lock-on Target: hooks

Worked example from `mods/lock-on-target/`. Game facts it rests on are in `mhgu-understanding`.

The mod does what clarkekoko's "Lock-on Target MHGU Recomp" and "No Target Reticle" cheats do, with
two Forge hooks that replace their function. It writes no code patches: MHGURecomp ignores a Forge
patch that does not form a hook (see "Do not" in `DEVELOPMENT.md`).

## Hooks

| Hook | Target | Displaced words | Original |
| --- | --- | --- | --- |
| Tap logic | `0x27e970(hunter)`, ARM | `push {r4-r7, r11, lr}`, `mov r4, r0` (relocatable) | Trampoline; run for every other hunter and for the game's other path |
| Reticle check | `0x190d4(camera) -> 0/1`, ARM | `push {r4, r5, r11, lr}`, `ldr r5, [r0, #0x54]` | None (`forge_hook_create` with a null original) |

Both replace behaviour, so neither accepts another plugin's hook on its entry; all six windows in
`core/src/signatures.rs` stay exact (both entries, the hold thresholds, the window writes, the
tail with its GOT literals, and the GNU build-ID note).

## Tap logic detour

For the local hunter (`[sFestaUnit+0x58]`), the detour runs what the cheat leaves of the function:

1. `camera = [[main+0x1838d90]]` (sFestaCamera). If `0x18e50(camera)` != 1, run the original
   (its other path, which the cheat does not change) and set the lock on.
2. `mask = 0x65f8ec()`. If held `+0x13f4` has the button and hold time `+0x1450` > 18.0: raise
   bit 6 (`0x285ea0`), store 0 in the window `+0x145c`, lock on.
3. Otherwise a tap (pressed `+0x13f8` has the button) flips the lock. While the lock is on and
   `0x18ffc(camera)` != 0, raise bit 0.
4. The original's tail: if `0x5584a0([[main+0x1838da8]])` == 1, clear bit 6 (`0x285ec4`).

`core::step` holds steps 2 and 3 and has host tests; `longer_than_18` compares the f32 as bits, as
`vcmpe` + `ble` does (NaN and negatives are not longer). The cheat keeps the lock flag in the
window field of every hunter; the plugin keeps one flag for the local hunter and resets it when the
hunter changes. Other players' hunters run the original unchanged, so the window stays a float.

Only bit 0 is raised while locked. The original's countdown path also raises bit 24 (unless
session `+0x6f` == 1), which ruined aiming in r2 by the user's report; that bit 24 is the cause is
inference.

## Reticle detour

`0x190d4` returns the current camera's byte `+0xa7c` as 0 or 1. Its only callers are the reticle
update `0x61e95c`'s compares with 1 at `0x61e9e0` and `0x61ea28`, which draw the reticle
(`0x61ecc0`). The cheat compares with 2; the detour returns 0, which neither compare takes.

## Install

`forge_onInit` looks up `sFestaUnit`, checks the six windows, then installs both hooks all or
nothing (`core::install`), with read-back of each branch and trampoline. On MHGURecomp both are
function-entry hooks, the kind it applies.
