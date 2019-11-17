use crate::range::{RangeSet, Range};
use mutex::Mutex;

pub struct PhysicalRange {
    rg: Range,
}

pub struct PhysicalRangeAllocator {
    rs: RangeSet,
}

static PHYS_ALLOC: Mutex<PhysicalRangeAllocator> = Mutex::new(PhysicalRangeAllocator { rs: RangeSet::empty(), });

impl PhysicalRangeAllocator {
    pub fn alloc(&mut self, sz: usize) -> Option<PhysicalRange> {
        if let Some(rg) = self.rs.find(sz) {
            return Some(PhysicalRange { rg: rg });
        }
        None
    }
    // Don't need free, because it's implemented as drop.
}


impl PhysicalRange {
    // Used by mmu which needs to make and unmake ranges to put them in the
    // page tables.
    unsafe fn remake(start: usize, end: usize) -> Self {
        Self { rg: Range::new(start, end) }
    }
    unsafe fn bits(self) -> (usize, usize) {
        (self.rg.start, self.rg.end)
    }
}

impl Drop for PhysicalRange {
    fn drop(&mut self) {
        // Hmm how do I just consume the value?
        PHYS_ALLOC.lock().rs.insert(self.rg.clone());
    }
}
