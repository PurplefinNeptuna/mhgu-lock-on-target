//! All-or-nothing installation of the code patches. Every patch is checked against its original
//! words before the first write; a later failure restores the patches already written, in reverse.
use crate::CodePatch;

pub const LDR_PC: u32 = 0xe51ff004;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstallError {
    InvalidRecipe,
    SignatureMismatch,
    PublicationFailure,
    RestoreFailure,
}

/// A signature window as it may legitimately be found: the original bytes, or for a `chains` window,
/// the original bytes with another plugin's Forge entry hook (`ldr pc, [pc, #-4]` plus its detour
/// address) over the first eight. The patches never touch a function entry, so such a hook keeps
/// working beside them.
pub fn window_matches(byte: impl Fn(u32) -> u8, expected: &[u8; 64], chains: bool) -> bool {
    let hooked = chains && (0..4).all(|i| byte(i) == LDR_PC.to_le_bytes()[i as usize]);
    (if hooked { 8 } else { 0 }..64).all(|i| byte(i) == expected[i as usize])
}

pub const fn overlaps(a: u32, n: u32, b: u32, m: u32) -> bool {
    (a as u64) < b as u64 + m as u64 && (b as u64) < a as u64 + n as u64
}

pub trait Platform {
    fn read(&self, address: u32) -> u32;
    /// Write one patch at `base + offset` and read it back. False leaves the site for `restore`.
    fn apply(&mut self, index: usize, address: u32, patch: &CodePatch) -> bool;
    /// Put the original words back and read them back.
    fn restore(&mut self, index: usize, address: u32, patch: &CodePatch) -> bool;
}

fn holds<P: Platform>(p: &P, address: u32, words: &[u32]) -> bool {
    words.iter().enumerate().all(|(i, &w)| p.read(address + i as u32 * 4) == w)
}

fn validate<P: Platform>(p: &P, base: u32, patches: &[CodePatch]) -> Result<(), InstallError> {
    if patches.is_empty() || base & 3 != 0 {
        return Err(InstallError::InvalidRecipe);
    }
    for (i, patch) in patches.iter().enumerate() {
        let address = base as u64 + patch.offset as u64;
        if patch.original.is_empty()
            || patch.original.len() != patch.patched.len()
            || patch.offset & 3 != 0
            || address + patch.bytes() as u64 > 0x1_0000_0000
            || patches[..i].iter().any(|other| overlaps(patch.offset, patch.bytes(), other.offset, other.bytes()))
        {
            return Err(InstallError::InvalidRecipe);
        }
        if !holds(p, base + patch.offset, patch.original) {
            return Err(InstallError::SignatureMismatch);
        }
    }
    Ok(())
}

// Restore every touched patch, even after one restore fails.
fn rollback<P: Platform>(p: &mut P, base: u32, patches: &[CodePatch], touched: usize, cause: InstallError) -> InstallError {
    let mut good = true;
    for i in (0..touched).rev() {
        let address = base + patches[i].offset;
        let restored = p.restore(i, address, &patches[i]);
        good = restored && holds(p, address, patches[i].original) && good;
    }
    if good { cause } else { InstallError::RestoreFailure }
}

pub fn install<P: Platform>(p: &mut P, base: u32, patches: &[CodePatch]) -> Result<(), InstallError> {
    validate(p, base, patches)?;
    for i in 0..patches.len() {
        let address = base + patches[i].offset;
        // Include this patch before the write, so a partial write is restored too.
        if !p.apply(i, address, &patches[i]) || !holds(p, address, patches[i].patched) {
            return Err(rollback(p, base, patches, i + 1, InstallError::PublicationFailure));
        }
    }
    Ok(())
}
