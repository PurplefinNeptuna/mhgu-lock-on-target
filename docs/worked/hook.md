# Lock-on Target: hook

Worked example from `mods/lock-on-target/`. Game facts it rests on are in `mhgu-understanding`.

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
