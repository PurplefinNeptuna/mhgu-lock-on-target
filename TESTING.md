# Testing Lock-on Target r1

Remove or turn off the "Lock-on Target V1" cheat first, so only the mod holds the window.

1. Start MHGU with the mod and Forge installed, then go on any quest with a large monster.
2. Press your target-camera button (R3 in the tester's settings) to lock on.
3. Check that the camera stays on the monster without pressing again, as the cheat did.
4. Press the button again to cycle to another large monster.
5. Watch for the zoom-in jerk ("spasm"). r1 is expected to still show it; say how often.

Send the emulator's log. The lines that matter:

| Line | Meaning |
| --- | --- |
| `[lock-on-target] r1 loaded` | The hook is installed |
| `[lock-on-target] holding the local hunter's target window` | The hook reached the local hunter at least once (logged once per start) |
| `[lock-on-target] Inactive: ...` | Not installed; the rest of the line says why |
| `[lock-on-target] ERROR: hook rollback failed; restart required` | Restart the game |
