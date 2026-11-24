//! Native layout for Lock-on Target, a Forge port of clarkekoko's "Lock-on Target MHGU Recomp"
//! cheat. Native writes stay in the plugin. Offsets are main-relative, MHGU 1.4.0 (build
//! FB08F1D20FD1204F).
#![no_std]
pub mod install;
pub mod signatures;

/// One code patch: where it goes, the words it must find there, and the words it writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CodePatch {
    pub offset: u32,
    pub original: &'static [u32],
    pub patched: &'static [u32],
}

impl CodePatch {
    pub const fn bytes(&self) -> u32 {
        self.original.len() as u32 * 4
    }
}

/// The longest patch, in words; sizes the plugin's byte buffer.
pub const MAX_WORDS: usize = 22;

/// The hunter's target-camera tap logic `(hunter)`, main+0x27e970, 568 bytes. The patches turn the
/// tap window f32 at hunter+0x145c into a lock flag (integer 0 or 1). A tap of the target-camera
/// button flips it. While it is 0, only the single-tap target request (bit 0 of hunter+0x1434) is
/// raised, every frame. Bit 24 and the 12-frame tap window are never used.
pub const TAP_LOGIC_OFFSET: u32 = 0x27e970;

pub const PATCHES: [CodePatch; 3] = [
    // A hold of 1 to 18 frames runs the lock block instead of the window reopen at 0x27eb10.
    CodePatch { offset: 0x27e9d8, original: &[0xda00_004c], patched: &[] },
    // After a long hold (request bit 6, flag cleared to 0), skip the window reopen at 0x27eb4c
    // and go straight to the shared tail at 0x27eb80.
    CodePatch { offset: 0x27e9f8, original: &[0xea00_0053], patched: &[] },
    // The lock block, over the window countdown at 0x27ea78. Every branch leaves to 0x27eb80;
    // the old countdown words after 0x27ead0 are no longer reached.
    CodePatch {
        offset: 0x27ea78,
        original: &[
            0xe301_045c, 0xe084_5000, 0xed95_0a00, 0xeeb5_0ac0, 0xeef1_fa10, 0xda00_002e, 0xe59f_0114,
            0xeeb7_0a00, 0xe1a0_1005, 0xe79f_0000, 0xe590_0000, 0xe5d0_606f, 0xe1a0_0004, 0xeb0a_eff4,
            0xeb0f_838d, 0xe301_13f8, 0xe194_10b1, 0xe001_0000, 0xe30f_1fff, 0xe110_0001, 0x0a00_0013,
            0xe1a0_0004,
        ],
        patched: &[
            // r5 = &hunter[+0x145c]; r0 = 0x65f8ec(), the target-camera button mask
            // flag ^= 1 when hunter[+0x13f8] (pressed this frame) has the button
            // flag 1: lock off
            // 0x18ffc([0x184d750] sFestaCamera): non-zero when the camera has a hunter to follow
            // 0x285ea0(hunter, 0): raise the single-tap target request
            // literal: the GOT slot 0x1838d90 that holds &sFestaCamera, relative to 0x27eab0
        ],
    },
];
