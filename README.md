# Lock-on Target

Keeps the target camera locked on the monster the game targets, instead of
turning back to normal after one press. It does what the Lock-on Target V1
cheat does, as a Forge plugin.

## Requirements

- MHGU 1.4.0 for Nintendo Switch.
- [Fexty's Forge 1.5.1](https://github.com/Fexty12573/forge/releases/tag/v1.5.1).

## Installation

1. Close the emulator.
2. Extract `lock-on-target-r1-candidate.zip`.
3. Copy the whole `lock-on-target` folder into your emulator's MHGU mod directory:
   - Eden: `~/.local/share/eden/load/0100770008DD8000/`
   - Ryujinx: `mods/contents/0100770008dd8000/` inside its data directory.
   - MHGURecomp: `mods/` next to the game's executable.
4. Enable `lock-on-target` and launch MHGU.
5. Turn off the Lock-on Target V1 cheat if you use it. The mod replaces it.

## Usage

| Action | Input |
| --- | --- |
| Lock the camera on a monster | Your target camera button |
| Switch to another monster | Your target camera button again |

The mod uses the game's own target camera. Which monster it picks, how it
switches between them and which button does it all come from your in-game
camera and button settings. There are no settings of its own.

## Update and removal

Close the emulator before updating or removing the mod. To update, replace the
`lock-on-target` folder with the one from the new ZIP. To remove it, disable
the package or delete its folder from the emulator's mod directory.

## Compatibility and known issues

While locked, the target crosshair keeps replaying its zoom-in animation, so
it twitches. The cheat does the same.
