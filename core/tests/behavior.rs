use std::collections::BTreeMap;
use lock_on_target_core::install::*;
use lock_on_target_core::signatures::SIGNATURES;
use lock_on_target_core::*;

/// The cheat as published (lock-on and No Target Reticle), code-type 0 writes to main.
const CHEAT: &str = "
";

fn written() -> BTreeMap<u32, u32> {
    let mut out = BTreeMap::new();
    for p in &PATCHES {
        for (i, &w) in p.patched.iter().enumerate() {
            assert!(out.insert(p.offset + i as u32 * 4, w).is_none(), "patches overlap");
        }
    }
    out
}

#[test]
fn the_patches_write_exactly_the_cheats_words() {
    let cheat: BTreeMap<u32, u32> = CHEAT
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| {
            let f: Vec<_> = l.split(' ').collect();
            assert_eq!(f[0], "04000000");
            (u32::from_str_radix(f[1], 16).unwrap(), u32::from_str_radix(f[2], 16).unwrap())
        })
        .collect();
    assert_eq!(written(), cheat);
    assert!(PATCHES.iter().all(|p| p.original.len() == p.patched.len() && p.original.len() <= MAX_WORDS));
}

fn branch(at: u32, word: u32) -> u32 {
    let imm = ((word & 0xff_ffff) << 8) as i32 >> 6;
    at.wrapping_add(8).wrapping_add(imm as u32)
}

#[test]
fn every_patched_branch_lands_where_the_block_expects() {
    let w = written();
    assert_eq!(branch(0x27e9d8, w[&0x27e9d8]), 0x27ea78, "hold 1-18: the lock block");
    assert_eq!(branch(0x27e9f8, w[&0x27e9f8]), 0x27eb80, "long hold: the shared tail");
    for at in [0x27eaa0, 0x27eab8, 0x27eac8] {
        assert_eq!(branch(at, w[&at]), 0x27eb80, "{at:#x}");
    }
    assert_eq!(branch(0x27ea80, w[&0x27ea80]), 0x65f8ec, "button mask, as the original calls");
    assert_eq!(branch(0x27eab0, w[&0x27eab0]), 0x18ffc, "camera follow check");
    assert_eq!(branch(0x27eac4, w[&0x27eac4]), 0x285ea0, "request-bit setter");
    // `ldr r0, [pc, #0x20]` at 0x27eaa4 reads 0x27eacc; `ldr r0, [pc, r0]` at 0x27eaa8 adds 0x27eab0.
    assert_eq!(0x27eab0u32.wrapping_add(w[&0x27eacc]), 0x1838d90, "GOT slot of sFestaCamera");
}

#[test]
fn the_patches_stay_inside_their_functions_and_off_the_entries() {
    // Tap logic is 568 bytes; the reticle update 372.
    for p in &PATCHES {
        let inside = |start: u32, size: u32| p.offset >= start + 8 && p.offset + p.bytes() <= start + size;
        assert!(inside(TAP_LOGIC_OFFSET, 568) || inside(RETICLE_OFFSET, 372), "{:#x}", p.offset);
    }
    assert_eq!(SIGNATURES[0].0, TAP_LOGIC_OFFSET);
}

#[test]
fn the_reticle_patches_only_change_the_compared_value() {
    for p in &PATCHES[3..] {
        // cmp r0, #imm: same register and opcode, 1 becomes 2.
        assert_eq!((p.original[0], p.patched[0]), (0xe350_0001, 0x0000_0000));
    }
}

#[test]
fn a_chaining_window_accepts_only_a_foreign_forge_entry_hook() {
    let mut expected = [0u8; 64];
    expected.iter_mut().enumerate().for_each(|(i, b)| *b = i as u8);
    let mut hooked = expected;
    hooked[..4].copy_from_slice(&LDR_PC.to_le_bytes());
    hooked[4..8].copy_from_slice(&0x0320_1000u32.to_le_bytes());
    let check = |w: [u8; 64], chains| window_matches(|i| w[i as usize], &expected, chains);
    assert!(check(expected, true) && check(expected, false));
    assert!(check(hooked, true));
    assert!(!check(hooked, false), "every other window stays exact");
    let mut later = hooked;
    later[8] ^= 1;
    assert!(!check(later, true), "past the entry hook the bytes stay exact");
}

// ---- install ----
const BASE: u32 = 0x1000_0000;

#[derive(Default)]
struct Fake {
    memory: BTreeMap<u32, u32>,
    writes: u32,
    fail_apply: Option<usize>,
    restore_failure: bool,
    order: Vec<usize>,
}
impl Fake {
    fn new() -> Self {
        let mut f = Self::default();
        for p in &PATCHES {
            f.put(BASE + p.offset, p.original);
        }
        f
    }
    fn put(&mut self, at: u32, words: &[u32]) {
        for (i, &w) in words.iter().enumerate() {
            self.memory.insert(at + i as u32 * 4, w);
        }
    }
    fn holds(&self, pick: fn(&CodePatch) -> &'static [u32]) -> bool {
        PATCHES.iter().all(|p| pick(p).iter().enumerate().all(|(i, &w)| self.memory[&(BASE + p.offset + i as u32 * 4)] == w))
    }
}
impl Platform for Fake {
    fn read(&self, a: u32) -> u32 {
        self.memory[&a]
    }
    fn apply(&mut self, i: usize, at: u32, p: &CodePatch) -> bool {
        self.writes += 1;
        // A failure can follow a partial write.
        self.put(at, &p.patched[..1]);
        if self.fail_apply == Some(i) {
            return false;
        }
        self.put(at, p.patched);
        true
    }
    fn restore(&mut self, i: usize, at: u32, p: &CodePatch) -> bool {
        self.order.push(i);
        if self.restore_failure {
            return false;
        }
        self.put(at, p.original);
        true
    }
}

#[test]
fn a_clean_install_writes_every_patch() {
    let mut f = Fake::new();
    assert_eq!(install(&mut f, BASE, &PATCHES), Ok(()));
    assert_eq!(f.writes, PATCHES.len() as u32);
    assert!(f.holds(|p| p.patched));
}

#[test]
fn every_write_failure_restores_the_originals_in_reverse() {
    for fail in 0..PATCHES.len() {
        let mut x = Fake::new();
        x.fail_apply = Some(fail);
        assert_eq!(install(&mut x, BASE, &PATCHES), Err(InstallError::PublicationFailure));
        assert!(x.holds(|p| p.original));
        assert_eq!(x.order, (0..=fail).rev().collect::<Vec<_>>());
    }
}

#[test]
fn foreign_bytes_anywhere_refuse_before_any_write() {
    for p in &PATCHES {
        let mut x = Fake::new();
        let last = BASE + p.offset + p.bytes() - 4;
        *x.memory.get_mut(&last).unwrap() ^= 1;
        assert_eq!(install(&mut x, BASE, &PATCHES), Err(InstallError::SignatureMismatch));
        assert_eq!(x.writes, 0);
    }
    let mut x = Fake::new();
    let overlapping = [PATCHES[2], CodePatch { offset: PATCHES[2].offset + 4, ..PATCHES[0] }];
    assert_eq!(install(&mut x, BASE, &overlapping), Err(InstallError::InvalidRecipe));
    assert_eq!(x.writes, 0);
}

#[test]
fn a_failed_rollback_is_never_reported_as_vanilla() {
    let mut x = Fake::new();
    x.fail_apply = Some(PATCHES.len() - 1);
    x.restore_failure = true;
    assert_eq!(install(&mut x, BASE, &PATCHES), Err(InstallError::RestoreFailure));
    assert_eq!(x.order, (0..PATCHES.len()).rev().collect::<Vec<_>>());
}
