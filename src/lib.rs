//! Lock-on Target: keeps the target camera on the monster the game targets, and a tap of the
//! target-camera button turns that off or on; the target reticle is not drawn. A Forge port of
//! clarkekoko's "Lock-on Target MHGU Recomp" and "No Target Reticle" cheats as two hooks that
//! replace their functions, installed before the game starts. No settings, so no Plugin UI panel.
#![no_std]
mod hooks;
mod platform;

use core::ffi::CStr;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use lock_on_target_core::*;

/// main's base and the sFestaUnit singleton once the hooks are live; zero passes straight through.
static BASE: AtomicU32 = AtomicU32::new(0);
static UNIT: AtomicU32 = AtomicU32::new(0);
static STARTED: AtomicBool = AtomicBool::new(false);
/// The cheat keeps this flag in the hunter's tap window; here it is the plugin's, for the local
/// hunter only. False (lock on) for a new hunter and whenever the game's other path runs.
static LOCK_OFF: AtomicBool = AtomicBool::new(false);
static LAST_HUNTER: AtomicU32 = AtomicU32::new(0);
/// Set the first time the detour raises the target request; reported once from `forge_onUpdate`.
static RAISED: AtomicBool = AtomicBool::new(false);

fn call0(address: u32) -> u32 {
    // SAFETY: a signature-checked game function taking no arguments.
    unsafe { core::mem::transmute::<u32, extern "C" fn() -> u32>(address)() }
}
fn call1(address: u32, a: u32) -> u32 {
    // SAFETY: a signature-checked game function of one argument, passed the object the game passes.
    unsafe { core::mem::transmute::<u32, extern "C" fn(u32) -> u32>(address)(a) }
}
fn call2(address: u32, a: u32, b: u32) {
    // SAFETY: the request setter or clearer, given the live hunter and a bit below 32.
    unsafe { core::mem::transmute::<u32, extern "C" fn(u32, u32)>(address)(a, b) }
}

fn run_original(hunter: u32) {
    let original = hooks::original(TAP_LOGIC);
    if original != 0 {
        // SAFETY: a published Forge trampoline of the signature-checked `(hunter)` function.
        unsafe { core::mem::transmute::<u32, extern "C" fn(u32)>(original)(hunter) }
    }
}

/// `(hunter)` main+0x27e970. The local hunter gets the cheat's version of the function: the long
/// hold, the lock toggle, the target request while the lock is on, then the original's tail.
extern "C" fn tap_logic(hunter: u32) {
    let (base, unit) = (BASE.load(Ordering::Acquire), UNIT.load(Ordering::Acquire));
    // sFestaUnit lives for the whole process; its local-hunter slot is zero outside a quest.
    if base == 0 || unit == 0 || !holds(hunter, platform::word(unit + LOCAL_HUNTER)) {
        return run_original(hunter);
    }
    if LAST_HUNTER.swap(hunter, Ordering::Relaxed) != hunter {
        LOCK_OFF.store(false, Ordering::Relaxed);
    }
    // The GOT slot holds the cell's address; the cell holds sFestaCamera, as at main+0x27e980.
    let camera = platform::word(platform::word(base + CAMERA_GOT));
    if camera == 0 || call1(base + CAMERA_GATE, camera) != 1 {
        // The game's other path, which the cheat leaves alone; it closes the window, so the lock is on.
        LOCK_OFF.store(false, Ordering::Relaxed);
        return run_original(hunter);
    }
    let (mask, aim_mask) = (call0(base + BUTTON_MASK), call0(base + AIM_MASK));
    let held = platform::half(hunter + HELD);
    let pressed = platform::half(hunter + PRESSED);
    let mut lock_off = LOCK_OFF.load(Ordering::Relaxed);
    match step(held, pressed, mask, aim_mask, platform::word(hunter + HOLD_TIME), &mut lock_off) {
        Step::LongHold => {
            call2(base + SET_REQUEST, hunter, HOLD_REQUEST);
            platform::write_word(hunter + TAP_TIMER, 0);
        }
        Step::Raise => {
            if call1(base + CAMERA_FOLLOW, camera) != 0 {
                call2(base + SET_REQUEST, hunter, TARGET_REQUEST);
                RAISED.store(true, Ordering::Relaxed);
            }
        }
        Step::Off | Step::Aiming => {}
    }
    LOCK_OFF.store(lock_off, Ordering::Relaxed);
    // The original's tail at main+0x27eb80.
    let tail = platform::word(platform::word(base + TAIL_GOT));
    if call1(base + TAIL_CHECK, tail) == 1 {
        call2(base + CLEAR_REQUEST, hunter, HOLD_REQUEST);
    }
}

/// `(camera)` main+0x190d4, called only by the reticle update's two compares with 1. Returning 0
/// does what the cheat's compare with 2 does: the reticle is never drawn.
extern "C" fn reticle_check(_camera: u32) -> u32 {
    0
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
    let base = forge::mem::text_addr();
    let mut detours = [0; HOOK_COUNT];
    detours[TAP_LOGIC] = tap_logic as *const () as u32;
    detours[RETICLE_CHECK] = reticle_check as *const () as u32;
    match hooks::install(base, &detours) {
        Ok(()) => {
            UNIT.store(unit, Ordering::Release);
            BASE.store(base, Ordering::Release);
            log::info!(concat!("[lock-on-target] ", env!("LOT_VERSION"), " loaded"));
        }
        Err(error) => log::error!("[lock-on-target] {error}"),
    }
}

/// One line the first time the lock raises the target request, as evidence it reached the local hunter.
#[unsafe(no_mangle)]
pub extern "C" fn forge_onUpdate(_delta: f32) {
    static REPORTED: AtomicBool = AtomicBool::new(false);
    if RAISED.load(Ordering::Relaxed) && !REPORTED.swap(true, Ordering::Relaxed) {
        log::info!("[lock-on-target] holding the target lock");
    }
}
