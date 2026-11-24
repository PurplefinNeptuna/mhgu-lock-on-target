//! Forge patches for the tap logic. `core::install` checks every original first and rolls back in
//! reverse. Forge's patch writes are a plain copy, so each write is flushed here before it runs.
use crate::platform;
use forge::patch::Patch;
use lock_on_target_core::install::{self, InstallError, Platform};
use lock_on_target_core::signatures::SIGNATURES;
use lock_on_target_core::{CodePatch, MAX_WORDS, PATCHES};

fn bytes_of(words: &[u32], out: &mut [u8; MAX_WORDS * 4]) -> usize {
    for (i, w) in words.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&w.to_le_bytes());
    }
    words.len() * 4
}

struct ForgePlatform {
    patches: [Option<Patch>; PATCHES.len()],
}

impl Platform for ForgePlatform {
    fn read(&self, address: u32) -> u32 {
        platform::word(address)
    }
    fn apply(&mut self, index: usize, address: u32, patch: &CodePatch) -> bool {
        let (mut original, mut patched) = ([0; MAX_WORDS * 4], [0; MAX_WORDS * 4]);
        let length = bytes_of(patch.original, &mut original);
        bytes_of(patch.patched, &mut patched);
        let mut forge_patch = Patch::new(address, &patched[..length], false);
        // Forge snapshots the live bytes; they must be the checked originals before we write.
        let captured = forge_patch.addr() == address && forge_patch.original_bytes() == &original[..length];
        if captured {
            forge_patch.enable();
        }
        self.patches[index] = Some(forge_patch);
        captured && platform::sync_code(address, length as u32)
    }
    fn restore(&mut self, index: usize, address: u32, patch: &CodePatch) -> bool {
        let Some(forge_patch) = self.patches[index].as_mut() else {
            return false;
        };
        forge_patch.disable();
        platform::sync_code(address, patch.bytes())
    }
}

pub fn install(base: u32) -> Result<(), &'static str> {
    if base == 0 || base > 0xfe00_0000 || !platform::correct_program() {
        return Err("Inactive: unsupported program");
    }
    // The GNU note window holds this NSO's complete 16-byte build ID, with its header and name.
    for (i, &(offset, bytes)) in SIGNATURES.iter().enumerate() {
        if !platform::readable(base + offset, 64) {
            return Err("Inactive: signature range unreadable");
        }
        // Only the tap logic's entry may carry another plugin's hook; the rest stays exact.
        if !install::window_matches(|at| platform::byte(base + offset + at), &bytes, i == 0) {
            return Err("Inactive: build or code signature mismatch");
        }
    }
    let mut forge_platform = ForgePlatform { patches: [const { None }; PATCHES.len()] };
    // Forge calls onInit from sApp_run before the original runs, so no hunter update has started
    // yet. No hot install or removal is exposed.
    let result = install::install(&mut forge_platform, base, &PATCHES);
    // Dropping a Patch restores its site, which would undo a live patch.
    core::mem::forget(forge_platform);
    result.map_err(|error| match error {
        InstallError::RestoreFailure => "ERROR: patch rollback failed; restart required",
        InstallError::InvalidRecipe | InstallError::SignatureMismatch => "Inactive: patch recipe rejected",
        InstallError::PublicationFailure => "Inactive: patch failed; restored originals",
    })
}
