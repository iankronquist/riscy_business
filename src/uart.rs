use crate::mmio;
use crate::mmio::MmioRegion;
use crate::mutex::Mutex;
use core::cell::RefCell;
use core::fmt::{Error, Write};

pub struct Uart<'a> {
    rgn: Option<MmioRegion<'a>>,
}

pub struct UartLogger<'a> {
    uart: Mutex<Uart<'a>>,
}

pub static LOGGER: Mutex<Uart> = Mutex::new(Uart { rgn: None });

/*
impl<'a> UartLogger<'a> {
    pub unsafe fn bust_lock(&mut self) {
        self.uart.bust_lock();
    }
    pub fn init(&mut self, slc: &'a mut [u8]) {
        self.uart.lock().set_mmio(slc);
    }
}

impl<'a> Write for UartLogger<'a> {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        self.uart.lock().write_str(s)
    }
}
*/

impl<'a> Uart<'a> {
    pub fn new(slc: &'a mut [u8]) -> Self {
        let mut n = Self {
            rgn: Some(MmioRegion::from_slice(slc)),
        };
        n.configure();
        n
    }

    pub fn init(&mut self, slc: &'a mut [u8]) {
        self.rgn = Some(MmioRegion::from_slice(slc));
        self.configure();
    }

    pub fn configure(&mut self) {
        if let Some(rgn) = &self.rgn {
            let lcr: u8 = 1 | (1 << 1);
            rgn.write(3, lcr);

            // Set FIFO control
            rgn.write(2, 1);

            // Enable receiver buffer interrupts.
            rgn.write(1, 1);

            // Set divisor & baud.
            let divisor: u16 = 592;
            let divisor_least: u8 = (divisor & 0xff) as u8;
            let divisor_most: u8 = (divisor >> 8) as u8;

            // Set DLAB
            rgn.write(3, lcr | 1 << 7);

            // Set divisor bits
            rgn.write(0, divisor_least);
            rgn.write(0, divisor_most);

            // Set enabled again.
            rgn.write(3, lcr);
        }
    }

    fn putc(&self, byte: u8) {
        if let Some(rgn) = &self.rgn {
            rgn.write(0, byte);
        }
    }
}

impl<'a> Write for Uart<'a> {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for c in s.bytes() {
            self.putc(c);
        }
        Ok(())
    }
}
