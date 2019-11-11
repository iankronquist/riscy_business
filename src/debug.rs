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
    // Not sure about this, this is for debugging, and this causes a trap which
    // can mean trouble.
    /*
    unsafe {
        asm!("ebreak"::::"volatile");
    }
    */
}
