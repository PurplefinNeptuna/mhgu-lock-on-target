# Tests

`core/tests/behavior.rs` runs on the host (`./build.sh test`) with no game, emulator, ARM code
or Forge: the local-hunter rule, the held value, the signature table and the tap logic's timer
writes inside it, and install rollback and chaining. `scripts/verify.py` then checks the staged
NRO: signatures against the pinned NSO, imports, exports, text relocations and GE-affecting
instructions. Neither run touches a game, so runtime behaviour stays separate evidence.
