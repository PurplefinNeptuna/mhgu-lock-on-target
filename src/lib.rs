//! Lock-on Target: keeps the target camera on the monster the game targets, and a tap of the
//! target-camera button turns that on or off. A Forge port of clarkekoko's "Lock-on Target MHGU
//! Recomp" cheat: three code patches in the hunter's tap logic, written before the game starts.
//! No settings, so no Plugin UI panel.
#![no_std]
mod patches;
mod platform;

use core::sync::atomic::{AtomicBool, Ordering};

static STARTED: AtomicBool = AtomicBool::new(false);

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
    match patches::install(forge::mem::text_addr()) {
        Ok(()) => log::info!(concat!("[lock-on-target] ", env!("LOT_VERSION"), " loaded")),
        Err(error) => log::error!("[lock-on-target] {error}"),
    }
}
