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
            write!(writer, "\n");
        }
    }
    write!(writer, "\n");
}
