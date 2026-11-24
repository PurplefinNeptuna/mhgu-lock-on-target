# Testing Lock-on Target r6

Turn off every lock-on and reticle cheat first (clarkekoko's included), so only the mod changes the lock.

1. Start MHGU with the mod and Forge installed, then go on any quest with a large monster.
2. Without pressing anything, check whether the camera turns to the targeted monster and stays on it.
3. Tap your target-camera button (R3 in the tester's settings). The lock should turn off; tap it again to turn it back on.
4. With the lock on, hold R to aim (the Insect Glaive's kinsect aim, or a bow) and say whether the aim moves on its own. Let go of R and check that the camera turns back to the monster.
5. Check that no target reticle shows, and that the camera does not twitch.
6. Say whether you can still switch to another large monster, and how.

Send the emulator's log. Eden's Forge config (`level = error`) hides the info line below; MHGURecomp shows it. The lines that matter:

| Line | Meaning |
| --- | --- |
| `[lock-on-target] r6 loaded` | Both hooks are installed |
| `[lock-on-target] holding the target lock` | The lock reached the local hunter at least once (logged once per start) |
| `[lock-on-target] Inactive: ...` | Not installed; the rest of the line says why |
| `code patch at ... cannot apply to recompiled code` (MHGURecomp) | Must not appear for this mod |
| `[lock-on-target] ERROR: hook rollback failed; restart required` | Restart the game |
