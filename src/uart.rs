use crate::mmio;
use core::fmt::{Error, Write};

pub struct Uart {
    base: usize,
}

pub static mut UART: Uart = Uart { base: 0 };

impl Uart {
    /*
    pub fn new(base: usize) -> Self {
        let n = Self { base: base };
        n.init();
        n
    }*/

    pub fn init(&mut self, new_base: usize) {
        self.base = new_base;
        let ptr = self.base as *mut u8;
        // This bit is rather directly ripped off of the tutorial.
        unsafe {
            let lcr: u8 = (1 << 0) | (1 << 1);
            ptr.add(3).write_volatile(lcr);

            // Set FIFO control
            ptr.add(2).write_volatile(1 << 0);

            // Enable receiver buffer interrupts.
            ptr.add(1).write_volatile(1 << 0);

            // Set divisor & baud.
            let divisor: u16 = 592;
            let divisor_least: u8 = (divisor & 0xff) as u8;
            let divisor_most: u8 = (divisor >> 8) as u8;

            // Set DLAB
            ptr.add(3).write_volatile(lcr | 1 << 7);

            // more divisor bits
            ptr.add(0).write_volatile(divisor_least);
            ptr.add(1).write_volatile(divisor_most);

            // Set enabled again.
            ptr.add(3).write_volatile(lcr);
        }

    }

    fn putc(&self, byte: u8) {
        unsafe { mmio::write(self.base, 0, byte); }
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for c in s.bytes() {
            self.putc(c);
        }
        Ok(())
    }
}
