use core::fmt::Write;

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
