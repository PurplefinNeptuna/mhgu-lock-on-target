//! All-or-nothing installation of the eight hooks. Every recipe is validated before the first
//! write; a later failure restores the sites already touched, in reverse order.
pub const LDR_PC: u32 = 0xe51ff004;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HookSpec {
    pub target: u32,
    pub detour: u32,
    pub expected: [u32; 2],
    /// Only runs code around the original, so it may stack on another plugin's Forge hook.
    pub chains: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstallError {
    InvalidRecipe,
    SignatureMismatch,
    UnsafeInstruction,
    PublicationFailure,
    RestoreFailure,
}

/// Deliberately narrow whitelist: Forge copies the displaced instructions without relocating them.
pub const fn relocatable(op: u32) -> bool {
    if op >> 28 != 14 {
        return false; // unconditional ARM only
    }
    if op & 0xffff_0000 == 0xe92d_0000 {
        return op & 0x8000 == 0 && op & 0x2000 == 0 && op & 0xffff != 0; // push without pc/sp
    }
    if op & 0xffff_f000 == 0xe24d_d000 {
        return true; // sub sp, sp, #imm
    }
    if op & 0xffff_0ff0 == 0xe1a0_0000 {
        return (op >> 12) & 15 != 15 && op & 15 != 15; // mov rd, rm
    }
    if op & 0xffff_0000 == 0xe3a0_0000 {
        return (op >> 12) & 15 != 15; // mov rd, #imm
    }
    false
}

/// A signature window as it may legitimately be found: the original bytes, or for a `chains` window,
/// the original bytes with another plugin's Forge entry hook (`ldr pc, [pc, #-4]` plus its detour
/// address) over the first eight. Forge moves that pair into the next trampoline as it stands, and
/// the pair is position-independent, so the call still reaches the game's function.
pub fn window_matches(byte: impl Fn(u32) -> u8, expected: &[u8; 64], chains: bool) -> bool {
    let hooked = chains && (0..4).all(|i| byte(i) == LDR_PC.to_le_bytes()[i as usize]);
    (if hooked { 8 } else { 0 }..64).all(|i| byte(i) == expected[i as usize])
}

pub const fn overlaps(a: u32, n: u32, b: u32, m: u32) -> bool {
    (a as u64) < b as u64 + m as u64 && (b as u64) < a as u64 + n as u64
}

/// Forge's ARM trampoline: the two displaced words, then a literal jump to target+8.
pub const fn trampoline_ok(code: &[u32; 4], original: &[u32; 2], target: u32) -> bool {
    code[0] == original[0] && code[1] == original[1] && code[2] == LDR_PC && code[3] == target.wrapping_add(8)
}

/// A site may only be restored when each word is the original or our own branch.
pub const fn may_restore(site: &[u32; 2], original: &[u32; 2], detour: u32) -> bool {
    (site[0] == original[0] || site[0] == LDR_PC) && (site[1] == original[1] || site[1] == detour)
}

pub trait Platform {
    fn read(&self, address: u32) -> u32;
    /// Plant one hook and prove its trampoline and branch. False leaves the site for `restore`.
    fn plant(&mut self, index: usize, spec: &HookSpec) -> bool;
    /// Restore the original words and read them back. Foreign bytes are left alone and report false.
    fn restore(&mut self, index: usize, spec: &HookSpec) -> bool;
}

fn matches<P: Platform>(p: &P, h: &HookSpec) -> bool {
    p.read(h.target) == h.expected[0] && p.read(h.target + 4) == h.expected[1]
}

fn validate<P: Platform>(p: &P, hooks: &[HookSpec]) -> Result<(), InstallError> {
    if hooks.is_empty() {
        return Err(InstallError::InvalidRecipe);
    }
    for (i, h) in hooks.iter().enumerate() {
        if h.target == 0 || h.detour == 0 || (h.target | h.detour) & 3 != 0 || h.target as u64 + 8 >= 0x1_0000_0000
            || overlaps(h.target, 8, h.detour, 4)
            || hooks[..i].iter().any(|other| overlaps(h.target, 8, other.target, 8))
        {
            return Err(InstallError::InvalidRecipe);
        }
        if !matches(p, h) {
            return Err(InstallError::SignatureMismatch);
        }
        // A chained foreign branch moves as a pair; the prologue under it is checked by the host tests.
        let chained = h.chains && h.expected[0] == LDR_PC;
        if !chained && (!relocatable(h.expected[0]) || !relocatable(h.expected[1])) {
            return Err(InstallError::UnsafeInstruction);
        }
    }
    Ok(())
}

// Restore every touched entry, even after one restore fails.
fn rollback<P: Platform>(p: &mut P, hooks: &[HookSpec], touched: usize, cause: InstallError) -> InstallError {
    let mut good = true;
    for i in (0..touched).rev() {
        let restored = p.restore(i, &hooks[i]);
        good = restored && matches(p, &hooks[i]) && good;
    }
    if good { cause } else { InstallError::RestoreFailure }
}

pub fn install<P: Platform>(p: &mut P, hooks: &[HookSpec]) -> Result<(), InstallError> {
    validate(p, hooks)?;
    for i in 0..hooks.len() {
        if !matches(p, &hooks[i]) {
            return Err(rollback(p, hooks, i, InstallError::SignatureMismatch));
        }
        // Include this entry before the first write, so a partial failure is restored too.
        if !p.plant(i, &hooks[i]) {
            return Err(rollback(p, hooks, i + 1, InstallError::PublicationFailure));
        }
    }
    Ok(())
}
