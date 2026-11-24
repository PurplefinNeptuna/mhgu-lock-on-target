//! Native layout and portable decisions for Lock-on Target, a Forge port of clarkekoko's
//! "Lock-on Target MHGU Recomp" and "No Target Reticle" cheats as two hooks. Native calls and
//! memory access stay in the plugin. Offsets are main-relative, MHGU 1.4.0 (build FB08F1D20FD1204F).
#![no_std]
pub mod install;
pub mod signatures;

/// Two hooks, the first two entries of `signatures::SIGNATURES`. Both replace their function, so
/// neither may stack on another plugin's hook.
pub const HOOK_COUNT: usize = 2;
pub const TAP_LOGIC: usize = 0;
pub const RETICLE_CHECK: usize = 1;
pub const CHAINS: [bool; HOOK_COUNT] = [false, false];
/// The tap logic runs the original for other hunters; the reticle check never does.
pub const CALLS_ORIGINAL: [bool; HOOK_COUNT] = [true, false];

/// `(hunter)`, main+0x27e970, 568 bytes; its only caller is main+0x2a7b74, which passes the hunter
/// in r0 and reads no result. For the local hunter the detour runs the cheat's version of it.
pub const TAP_LOGIC_OFFSET: u32 = 0x27e970;
/// `(camera) -> 0 or 1`, main+0x190d4: the current camera's byte +0xa7c. Its only callers are the
/// reticle update's compares at main+0x61e9e0 and main+0x61ea28, which draw the reticle on 1. The
/// cheat compares with 2 instead; the detour returns 0, which neither compare takes.
pub const RETICLE_CHECK_OFFSET: u32 = 0x190d4;

/// GOT slot holding the address of sFestaCamera's cell (main+0x184d750), as the tap logic loads it.
pub const CAMERA_GOT: u32 = 0x1838d90;
/// GOT slot holding the address of the cell whose object the tail passes to `TAIL_CHECK`.
pub const TAIL_GOT: u32 = 0x1838da8;

/// `() -> mask`: the target-camera button, 0x200 or 0x400 by the button setting.
pub const BUTTON_MASK: u32 = 0x65f8ec;
/// `(camera) -> u32`: the tap logic's gate; anything but 1 runs the game's other path.
pub const CAMERA_GATE: u32 = 0x18e50;
/// `(camera) -> pointer`: non-zero when the current camera has a hunter to follow.
pub const CAMERA_FOLLOW: u32 = 0x18ffc;
/// `(hunter, bit)`: set / clear a bit of the request bitset at hunter+0x1434.
pub const SET_REQUEST: u32 = 0x285ea0;
pub const CLEAR_REQUEST: u32 = 0x285ec4;
/// `(object) -> u32`: 1 makes the tail clear request bit 6.
pub const TAIL_CHECK: u32 = 0x5584a0;

/// Hunter fields: logical held / pressed-this-frame buttons (u16), hold time and tap window (f32,
/// 1/60 s).
pub const HELD: u32 = 0x13f4;
pub const PRESSED: u32 = 0x13f8;
pub const HOLD_TIME: u32 = 0x1450;
pub const TAP_TIMER: u32 = 0x145c;
/// Request bits: 0 is the single-tap target request, 6 the long hold.
pub const TARGET_REQUEST: u32 = 0;
pub const HOLD_REQUEST: u32 = 6;

/// `sFestaUnit + 0x58`: the local player's hunter.
pub const LOCAL_HUNTER: u32 = 0x58;

/// Run the cheat's version only for the local hunter; every other hunter runs the original.
pub const fn holds(hunter: u32, local_hunter: u32) -> bool {
    hunter != 0 && hunter == local_hunter
}

/// `hold > 18.0` as the game's `vcmpe` + `ble` decides it, on the f32's bits: NaN and negative
/// values are not longer. Bits, so the plugin needs no float register.
pub const fn longer_than_18(bits: u32) -> bool {
    (bits as i32) > 0x4190_0000 && bits <= 0x7f80_0000
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// Held past 18 frames: raise bit 6 and close the window; the lock turns back on.
    LongHold,
    /// Lock on: raise the target request if the camera has a hunter to follow.
    Raise,
    /// Lock off: raise nothing.
    Off,
}

/// One frame of the cheat's tap logic once the camera gate is 1. A tap flips `lock_off`.
pub fn step(held: u16, pressed: u16, mask: u32, hold_bits: u32, lock_off: &mut bool) -> Step {
    if held as u32 & mask & 0xffff != 0 && longer_than_18(hold_bits) {
        *lock_off = false;
        return Step::LongHold;
    }
    if pressed as u32 & mask != 0 {
        *lock_off = !*lock_off;
    }
    if *lock_off { Step::Off } else { Step::Raise }
}
