# Developing Lock-on Target

## Current state

r6 DEVELOPMENT candidate at `build/lock-on-target/candidates/r6/delivery/`, not run. Two Forge hooks
do what clarkekoko's "Lock-on Target MHGU Recomp" and "No Target Reticle" cheats do for the local
hunter: a tap of the target-camera button flips the lock, while it is on the game raises its
single-tap target request every frame, and the reticle is never drawn. Unlike the cheat, the lock
pauses while weapon R is held. r5 (no pause) ran on MHGURecomp and pulled R aiming onto the monster.
The last accepted build is r2, at `build/lock-on-target/candidates/r2/`; its release is archived in
`backups/accepted/lock-on-target/r2-before-cheat-patches/`.

The user's words: "i want auto lock target to be based on this cheat", "this is much better than
our very aggressive lock (didnt ruin aiming)" and, for the reticle part, "it should be included
here because the reticle will have problem without that", then "please dont make code patches" and
""make a real forge mod with hooks while stil based on the cheat", then "still locking too hard
when aiming with R, unlike when using the cheat, please fix (and dont forget: no code patches that
look like from the cheat itself)". Asked how aiming should behave, the user chose to pause the lock
while R is held. The game's own target choice and button
mapping stay in use.

## Findings

All addresses are main-relative on MHGU 1.4.0 (build FB08F1D20FD1204F), read from the pinned
NSO. Research notes are in `work/lock-on-target/NOTES.md`.

The tap logic, its window, button words, request bitset and camera gates are in
`.agents/skills/mhgu-understanding/references/input-and-actions.md`, section Target camera tap.

## How it works

Hooks, detours and install are in `docs/worked/`: [Lock-on Target: hooks](docs/worked/hooks.md).

## Open items

- r6 has no run: lock on at quest start, tap to turn it off and on, no reticle, the pause while R
  is held, and target cycling.
- The cheat aimed fine on Eden; r5 did not on MHGURecomp with Modern Weapon Controls. Statically
  r5 matches the cheat for the local hunter; the difference was not found.
- `0x18e50` runs twice on a frame where it is not 1 (the detour's check, then the original's). It
  ends in a virtual call (+0x3c4) not read; inference that it has no side effect.
- The cheats state no licence. r5 ships none of their words but follows their logic; credit stays.
- Ryujinx and Switch hardware have no run.

## Do not

- Write code patches (r4): MHGURecomp applies a Forge patch only when it forms a hook, logs
  "cannot apply to recompiled code: ignored", and the bytes still read back as written.

- Run the tap window out every frame (r2's 0.001 hook): the countdown path also raises bit 24,
  and the user found it ruined aiming.
- Hold the tap window open (write 5.0 every frame): it never runs out, so no target request fires.
- Replace the game's target choice or button mapping: the tester asked for the game's own.
