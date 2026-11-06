//! Forge-owned trampoline for the hook. `core::install` validates every recipe first and
//! rolls back in reverse. Forge frees no trampoline, so an original stays callable even if a
//! restore fails and a detour remains reachable; it only ever passes through then.
use crate::platform;
use core::ffi::c_void;
use core::sync::atomic::{AtomicU32, Ordering};
use forge::patch::Patch;
use forge::sys::hook::{Hook, forge_hook_create};
use lock_on_target_core::install::{self, HookSpec, InstallError, LDR_PC, Platform};
use lock_on_target_core::signatures::SIGNATURES;
use lock_on_target_core::{CHAINS, HOOK_COUNT};

// Native Forge ABI: target/detour/original/context/ctx_cell pointers, then has_ctx.
const _: () = assert!(core::mem::size_of::<Hook>() == 24 && core::mem::offset_of!(Hook, ctx_cell) == 16);

/// Published trampolines, by hook index.
static ORIGINALS: [AtomicU32; HOOK_COUNT] = [const { AtomicU32::new(0) }; HOOK_COUNT];

pub fn original(index: usize) -> u32 {
    ORIGINALS[index].load(Ordering::Acquire)
}

fn bytes_of(words: [u32; 2]) -> [u8; 8] {
    let mut out = [0; 8];
    out[..4].copy_from_slice(&words[0].to_le_bytes());
    out[4..].copy_from_slice(&words[1].to_le_bytes());
    out
}

struct ForgePlatform {
    snapshots: [Option<Patch>; HOOK_COUNT],
}

impl Platform for ForgePlatform {
    fn read(&self, address: u32) -> u32 {
        platform::word(address)
    }
    fn plant(&mut self, index: usize, spec: &HookSpec) -> bool {
        let branch = [LDR_PC, spec.detour];
        // A disabled snapshot of the original words, used only for rollback.
        let snapshot = Patch::new(spec.target, &bytes_of(branch), false);
        let captured = snapshot.addr() == spec.target;
        self.snapshots[index] = Some(snapshot);
        if !captured {
            return false;
        }
        let mut original = core::ptr::null();
        // SAFETY: target is a signature-checked ARM entry whose displaced words passed `relocatable`.
        let hook = unsafe { forge_hook_create(spec.target as *const c_void, spec.detour as *const c_void, &mut original) };
        let continuation = original as u32;
        if continuation == 0 || continuation & 3 != 0 || !platform::executable(continuation, 16) {
            return false;
        }
        let code = core::array::from_fn(|i| platform::word(continuation + i as u32 * 4));
        if !install::trampoline_ok(&code, &spec.expected, spec.target) {
            return false;
        }
        ORIGINALS[index].store(continuation, Ordering::Release);
        hook.original == original
            && hook.target as u32 == spec.target
            && hook.detour as u32 == spec.detour
            && platform::word(spec.target) == LDR_PC
            && platform::word(spec.target + 4) == spec.detour
    }
    fn restore(&mut self, index: usize, spec: &HookSpec) -> bool {
        let Some(snapshot) = self.snapshots[index].as_mut() else {
            return false;
        };
        let site = [platform::word(spec.target), platform::word(spec.target + 4)];
        if !install::may_restore(&site, &spec.expected, spec.detour) {
            return false;
        }
        if site != spec.expected {
            snapshot.enable();
            snapshot.disable();
        }
        platform::sync_code(spec.target, 8)
    }
}

pub fn install(base: u32, detours: &[u32; HOOK_COUNT]) -> Result<(), &'static str> {
    if base == 0 || base > 0xfe00_0000 || !platform::correct_program() {
        return Err("Inactive: unsupported program");
    }
    // The GNU note window holds this NSO's complete 16-byte build ID, with its header and name.
    for (i, &(offset, bytes)) in SIGNATURES.iter().enumerate() {
        if !platform::readable(base + offset, 64) {
            return Err("Inactive: signature range unreadable");
        }
        // Only the hook entry may carry another plugin's hook; the other windows are the middle of
        // the same function and the build note, so they stay exact.
        let chains = i < HOOK_COUNT && CHAINS[i];
        if !install::window_matches(|at| platform::byte(base + offset + at), &bytes, chains) {
            return Err("Inactive: build or code signature mismatch");
        }
    }
    let specs: [HookSpec; HOOK_COUNT] = core::array::from_fn(|i| {
        let target = base + SIGNATURES[i].0;
        HookSpec { target, detour: detours[i], expected: [platform::word(target), platform::word(target + 4)], chains: CHAINS[i] }
    });
    let mut forge_platform = ForgePlatform { snapshots: [const { None }; HOOK_COUNT] };
    // Forge calls onInit from sApp_run before the original runs, so no hunter update has started
    // yet. No hot install or removal is exposed.
    let result = install::install(&mut forge_platform, &specs);
    // Dropping a Patch restores its site, which would undo a live hook.
    core::mem::forget(forge_platform);
    result.map_err(|error| match error {
        InstallError::RestoreFailure => "ERROR: hook rollback failed; restart required",
        InstallError::InvalidRecipe | InstallError::SignatureMismatch | InstallError::UnsafeInstruction => {
            "Inactive: hook recipe rejected"
        }
        InstallError::PublicationFailure => "Inactive: Forge hook failed; restored originals",
    })
}
