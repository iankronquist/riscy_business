use core::cell::RefCell;
use core::slice;
pub struct MmioRegion<'a> {
    rgn: RefCell<&'a mut [u8]>,
}

impl<'a> MmioRegion<'a> {
    pub fn from_slice(slc: &'a mut [u8]) -> Self {
        Self {
            rgn: RefCell::new(slc),
        }
    }
    pub fn write(&self, offset: usize, value: u8) {
        let ptr = (&mut (self.rgn.borrow_mut()[offset])) as *mut u8;
        unsafe { ptr.write_volatile(value) }
    }
    pub fn read(&self, offset: usize, value: u8) -> u8 {
        let ptr = &mut self.rgn.borrow_mut()[offset] as *mut u8;
        unsafe { ptr.read_volatile() }
    }
}
