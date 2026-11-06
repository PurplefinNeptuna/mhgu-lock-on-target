# Development provenance

Project-owned code in this folder belongs to PurplefinNeptuna (2026), all rights reserved;
see `LICENSE`.

## Vendored and linked components

- `vendor/forge-rs` — forge-rs 1.5.0, commit
  `d1865b68e9b194eb3eacc8bd71bbf601a5fac63e`, by Fexty, MIT
  (`vendor/forge-rs/LICENSE`, `licenses/FORGE-MIT.txt`). Built from source; no
  upstream binaries. It supplies the plugin lifecycle, hooks, patches and
  logging.
- `vendor/libnx` — libnx (Hydrosphere Developers, MIT,
  `vendor/libnx/LICENSE-MIT`). The three ARM32 SVC wrappers in `src/svc.S`
  (`svcQueryMemory`, `svcGetInfo`, `svcFlushProcessDataCache`) are adapted from
  it and keep the upstream copyright line. Forge exposes no mapping query,
  program-id query or process cache flush, which is the documented gap beside
  their use in `src/lib.rs`.
- `vendor/crates` — the exact registry sources resolved by `Cargo.lock`
  (48 directories, 23 locked packages). Runtime-linked: `log` (MIT,
  `vendor/crates/log-0.4.34/LICENSE-MIT`). Build-time only: `mhgu-forge-macros`
  and its `proc-macro2`/`quote`/`syn`/`proc-macro-crate`/`toml_*`/`winnow`
  chain, `serde_core`/`serde_derive`, `hashbrown`/`indexmap`/`equivalent`,
  `memchr`, `unicode-ident`. Each directory keeps its own license file.
- `licenses/RUST-MIT.txt`, `licenses/UNICODE-3.0.txt` and
  `licenses/COMPILER_BUILTINS.txt` cover the `core` and `alloc` built from
  source by `build-std`, including the compiler-builtins licensing exception.

Host-only toolchain, not shipped and not linked into the payload: devkitARM
binutils (`arm-none-eabi-gcc/-ld/-nm/-objdump`), `cargo-forge`, and the pinned
nightly Rust (`rust-toolchain.toml`, `dependencies.lock.json`).

## Ported behaviour

The mod reproduces the "Lock-on Target V1" Atmosphere cheat for build FB08F1D20FD1204F, which
writes 5.0 to `[[sFestaUnit]+0x58]+0x145C` while `[main+0x184DDC8]` is nonzero. Source: the
"Camera Lock-on V1" entry of the MHGU Modding wiki's Cheat Codes page
(https://github.com/RTHKKona/MHGU-Modding/wiki/Cheat-Codes), which says it was found in
https://www.youtube.com/live/cJER9XHHHoU and names no author. No licence is stated. No cheat
text is copied or shipped; the mod writes a different value (0.001) to the same field.

## Research inputs

Offsets and signatures come from the user's own copy of MHGU 1.4.0, read from the hash-checked
main NSO. `scripts/verify.py` compares the exact-build signatures in `core/src/signatures.rs`
against the supplied main NSO. No game binary or asset is shipped.

## AI assistance

Code, tests, build scripts and documentation in this folder were written with
Claude Code (https://claude.com/claude-code).
