use crate::log;
use crate::interrupts;
use core::mem;

#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct TrapContext {
    regs: [usize; 32],
    trap_sp: usize,
    hartid: usize,
    satp: usize,
}

#[no_mangle]
pub static mut BOOTSTRAP_CORE_TRAP_CONTEXT: TrapContext = TrapContext::empty();

impl TrapContext {
    const fn empty() -> Self {
        Self {
            regs: [0; 32],
            satp: 0,
            trap_sp: 0,
            hartid: 0,
        }
    }
}

const INTERRUPT_TYPE_MASK: usize = 1 << ((mem::size_of::<usize>() - 1) * 8);
const USER_SOFTWARE_INTERRUPT: usize = 0;
const SUPERVISOR_SOFTWARE_INTERRUPT: usize = 1;
const MACHINE_SOFTWARE_INTERRUPT: usize = 3;
const USER_TIMER_INTERRUPT: usize = 4;
const SUPERVISOR_TIMER_INTERRUPT: usize = 5;
const MACHINE_TIMER_INTERRUPT: usize = 7;
const USER_EXTERNAL_INTERRUPT: usize = 8;
const SUPERVISOR_EXTERNAL_INTERRUPT: usize = 9;
const MACHINE_EXTERNAL_INTERRUPT: usize = 11;

const INSTRUCTION_ADDR_MISALIGNED: usize = INTERRUPT_TYPE_MASK;
const INSTRUCTION_ACCESS_FAULT: usize = 1 | INTERRUPT_TYPE_MASK;
const ILLEGAL_INSTRUCTION: usize = 2 | INTERRUPT_TYPE_MASK;
const BREAKPOINT: usize = 3 | INTERRUPT_TYPE_MASK;
const LOAD_ADDRESS_MISALIGNED: usize = 4 | INTERRUPT_TYPE_MASK;
const LOAD_ACCESS_FAULT: usize = 5 | INTERRUPT_TYPE_MASK;
const STORE_AMO_ADDRESS_MISALIGNED: usize = 6 | INTERRUPT_TYPE_MASK;
const STORE_AMO_ACCESS_FAULT: usize = 7 | INTERRUPT_TYPE_MASK;
const ECALL_FROM_UMODE: usize = 8 | INTERRUPT_TYPE_MASK;
const ECALL_FROM_SMODE: usize = 9 | INTERRUPT_TYPE_MASK;
const ECALL_FROM_MMODE: usize = 11 | INTERRUPT_TYPE_MASK;
const INSTRUCTION_PAGE_FAULT: usize = 12 | INTERRUPT_TYPE_MASK;
const LOAD_PAGE_FAULT: usize = 13 | INTERRUPT_TYPE_MASK;
const STORE_AMO_PAGE_FAULT: usize = 15 | INTERRUPT_TYPE_MASK;

fn trap_reason(mcause: usize) -> &'static str {
    match mcause {
        USER_SOFTWARE_INTERRUPT => "User software interrupt",
        SUPERVISOR_SOFTWARE_INTERRUPT => "Supervisor software interrupt",
        MACHINE_SOFTWARE_INTERRUPT => "Machine software interrupt",
        USER_TIMER_INTERRUPT => "User timer interrupt",
        SUPERVISOR_TIMER_INTERRUPT => "Supervisor timer interrupt",
        MACHINE_TIMER_INTERRUPT => "Machine timer interrupt",
        USER_EXTERNAL_INTERRUPT => "User external interrupt",
        SUPERVISOR_EXTERNAL_INTERRUPT => "Supervisor external interrupt",
        MACHINE_EXTERNAL_INTERRUPT => "Machine external interrupt",

        INSTRUCTION_ADDR_MISALIGNED => "Instruction address misaligned",
        INSTRUCTION_ACCESS_FAULT => "Instruction access fault",
        ILLEGAL_INSTRUCTION => "Illegal instruction",
        BREAKPOINT => "Breakpoint",
        LOAD_ADDRESS_MISALIGNED => "Load address misaligned",
        LOAD_ACCESS_FAULT => "Load access fault",
        STORE_AMO_ADDRESS_MISALIGNED => "Store AMO address misaligned",
        STORE_AMO_ACCESS_FAULT => "Store AMO access fault",
        ECALL_FROM_UMODE => "ECall from UMode",
        ECALL_FROM_SMODE => "ECall from SMode",
        ECALL_FROM_MMODE => "ECall from MMode",
        INSTRUCTION_PAGE_FAULT => "Instruction page fault",
        LOAD_PAGE_FAULT => "Load page fault",
        STORE_AMO_PAGE_FAULT => "Store AMO page fault",
        _ => "Unknown reason",
    }
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rtrap(
    mepc: usize,
    mtval: usize,
    mcause: usize,
    mhartid: usize,
    mstatus: usize,
    trapctx: *mut TrapContext,
) -> usize {
    log!("Trap mepc: {:x}", mepc);
    log!("Trap mtval: {:x}", mtval);
    log!("Trap mcause: {:x}: {}", mcause, trap_reason(mcause));
    log!("Trap mhartid: {:x}", mhartid);
    log!("Trap mstatus: {:x}", mstatus);
    log!("Trap Context: {:#x?}", unsafe { *trapctx });
    match mcause {
        SUPERVISOR_TIMER_INTERRUPT => {
            interrupts::disable();
            let mut stime: usize;
            let mut stimecmp: usize;
            //unsafe {
            //    //asm!("csrr mtime, $0; csrr mtimecmp, $1" : "=r"(mtime), "=r"(mtimecmp):);
            //    mtimecmp = mtime + 0x10_000_000;
            //    //asm!("csrw $0, mtimecmp" : : "r"(mtimecmp));
            //}
        }
        _ => {
            panic!(
                "Unknown trap! Reason: {:x}: {}",
                mcause,
                trap_reason(mcause)
            );
        }
    }
    let mut cycle: usize;
    unsafe { asm!("rdcycle $0" : "=r"(cycle)); }

    log!("Trap cycle: {:x} ", cycle);

    //let mut time: usize;
    let mtime = 0x0200_bff8 as *const u64;
    //unsafe { asm!("rdtime $0" : "=r"(time)); }
    let time = unsafe { mtime.read_volatile() };
    log!("Trap time: {:x} ", time);
    mepc
}
