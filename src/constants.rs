/// Useful constants.

pub const KB: usize = 1024;
pub const MB: usize = 1024 * KB;
pub const GB: usize = 1024 * MB;
pub const TB: usize = 1024 * GB;
pub const PAGE_SIZE: usize = 0x1000;
pub const LARGE_PAGE_SIZE: usize = 2 * MB;
pub const HUGE_PAGE_SIZE: usize = GB;
pub const GIANT_PAGE_SIZE: usize = 512 * GB;
pub const LAST_PAGE: usize = 0xffff_ffff_ffff_f000;
pub const LAST_GIANT_PAGE: usize = LAST_PAGE - GIANT_PAGE_SIZE;
pub enum MemoryAccess {
    SupervisorReadable,
    SupervisorWritable,   // implies Readable.
    SupervisorExecutable, // implies Readable but not Writable.
    UserReadable,         // can be read by supervisor if you use copy_from_user.
    UserWritable,         // can be written by supervisor if you use copy_to_user.
    UserExecutable,       // implies not *Writable.
}

pub enum MemoryCacheability {
    Uncached,
    WriteBack,
    WriteThrough,
}
