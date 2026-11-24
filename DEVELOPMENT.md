# Developing Lock-on Target

## Current state

r3 DEVELOPMENT candidate at `build/lock-on-target/candidates/r3/delivery/`, not run. It writes the
three code patches of clarkekoko's "Lock-on Target MHGU Recomp" cheat into the hunter's
target-camera tap logic: a tap of the target-camera button flips a lock flag, and while the lock
is on the game raises its single-tap target request every frame. The user ran the cheat and
found it better than r2, which "didnt ruin aiming". The last accepted build is r2 (Eden and
MHGURecomp 0.14.3), at `build/lock-on-target/candidates/r2/`; its release is archived in
`backups/accepted/lock-on-target/r2-before-cheat-patches/`.

The user's words for r3: "i want auto lock target to be based on this cheat" and "this is much
better than our very aggressive lock (didnt ruin aiming)". The game's own target choice and button
mapping stay in use.

## Findings

All addresses are main-relative on MHGU 1.4.0 (build FB08F1D20FD1204F), read from the pinned
NSO. Research notes are in `work/lock-on-target/NOTES.md`.

The tap logic, its window, button words, request bitset and camera gates are in
`.agents/skills/mhgu-understanding/references/input-and-actions.md`, section Target camera tap.

## How it works

Patches, the lock block and install are in `docs/worked/`: [Lock-on Target: patches](docs/worked/patches.md).

## Open items

- r3 has no run: lock on at quest start, tap to turn it off and on, aiming with a bow or bowgun,
  and target cycling.
- The patches run for every hunter the tap logic updates, as the cheat does; multiplayer has no run.
- The cheat states no licence, and r3 ships its words. Ask clarkekoko before publishing.
- Ryujinx and Switch hardware have no run.

## Do not

- Run the tap window out every frame (r2's 0.001 hook): the countdown path also raises bit 24,
  and the user found it ruined aiming.
- Hold the tap window open (write 5.0 every frame): it never runs out, so no target request fires.
- Replace the game's target choice or button mapping: the tester asked for the game's own.
