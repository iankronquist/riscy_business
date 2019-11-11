/// Useful constants.

pub const KB: usize = 1024;
pub const MB: usize = 1024 * KB;
pub const GB: usize = 1024 * MB;
pub const PAGE_SIZE: usize = 0x1000;
//pub const LARGE_PAGE_SIZE: usize = 2 * MB;
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
