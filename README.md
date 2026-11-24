# Lock-on Target

Keeps the target camera turned to the monster the game targets, so you don't
have to keep pressing the button. Tap your target camera button to turn the
lock off, and tap it again to turn it back on. The target reticle stays
hidden while you play. It does what clarkekoko's "Lock-on Target MHGU Recomp"
and "No Target Reticle" cheats do, as a Forge plugin.

## Requirements

- MHGU 1.4.0 for Nintendo Switch.
- [Fexty's Forge 1.5.1](https://github.com/Fexty12573/forge/releases/tag/v1.5.1).

## Installation

1. Close the emulator.
2. Extract `lock-on-target-r6.zip`.
3. Copy the whole `lock-on-target` folder into your emulator's MHGU mod directory:
   - Eden: `~/.local/share/eden/load/0100770008DD8000/`
   - Ryujinx: `mods/contents/0100770008dd8000/` inside its data directory.
   - MHGURecomp: `mods/` next to the game's executable.
4. Enable `lock-on-target` and launch MHGU.
5. Turn off any lock-on cheat you use. The mod replaces it.

## Usage

| Action | Input |
| --- | --- |
| Turn the lock off, or back on | Tap your target camera button |
| Aim without the lock pulling the camera | Hold R (ZR if you swapped the shoulder buttons) |

The lock pauses while you hold R and picks up again when you let go. The mod
uses the game's own target camera, so the monster it turns to and the
button you tap come from your in-game camera and button settings. There are no
settings of its own.

## Update and removal

Close the emulator before updating or removing the mod. To update, replace the
`lock-on-target` folder with the one from the new ZIP. To remove it, disable
the package or delete its folder from the emulator's mod directory.
