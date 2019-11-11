use constants::{PAGE_SIZE};

struct AddressSpace { }

struct PageTable { }

// We map all of physical memory from HIGHER_HALF_END - mem_size to HIGHER_HALF_END
const HIGHER_HALF_END: usize = !0;
