use std::collections::BTreeMap;
use lock_on_target_core::install::*;
use lock_on_target_core::signatures::SIGNATURES;
use lock_on_target_core::*;

#[test]
fn only_the_local_hunter_runs_the_cheats_version() {
    assert!(holds(0x1660_0060, 0x1660_0060));
    assert!(!holds(0x1660_0060, 0x1770_0000), "another player's hunter runs the original");
    assert!(!holds(0, 0), "no hunter outside a quest or before sFestaUnit fills its slot");
}

#[test]
fn a_hold_is_long_only_past_18_frames() {
    for (hold, long) in [(18.0f32, false), (18.001, true), (1.0, false), (0.0, false), (-0.0, false), (-30.0, false), (300.0, true), (f32::INFINITY, true), (f32::NAN, false)] {
        assert_eq!(longer_than_18(hold.to_bits()), long, "{hold}");
    }
}

const MASK: u32 = 0x200;
fn bits(f: f32) -> u32 {
    f.to_bits()
}

#[test]
fn a_tap_flips_the_lock_and_the_lock_starts_on() {
    let mut off = false;
    assert_eq!(step(0, 0, MASK, 0, &mut off), Step::Raise, "on with no press");
    assert_eq!(step(0x200, 0x200, MASK, bits(0.0), &mut off), Step::Off, "a tap turns it off");
    assert_eq!(step(0x200, 0, MASK, bits(5.0), &mut off), Step::Off, "still held, no new press");
    assert_eq!(step(0, 0, MASK, 0, &mut off), Step::Off);
    assert_eq!(step(0x200, 0x200, MASK, bits(0.0), &mut off), Step::Raise, "a second tap turns it on");
    assert_eq!(step(0x400, 0x400, MASK, bits(0.0), &mut off), Step::Raise, "another button is not the target button");
}

#[test]
fn a_long_hold_raises_bit_6_and_turns_the_lock_on() {
    let mut off = true;
    assert_eq!(step(0x200, 0, MASK, bits(18.5), &mut off), Step::LongHold);
    assert!(!off);
    // A long hold time with the button no longer held is a normal frame.
    let mut off = true;
    assert_eq!(step(0, 0, MASK, bits(40.0), &mut off), Step::Off);
}

#[test]
fn the_signature_table_starts_with_the_hook_targets() {
    assert_eq!(SIGNATURES[TAP_LOGIC].0, TAP_LOGIC_OFFSET);
    assert_eq!(SIGNATURES[RETICLE_CHECK].0, RETICLE_CHECK_OFFSET);
    assert_eq!(SIGNATURES.len(), HOOK_COUNT + 4);
    assert!(CHAINS.iter().all(|&c| !c), "both hooks replace their function");
}

fn word(window: &[u8; 64], index: usize) -> u32 {
    u32::from_le_bytes(window[index * 4..index * 4 + 4].try_into().unwrap())
}

#[test]
fn the_windows_hold_the_fields_and_calls_the_detour_reproduces() {
    // Tap logic entry: push {r4-r7, r11, lr}; mov r4, r0; movw r0, #0x13f4 (held).
    let entry = &SIGNATURES[TAP_LOGIC].1;
    assert_eq!([word(entry, 0), word(entry, 1)], [0xe92d_48f0, 0xe1a0_4000]);
    assert_eq!(word(entry, 2), 0xe300_0000 | (HELD & 0xf000) << 4 | HELD & 0xfff);
    // Reticle check: push {r4, r5, r11, lr}; ldr r5, [r0, #0x54].
    let reticle = &SIGNATURES[RETICLE_CHECK].1;
    assert_eq!([word(reticle, 0), word(reticle, 1)], [0xe92d_4830, 0xe590_5054]);
    // Tail at 0x27eb80: `ldr r0, [pc, #0x28]` reads 0x27ebb0; `ldr r0, [pc, r0]` at 0x27eb84 adds 0x27eb8c.
    let (offset, tail) = &SIGNATURES[4];
    assert_eq!(*offset, 0x27eb80);
    assert_eq!(0x27eb8cu32.wrapping_add(word(tail, 12)), TAIL_GOT);
    // At 0x27e980 `ldr r0, [pc, #0x220]` reads 0x27eba8; `ldr r0, [pc, r0]` at 0x27e984 adds 0x27e98c.
    assert_eq!(0x27e98cu32.wrapping_add(word(tail, 10)), CAMERA_GOT);
}

#[test]
fn only_calling_hooks_need_relocatable_words() {
    for (i, (offset, bytes)) in SIGNATURES[..HOOK_COUNT].iter().enumerate() {
        let ok = bytes[..8].chunks(4).all(|w| relocatable(u32::from_le_bytes(w.try_into().unwrap())));
        assert!(ok || !CALLS_ORIGINAL[i], "{offset:#x}");
    }
}

#[test]
fn a_replacing_hook_with_no_trampoline_skips_the_relocation_check_but_not_the_bytes() {
    let mut x = Fake::new();
    let mut hs = RECIPES;
    hs[0].expected[1] = 0xe590_5054;
    x.memory.insert(hs[0].target + 4, hs[0].expected[1]);
    assert_eq!(install(&mut x, &hs), Err(InstallError::UnsafeInstruction));
    hs[0].calls_original = false;
    assert_eq!(install(&mut x, &hs), Ok(()));
    let mut y = Fake::new();
    y.memory.insert(hs[0].target + 4, hs[0].expected[1]);
    hs[0].chains = true;
    assert_eq!(install(&mut y, &hs), Err(InstallError::InvalidRecipe), "a replacing hook never chains");
}

#[test]
fn the_whitelist_refuses_writes_to_pc_and_pc_relative_loads() {
    assert!(relocatable(0xe3a00000), "mov r0, #0");
    assert!(!relocatable(0xe3a0f000), "mov pc, #0");
    assert!(!relocatable(0xe59f0034), "ldr r0, [pc, #0x34]");
    assert!(!relocatable(0x03a00000), "conditional");
}

#[test]
fn the_trampoline_must_hold_the_displaced_words_and_jump_to_target_plus_8() {
    let original = [0xe92d48f0, 0xe1a04001];
    assert!(trampoline_ok(&[original[0], original[1], LDR_PC, 0x1008], &original, 0x1000));
    assert!(!trampoline_ok(&[original[0], original[1], LDR_PC, 0x1004], &original, 0x1000));
    assert!(!trampoline_ok(&[original[1], original[0], LDR_PC, 0x1008], &original, 0x1000));
}

#[test]
fn only_our_branch_or_the_original_may_be_restored() {
    let original = [0xe92d48f0, 0xe1a04001];
    assert!(may_restore(&original, &original, 0x3000));
    assert!(may_restore(&[LDR_PC, 0x3000], &original, 0x3000));
    assert!(!may_restore(&[LDR_PC, 0x4000], &original, 0x3000));
    assert!(!may_restore(&[0xe3a00000, original[1]], &original, 0x3000));
}

// ---- install ----
const RECIPES: [HookSpec; 3] = [
    HookSpec { target: 0x1000, detour: 0x3000, expected: [0xe92d48f0, 0xe1a04001], chains: false, calls_original: true },
    HookSpec { target: 0x1100, detour: 0x3100, expected: [0xe1a02000, 0xe3a00000], chains: false, calls_original: true },
    HookSpec { target: 0x1200, detour: 0x3200, expected: [0xe92d4010, 0xe1a04000], chains: false, calls_original: true },
];
#[derive(Default)]
struct Fake {
    memory: BTreeMap<u32, u32>,
    plants: u32,
    fail_plant: Option<usize>,
    restore_failure: bool,
    order: Vec<usize>,
}
impl Fake {
    fn new() -> Self {
        let mut f = Self::default();
        for h in RECIPES {
            f.memory.insert(h.target, h.expected[0]);
            f.memory.insert(h.target + 4, h.expected[1]);
        }
        f
    }
    fn baseline(&self) -> bool {
        RECIPES.iter().all(|h| self.memory[&h.target] == h.expected[0] && self.memory[&(h.target + 4)] == h.expected[1])
    }
}
impl Platform for Fake {
    fn read(&self, a: u32) -> u32 {
        self.memory[&a]
    }
    fn plant(&mut self, i: usize, h: &HookSpec) -> bool {
        self.plants += 1;
        self.memory.insert(h.target + 4, h.detour); // A failure can follow a partial write.
        if self.fail_plant == Some(i) {
            return false;
        }
        self.memory.insert(h.target, LDR_PC);
        true
    }
    fn restore(&mut self, i: usize, h: &HookSpec) -> bool {
        self.order.push(i);
        if self.restore_failure {
            return false;
        }
        self.memory.insert(h.target, h.expected[0]);
        self.memory.insert(h.target + 4, h.expected[1]);
        true
    }
}

#[test]
fn every_publication_failure_restores_the_baseline_in_reverse() {
    let mut f = Fake::new();
    assert_eq!(install(&mut f, &RECIPES), Ok(()));
    assert_eq!(f.plants, 3);
    for fail in 0..3 {
        let mut x = Fake::new();
        x.fail_plant = Some(fail);
        assert_eq!(install(&mut x, &RECIPES), Err(InstallError::PublicationFailure));
        assert!(x.baseline());
        assert_eq!(x.order, (0..=fail).rev().collect::<Vec<_>>());
    }
}

#[test]
fn rejection_happens_before_any_write() {
    for scenario in 0..5 {
        let mut x = Fake::new();
        let mut hs = RECIPES;
        match scenario {
            0 => hs[0].target |= 1,
            1 => hs[1].target = hs[0].target + 4,
            2 => hs[0].detour |= 1,
            3 => hs[0].expected[0] ^= 1,
            4 => {
                hs[0].expected[1] = 0xe59f0034;
                x.memory.insert(0x1004, hs[0].expected[1]);
            }
            _ => {}
        }
        assert!(install(&mut x, &hs).is_err(), "scenario {scenario}");
        assert_eq!(x.plants, 0);
    }
}

#[test]
fn a_failed_rollback_is_never_reported_as_vanilla() {
    let mut x = Fake::new();
    x.fail_plant = Some(2);
    x.restore_failure = true;
    assert_eq!(install(&mut x, &RECIPES), Err(InstallError::RestoreFailure));
    assert_eq!(x.order, [2, 1, 0]);
}

#[test]
fn a_chaining_hook_accepts_only_a_foreign_forge_entry_hook() {
    let mut expected = [0u8; 64];
    expected.iter_mut().enumerate().for_each(|(i, b)| *b = i as u8);
    let mut hooked = expected;
    hooked[..4].copy_from_slice(&LDR_PC.to_le_bytes());
    hooked[4..8].copy_from_slice(&0x0320_1000u32.to_le_bytes());
    let check = |w: [u8; 64], chains| window_matches(|i| w[i as usize], &expected, chains);
    assert!(check(expected, true) && check(expected, false));
    assert!(check(hooked, true));
    assert!(!check(hooked, false), "a hook that replaces behaviour stays exact");
    let mut stray = expected;
    stray[0] ^= 1;
    assert!(!check(stray, true));
    let mut later = hooked;
    later[8] ^= 1;
    assert!(!check(later, true), "past the entry hook the bytes stay exact");
}

#[test]
fn a_foreign_entry_hook_passes_the_relocation_check_only_when_chaining() {
    let mut x = Fake::new();
    let mut hs = RECIPES;
    hs[0].expected = [LDR_PC, 0x0320_1000];
    x.memory.insert(hs[0].target, LDR_PC);
    x.memory.insert(hs[0].target + 4, 0x0320_1000);
    assert_eq!(install(&mut x, &hs), Err(InstallError::UnsafeInstruction));
    assert_eq!(x.plants, 0);
    hs[0].chains = true;
    assert_eq!(install(&mut x, &hs), Ok(()));
}
