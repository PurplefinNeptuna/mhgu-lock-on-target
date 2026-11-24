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

## How it works

Design, hooks and native detail live in the skills: [Small hook mods: Village Quests, Shared Urgent Quests, Lock-on Target](../../.agents/skills/mhgu-forge-modding/references/worked/small-hooks.md).

## Open items

- The cheat's author: the MHGU Modding wiki lists only the stream it was found in.
- Target cycling is untested as an item, and Ryujinx and Switch hardware have no run.

The cheat's crosshair twitch (its zoom-in replaying) came from the target request firing about
12 times a second on Eden; fired every frame, the crosshair does not twitch.

## Do not

- Hold the tap window open (write 5.0 every frame): it never runs out, so no target request fires.
- Hold the window for every hunter: the hook also runs for other players' hunters.
- Replace the game's target choice, cycling or button mapping: the tester asked for the game's own.
