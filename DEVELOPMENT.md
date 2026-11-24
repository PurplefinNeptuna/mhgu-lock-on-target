# Developing Lock-on Target

## Current state

r6 READY_TO_DISTRIBUTE (2026-11-24), unpublished, release at `build/lock-on-target/releases/r6/`.
Two Forge hooks do what clarkekoko's "Lock-on Target MHGU Recomp" and "No Target Reticle" cheats do
for the local hunter: a tap of the target-camera button flips the lock, while it is on the game
raises its single-tap target request every frame, and the reticle is never drawn. Unlike the
cheat, the lock pauses while weapon R is held. Accepted on MHGURecomp with Modern Weapon Controls
("now it works correctly, please promote"); Eden has no r6 run.

The user's requirements: the lock follows the cheat ("i want auto lock target to be based on this
cheat"), includes its reticle part, uses hooks and no code patches ("make a real forge mod with
hooks while stil based on the cheat"), and pauses while R is held. The game's own target choice
and button mapping stay in use.

## Findings

All addresses are main-relative on MHGU 1.4.0 (build FB08F1D20FD1204F), read from the pinned
NSO. Research notes are in `work/lock-on-target/NOTES.md`.

The tap logic, its window, button words, request bitset and camera gates are in
`.agents/skills/mhgu-understanding/references/input-and-actions.md`, section Target camera tap.

## How it works

Hooks, detours and install are in `docs/worked/`: [Lock-on Target: hooks](docs/worked/hooks.md).

## Open items

- Not itemized in the accepted run: target cycling and multiplayer. Eden, Ryujinx and Switch
  hardware have no r6 run.
- The cheat aimed fine on Eden without a pause; the same logic pulled R aiming on MHGURecomp with
  Modern Weapon Controls. The cause was not found.
- `0x18e50` runs twice on a frame where it is not 1 (the detour's check, then the original's). It
  ends in a virtual call (+0x3c4) not read; inference that it has no side effect.
- The cheats state no licence. The mod ships none of their words but follows their logic; credit stays.

## Do not

- Write code patches: MHGURecomp applies a Forge patch only when it forms a hook, logs
  "cannot apply to recompiled code: ignored", and the bytes still read back as written.
- Run the tap window out every frame (a 0.001 store before the original): the countdown path also raises bit 24,
  and the user found it ruined aiming.
- Hold the tap window open (write 5.0 every frame): it never runs out, so no target request fires.
- Replace the game's target choice or button mapping: the tester asked for the game's own.
