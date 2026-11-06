//! Lock-on Target: lets the local hunter's target-camera tap window run out every frame, so the
//! game raises its target request every frame and the camera stays on the monster it targets; the
//! target-camera button cycles targets as the game sets it up. A Forge port of the "Lock-on Target
//! V1" cheat, whose window runs out only between its 12-per-second writes on Eden. One ordinary
//! Forge hook, installed before the game starts. No settings, so no Plugin UI panel.
#![no_std]
mod hooks;
mod platform;

use core::ffi::CStr;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use lock_on_target_core::*;

/// The sFestaUnit singleton once the hook is live; zero leaves the detour passing straight through.
static UNIT: AtomicU32 = AtomicU32::new(0);
static STARTED: AtomicBool = AtomicBool::new(false);
/// Set by the detour the first time it runs the window out; reported once from `forge_onUpdate`.
static HELD: AtomicBool = AtomicBool::new(false);

/// `(hunter)` main+0x27e970. Writes the window for the local hunter, then runs the game's tap logic
/// once with the same argument. Nothing else changes, so it stacks on another plugin's hook here.
extern "C" fn tap_logic(hunter: u32) {
    let unit = UNIT.load(Ordering::Acquire);
    // sFestaUnit lives for the whole process; its local-hunter slot is zero outside a quest.
    if unit != 0 && holds(hunter, platform::word(unit + LOCAL_HUNTER)) {
        platform::write_word(hunter + TAP_TIMER, EXPIRING_TIMER);
        HELD.store(true, Ordering::Relaxed);
    }
    let original = hooks::original(TAP_LOGIC);
    if original != 0 {
        // SAFETY: a published Forge trampoline of the signature-checked `(hunter)` function.
        unsafe { core::mem::transmute::<u32, extern "C" fn(u32)>(original)(hunter) }
    }
}

/// Forge's own singleton table, resolved before plugins load. Raw lookup avoids creating
/// `&'static mut` aliases of a game object.
fn singleton(name: &CStr) -> u32 {
    // SAFETY: a NUL-terminated name; Forge only reads it.
    unsafe { forge::sys::singleton::forge_singleton_getInstanceByName(name.as_ptr().cast()) as u32 }
}

// forge-rs names only the six-byte version prefix of Forge's 70-byte output record.
#[repr(C)]
struct PluginMetadata {
    version: forge::ForgeVersion,
    name: [u8; 64],
}
const _: () = assert!(
    core::mem::size_of::<*const u8>() == 4
        && core::mem::size_of::<PluginMetadata>() == 70
        && core::mem::offset_of!(PluginMetadata, name) == 6
);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn forge_onLoad(params: *mut forge::PluginInitParams) {
    if params.is_null() {
        return;
    }
    let mut name = [0; 64];
    let label = concat!("Lock-on Target ", env!("LOT_VERSION")).as_bytes();
    name[..label.len()].copy_from_slice(label);
    // SAFETY: Forge owns the writable native metadata record for this call.
    unsafe {
        params.cast::<PluginMetadata>().write(PluginMetadata {
            version: forge::ForgeVersion { major: 1, minor: 5, patch: 1 },
            name,
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn forge_onInit() {
    if STARTED.swap(true, Ordering::AcqRel) {
        return;
    }
    forge::log::init().expect("Failed to initialize the Forge logger");
    let unit = singleton(c"sFestaUnit");
    if unit == 0 || !platform::readable(unit + LOCAL_HUNTER, 4) {
        log::error!("[lock-on-target] Inactive: sFestaUnit not found");
        return;
    }
    let mut detours = [0; HOOK_COUNT];
    detours[TAP_LOGIC] = tap_logic as *const () as u32;
    match hooks::install(forge::mem::text_addr(), &detours) {
        Ok(()) => {
            UNIT.store(unit, Ordering::Release);
            log::info!(concat!("[lock-on-target] ", env!("LOT_VERSION"), " loaded"));
        }
        Err(error) => log::error!("[lock-on-target] {error}"),
    }
}

/// One line the first time the window is held, as evidence the hook reached the local hunter.
#[unsafe(no_mangle)]
pub extern "C" fn forge_onUpdate(_delta: f32) {
    static REPORTED: AtomicBool = AtomicBool::new(false);
    if HELD.load(Ordering::Relaxed) && !REPORTED.swap(true, Ordering::Relaxed) {
        log::info!("[lock-on-target] expiring the local hunter's target window");
    }
}
