//! Narrow native boundary. Forge's public API has no mapping query, program-ID query or process
//! cache flush; these libnx-derived SVC wrappers cover that gap. See THIRD_PARTY.md.

// Horizon's pseudo-handle for this process; nothing owns or closes it.
const CURRENT_PROCESS: u32 = 0xffff8001;
const TITLE: u64 = 0x0100770008dd8000;

// Kernel query-memory output, including padding, as the SVC wrapper writes it.
#[repr(C)]
#[derive(Default)]
struct MemoryInfo {
    address: u64,
    size: u64,
    kind: u32,
    attributes: u32,
    permissions: u32,
    ipc: u32,
    device: u32,
    padding: u32,
}
const _: () = assert!(core::mem::size_of::<MemoryInfo>() == 40);

unsafe extern "C" {
    fn lot_svc_query(info: *mut MemoryInfo, page: *mut u32, address: u64) -> u32;
    fn lot_svc_get_info(out: *mut u64, id: u32, handle: u32, sub_id: u64) -> u32;
    fn lot_svc_flush(handle: u32, address: u64, bytes: u64) -> u32;
}
core::arch::global_asm!(include_str!("svc.S"), options(raw));
core::arch::global_asm!(".hidden lot_svc_query\n.hidden lot_svc_get_info\n.hidden lot_svc_flush");

pub fn correct_program() -> bool {
    let mut title = 0;
    // SAFETY: writable output and current-process pseudo-handle match the SVC ABI.
    unsafe { lot_svc_get_info(&mut title, 18, CURRENT_PROCESS, 0) == 0 && title == TITLE }
}

fn mapped(address: u32, bytes: u32, permissions: u32) -> bool {
    if address == 0 || address as u64 + bytes as u64 > 0x1_0000_0000 {
        return false;
    }
    let mut info = MemoryInfo::default();
    let mut page = 0;
    // SAFETY: outputs are initialized structs of the kernel's declared size and alignment.
    unsafe {
        lot_svc_query(&mut info, &mut page, address as u64) == 0
            && info.permissions & permissions == permissions
            && info.address <= address as u64
            && address as u64 + bytes as u64 <= info.address + info.size
    }
}
pub fn readable(address: u32, bytes: u32) -> bool {
    mapped(address, bytes, 1)
}
pub fn executable(address: u32, bytes: u32) -> bool {
    mapped(address, bytes, 5)
}
pub fn sync_code(address: u32, bytes: u32) -> bool {
    // SAFETY: the caller passes a validated code span; the barriers change no memory.
    unsafe {
        let result = lot_svc_flush(CURRENT_PROCESS, address as u64, bytes as u64);
        core::arch::asm!("dsb sy", "isb", options(nostack, preserves_flags));
        result == 0
    }
}
pub fn word(address: u32) -> u32 {
    // SAFETY: every caller checks the mapping first, or holds a native object it was handed.
    unsafe { core::ptr::read_volatile(address as *const u32) }
}
pub fn half(address: u32) -> u16 {
    // SAFETY: as for `word`.
    unsafe { core::ptr::read_volatile(address as *const u16) }
}
pub fn byte(address: u32) -> u8 {
    // SAFETY: as for `word`.
    unsafe { core::ptr::read_volatile(address as *const u8) }
}
pub fn write_word(address: u32, value: u32) {
    // SAFETY: the caller writes a field of the live hunter the game handed to the hook.
    unsafe { core::ptr::write_volatile(address as *mut u32, value) }
}
