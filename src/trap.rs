use crate::log;

#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct TrapContext {
    regs: [usize; 32],
    satp: usize,
    trap_sp: usize,
    mhartid: usize,
}

#[no_mangle]
pub static mut BOOTSTRAP_CORE_TRAP_CONTEXT: TrapContext = TrapContext::empty();

impl TrapContext {
    const fn empty() -> Self {
        Self {
            regs: [0; 32],
            satp: 0,
            trap_sp: 0,
            mhartid: 0,
        }
    }
}

#[no_mangle]
pub extern "C" fn rtrap(
    mepc: usize,
    mtval: usize,
    mcause: usize,
    mhartid: usize,
    mstatus: usize,
    trapctx: *mut TrapContext,
) -> usize {
    log!("Trap mcause: {:x} ", mcause);
    log!("Trap mepc: {:x} ", mepc);
    log!("Trap mhartid: {:x} ", mhartid);
    log!("Trap mstatus: {:x} ", mstatus);
    log!("Trap mtval: {:x} ", mtval);
    log!("Trap Context: {:#x?}", unsafe { *trapctx });

    let mtime = 0x0200_bff8 as *const u64;
    let time = unsafe { mtime.read_volatile() };
    log!("Trap time: {:x} ", time);
    mepc
}
