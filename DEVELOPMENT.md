# Developing Lock-on Target

## Current state

r2 READY_TO_DISTRIBUTE (2026-11-06), unpublished, release at `build/lock-on-target/releases/r2/`, a
Forge port of the "Lock-on Target V1" cheat. One pre-only entry hook on the hunter's
target-camera tap logic sets the local hunter's tap window to 0.001 right before the game reads
it, so the window runs out in that call and the game raises its single-tap target request every
frame. Accepted on Eden and MHGURecomp 0.14.3 ("ok, work for both eden and recomp"), with no
crosshair twitch ("its not spazzing at all"); target cycling was not itemized.

Tester's words, which r1 and later versions build to: "when locked in (r3) camera point to
monster like L button but forever, r3 will cycle the lock target. also without patching, the
targetting have spasm because there are animation like targeting zoomin", and "number 2 and 3
follow the ingame camera lock and button settings": the game's own target choice, cycling and
button mapping are used, never replaced.

## Findings

All addresses are main-relative on MHGU 1.4.0 (build FB08F1D20FD1204F), read from the pinned
NSO. Research notes, including the searches that found no camera-side reader, are in
`work/lock-on-target/NOTES.md`.

The findings (local hunter, tap logic and its window, button words, hold time, request bitset and its clears, the countdown, and why the cheat works on Eden but not MHGURecomp) are in `.agents/skills/mhgu-understanding/references/input-and-actions.md`, section Target camera tap.

## Hook

| Field | Value |
| --- | --- |
| Target | `main+0x27e970`, entry, ARM |
| Displaced words | `e92d48f0` `push {r4-r7, r11, lr}`, `e1a04000` `mov r4, r0`; both position-independent |
| Arguments / result | r0 hunter; no result read by the caller (`0x2a7b78` reloads r0) |
| Detour | If the hunter equals `[sFestaUnit+0x58]`, store `0x3a83126f` (0.001) at hunter `+0x145c`; then call the original once with r0 unchanged |
| Chaining | Pre-only, so the entry accepts another plugin's Forge hook (`e51ff004` plus literal); every later word stays exact |
| GE | No `SEL`, `MRS`, `MSR` or parallel add/subtract in `0x27e970`'s 568 bytes or in the emitted NRO (verify) |
| Windows | `0x27e970`, `0x27e9b0`, `0x27e9f0` (through the timer writes) and the GNU build-ID note `0x172e52c` |
| Pool | One ordinary 16-byte trampoline |

`sFestaUnit` is a `cSystem` (via `sUnit`), so Forge's constructor hook registers it and
`forge_singleton_resolve` maps it before plugins load; `forge_onInit` looks it up once and
refuses to install without it. It lives for the process, so the detour keeps only its address.

## Open items

- The cheat's author: the MHGU Modding wiki lists only the stream it was found in.
- Target cycling is untested as an item, and Ryujinx and Switch hardware have no run.

The cheat's crosshair twitch (its zoom-in replaying) came from the target request firing about
12 times a second on Eden; fired every frame, the crosshair does not twitch.

## Do not

- Hold the tap window open (write 5.0 every frame): it never runs out, so no target request fires.
- Hold the window for every hunter: the hook also runs for other players' hunters.
- Replace the game's target choice, cycling or button mapping: the tester asked for the game's own.
