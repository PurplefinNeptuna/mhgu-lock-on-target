# Developing Lock-on Target

## Current state

r1 DEVELOPMENT (2026-11-06), candidate at `build/lock-on-target/candidates/r1/delivery/`, a
Forge port of the "Lock-on Target V1" cheat. One pre-only entry hook on the hunter's
target-camera tap logic writes the cheat's 5.0 into the local hunter's tap window right before
the game reads it. Built, host-tested and statically verified; gameplay NOT RUN. The target
crosshair's zoom-in replay, the tester's "spasm", is ported as it is; see Open items.

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

## Hook

| Field | Value |
| --- | --- |
| Target | `main+0x27e970`, entry, ARM |
| Displaced words | `e92d48f0` `push {r4-r7, r11, lr}`, `e1a04000` `mov r4, r0`; both position-independent |
| Arguments / result | r0 hunter; no result read by the caller (`0x2a7b78` reloads r0) |
| Detour | If the hunter equals `[sFestaUnit+0x58]`, store `0x40a00000` at hunter `+0x145c`; then call the original once with r0 unchanged |
| Chaining | Pre-only, so the entry accepts another plugin's Forge hook (`e51ff004` plus literal); every later word stays exact |
| GE | No `SEL`, `MRS`, `MSR` or parallel add/subtract in `0x27e970`'s 568 bytes or in the emitted NRO (verify) |
| Windows | `0x27e970`, `0x27e9b0`, `0x27e9f0` (through the timer writes) and the GNU build-ID note `0x172e52c` |
| Pool | One ordinary 16-byte trampoline |

`sFestaUnit` is a `cSystem` (via `sUnit`), so Forge's constructor hook registers it and
`forge_singleton_resolve` maps it before plugins load; `forge_onInit` looks it up once and
refuses to install without it. It lives for the process, so the detour keeps only its address.

## Open items

- The spasm is the target crosshair (HUD), not the camera: its zoom-in animation replays while
  locked. Find the crosshair's GUI object and what restarts its animation; the request bits 0,
  24 and 6 at hunter `+0x1434` are the likely trigger. Lead: `cUIOCPanelTargetCamera` (DTI size
  `0x58`, vtable `0x177a4d0`), possibly only the options panel.
- Why the cheat had no visible effect on MHGURecomp while the tap window read back held.
- The cheat's author, for credits.

## Do not

- Hold the window for every hunter: the hook also runs for other players' hunters.
- Replace the game's target choice, cycling or button mapping: the tester asked for the game's own.
