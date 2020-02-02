use core::fmt::Write;

/// Format the given byte slice to the given writer as an xxd-style hexdump.
pub fn hexdump(writer: &mut dyn Write, mem: &[u8]) {
    for (i, b) in mem.iter().enumerate() {
        // Every 16 bytes, print the current index.
        if ((i % 16) == 0) {
            write!(writer, "{:04x} ", i);
        }
        write!(writer, "{:02x}", b);
        // Every half word, print a space.
        if ((i % 2) == 1) {
            write!(writer, " ");
        }
        // After 16 bytes, print a newline.
        if ((i % 16) == 15) {
            writeln!(writer);
        }
    }
    writeln!(writer);
}

/// Debugger breakpoint. This is not an ebreak or int3 style breakpoint, but
/// rather a convenient place for debuggers to insert breakpoints.
/// This is used in the panic implementation.
#[inline(never)]
pub fn breakpoint() {
    // If we have something like the Bochs magic breakpoint, put it here.
}

/*
#[repr(C)]
struct StackFrame {
    next: *const StackFrame,
    ret: usize,
}

/// Print out the stack trace if there are frame pointers. Does not desymbolize.
pub unsafe fn show_stack_trace() {
    let max_stack_frames = 32;
    let stack_alignment = 4;
    log!("= Stack trace =");
    log!("FP               | PC");
    let mut fp: *const StackFrame;
    asm!("mv $0, fp" : "=r"(fp));
    //let pc: usize;
    //asm!("mv $0, pc" : "=r"(pc));
    //log!("{:x?} | {:x}", fp, pc);
    for _ in 0..max_stack_frames {
        if fp.is_null() {
            log!("fp is null");
            break;
        }
        if !is_kernel_text_address((*fp).ret) {
            log!("next pc is not a kernel text address: {:016x}", (*fp).ret);
            break;
        }
        log!("{:016x?} | {:016x}", (*fp).next, (*fp).ret);
        fp = (*fp).next;
    }
    log!("===============");
}
*/
