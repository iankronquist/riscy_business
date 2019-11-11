use core::cell::RefCell;
use core::slice;

/// A safer abstraction over MMIO.
/// By making MMIO memory a slice we can do a bounds check on every write,
/// which is safer than what the tutorial does, and has a nicer API.
/// We deliberately do not implement any Read/Write traits as MMIO memory is
/// pretty special stuff and you can't just dump anything in there.
pub struct MmioRegion<'a> {
    rgn: RefCell<&'a mut [u8]>,
}

impl<'a> MmioRegion<'a> {
    /// Create a new MmioRegion from a slice of backing memory.
    /// We do not make sure that the underlying page has reasonable
    /// permissions, that is up to the caller.
    pub fn from_slice(slc: &'a mut [u8]) -> Self {
        Self {
            rgn: RefCell::new(slc),
        }
    }
    /// Write an 8 bit value to the MmioRegion at a given offset.
    pub fn write(&self, offset: usize, value: u8) {
        let ptr = (&mut (self.rgn.borrow_mut()[offset])) as *mut u8;
        unsafe { ptr.write_volatile(value) }
    }
    /// Read an 8 bit value from the MmioRegion at a given offset.
    pub fn read(&self, offset: usize, value: u8) -> u8 {
        let ptr = &mut self.rgn.borrow_mut()[offset] as *mut u8;
        unsafe { ptr.read_volatile() }
    }
}
