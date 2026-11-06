//! Native layout and portable decisions for Lock-on Target. Native calls and memory access stay
//! in the plugin. Offsets are main-relative, MHGU 1.4.0 (build FB08F1D20FD1204F).
#![no_std]
pub mod install;
pub mod signatures;

/// One hook: the hunter's target-camera tap logic, the first entry of `signatures::SIGNATURES`.
pub const HOOK_COUNT: usize = 1;
pub const TAP_LOGIC: usize = 0;
/// The detour only writes a field of its argument, then calls the original once with the same
/// argument; the function returns nothing its only caller reads. So it may stack on another
/// plugin's Forge hook over the same entry.
pub const CHAINS: [bool; HOOK_COUNT] = [true];

/// `(hunter)`, main+0x27e970. Its only caller is main+0x2a7b74 in the hunter update main+0x2a79cc,
/// which passes the hunter in r0. It counts the tap window at `TAP_TIMER` down and turns presses
/// of the target-camera button into request bits at hunter+0x1434: while the window is open, a
/// press raises the request a single tap raises when the window runs out.
pub const TAP_LOGIC_OFFSET: u32 = 0x27e970;

/// f32 tap window, in frames. A press with the window closed opens it at 12.0.
pub const TAP_TIMER: u32 = 0x145c;
/// 5.0f, the value the "Lock-on Target V1" cheat keeps there. Written as bits so the plugin never
/// needs a float register.
pub const HELD_TIMER: u32 = 0x40a0_0000;
/// `sFestaUnit + 0x58`: the local player's hunter, the chain the cheat and Pixel Cheats' "Hunter
/// Speed" both read.
pub const LOCAL_HUNTER: u32 = 0x58;

/// Hold the window open only for the local hunter; every other hunter runs unchanged.
pub const fn holds(hunter: u32, local_hunter: u32) -> bool {
    hunter != 0 && hunter == local_hunter
}
