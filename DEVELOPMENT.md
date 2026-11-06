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

| Finding | Where | Evidence |
| --- | --- | --- |
| Local hunter | `sFestaUnit` (`main+0x2126f48` cell) `+0x58` | Same chain in the cheat and in Pixel Cheats' "Hunter Speed x5" (`+0x14b0`); matched a live hunter on MHGURecomp |
| Tap logic `(hunter)` | `0x27e970`, 568 bytes | Bytes. One caller, `0x2a7b74` in the hunter update `0x2a79cc`, with `mov r0, r4` (the hunter). Nothing branches to `0x27e974` |
| Tap window, f32 frames | hunter `+0x145c` | Bytes. Only float user is `0x27e970`: counts down via `0x53aa84(hunter, &timer, 1.0)`; a press with it closed sets 12.0 (`movt #0x4140`); cleared at `0x27e9f4`, `0x27ea04`, `0x27eaf0` |
| Button words | hunter `+0x13f4` held, `+0x13f8` pressed, u16 | Bytes, masked with `0x65f8ec()` |
| Hold time | hunter `+0x1450`, f32 | Bytes, compared with 1.0, 16.0 and 18.0 in `0x27e970` |
| Request bitset | hunter `+0x1434` | Bytes. `0x285ea0` sets bit n, `0x285ec4` clears, `0x285ee8` tests (no `b`/`bl` caller). The tap logic sets bits 0, 24 (unless session `0x188ca48` `+0x6f` == 1) and 6 |
| Request clears | bits 0 and 24 at `0x2a7dd8`/`0x2a7de4` when `0x2e0b10(hunter)` == 1; bit 6 in `0x27eccc` | Bytes |
| Window cleared each frame | when `0x18e50([0x184d750])` != 1 | Bytes; meaning of `0x18e50` unread |
| Countdown `(hunter, &timer, step)` | `0x53aa84` | Bytes. `timer -= hunter[+0x1c] * step`, frame time in 1/60 s (0.4 at 150 fps); returns 1 and stores the clamp once it reaches 0 |
| Why the cheat works on Eden | Eden's cheat engine | `CHEAT_ENGINE_NS = 1000000000 / 12` in Eden's `cheat_engine.cpp`: 5.0 drains between writes at 60 fps, so the window keeps running out and fires the target request. Written every frame (MHGURecomp), it never runs out |

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

- The cheat's author, for credits.
- Target cycling is untested as an item, and Ryujinx and Switch hardware have no run.

The cheat's crosshair twitch (its zoom-in replaying) came from the target request firing about
12 times a second on Eden; fired every frame, the crosshair does not twitch.

## Do not

- Hold the tap window open (write 5.0 every frame): it never runs out, so no target request fires.
- Hold the window for every hunter: the hook also runs for other players' hunters.
- Replace the game's target choice, cycling or button mapping: the tester asked for the game's own.
