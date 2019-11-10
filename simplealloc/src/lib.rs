#![no_std]

extern crate spin;
use core::alloc::{GlobalAlloc, Layout};



#[derive(Debug)]
struct AllocationHeader {
    next_free: *mut Self,
    len: usize,
}

// There must be a slick branch free way of doing this. I checked in hacker's
// delight and didn't find anything.
// 'by' must be power of two
fn round_up(mut roundee: usize, by: usize) -> usize {
    assert!((by & by-1) == 0);
    if roundee == 0 || (roundee & (by-1)) != 0 {
        roundee -= roundee & (by-1);
        roundee += by;
    }
    roundee
}

fn is_ptr_aligned_by(ptr :*mut u8, alignment: usize) -> bool {
    (ptr as usize & (alignment - 1)) == 0
}



const FREE_LIST_END_SENTINEL: *mut AllocationHeader = 1 as *mut AllocationHeader;
const ALLOCATION_ROUNDING_FACTOR: usize = core::mem::size_of::<*const u8>();
const SPLIT_FUDGE_FACTOR: usize = core::mem::size_of::<AllocationHeader>();

impl AllocationHeader {
    unsafe fn from_ptr(ptr: *mut u8) -> *mut Self {
        (ptr  as *mut Self).offset(-1) as *mut Self
    }
    unsafe fn allocation(&mut self) -> *mut u8 {
        (self as *mut Self).offset(1) as *mut u8
    }
    unsafe fn next(&mut self) -> *mut Self {
        self.allocation().add(self.len) as *mut Self
    }
    unsafe fn is_splittable(&self, request_size: usize) -> bool {
        self.len >= core::mem::size_of::<AllocationHeader>() + SPLIT_FUDGE_FACTOR + request_size
    }

    unsafe fn split_free(&mut self, request_size: usize) -> *mut Self {
        assert!(self.is_splittable(request_size));
        assert!(self.is_free());
        assert!(request_size != 0);
        let original_next = self.next();
        let original_len = self.len;
        let mut splitee = self.allocation().add(request_size) as *mut Self;
        (*splitee).len = self.len - request_size - core::mem::size_of::<AllocationHeader>();
        (*splitee).next_free = self.next_free;
        self.len = request_size;
        // We're now used.
        self.next_free = core::ptr::null_mut();

        assert_eq!(self.next(), splitee);
        assert_eq!((*splitee).next(), original_next);

        assert_eq!(self.len + (*splitee).len + core::mem::size_of::<AllocationHeader>(), original_len);

        assert!(!self.is_free());
        assert!((*splitee).is_free());
        splitee
    }
    unsafe fn can_merge(&mut self, other: &Self) -> bool {
        let cother = other as *const Self;
        self.next() as *const Self == cother && cother == self.next_free as *const Self
    }
    unsafe fn merge(&mut self, mut other: &mut Self) {
        assert!(self.can_merge(&other));
        self.len += other.len + core::mem::size_of::<Self>();
        self.next_free = other.next_free;
        // Not strictly necessary but will help with bugs.
        other.next_free = core::ptr::null_mut();
        other.len = 0;
    }
    fn is_free(&self) -> bool {
        !self.next_free.is_null()
    }
    fn mark_used(&mut self) {
        self.next_free = core::ptr::null_mut();
    }
}


struct Allocator {
    arena_start: *mut u8,
    arena_len: usize,
    free_list: *mut AllocationHeader,
}

impl Allocator {
    unsafe fn arena_end(&mut self) -> *mut AllocationHeader {
        // One past the end.
        self.arena_start.add(self.arena_len) as *mut AllocationHeader
    }
    pub fn init(&mut self, base: *mut u8, len: usize) {
        unsafe {
            self.arena_start = base;
            self.arena_len = len;
            self.free_list = self.arena_start as *mut AllocationHeader;
            (*self.free_list).len = self.arena_len - core::mem::size_of::<AllocationHeader>();
            (*self.free_list).next_free = FREE_LIST_END_SENTINEL;
        }
    }

    unsafe fn push_free(&mut self, free: *mut AllocationHeader) {
        assert!(!self.free_list.is_null());
        assert!(!free.is_null());
        (*free).next_free = self.free_list;
        self.free_list = free;
        assert!(!self.free_list.is_null());
        assert!((*free).is_free());
    }

    /*
    fn arena_start(&self) -> *mut AllocationHeader {
        self.arena_start as *mut AllocationHeader
    }
    unsafe fn print(&self, msg: &str) {
        print!("Heap: {} {{\n", msg);
        print!("free_list {:x}\n", self.free_list as u64);
        let mut cur = self.free_list;
        while cur != FREE_LIST_END_SENTINEL {
            print!("{:?}\n", *cur);
            cur = (*cur).next_free;
        }
        print!("}}\n");

        print!("all_blocks {:x}\n", self.free_list as u64);
        let mut blk = self.arena_start();
        while blk != self.arena_end() {
            print!("{:?}\n", *blk);
            blk = (*blk).next();
            println!("{:?} {:?}", blk, self.arena_end());
            assert!(blk <= self.arena_end());
        }
        print!("}}\n");
    }
    */

    unsafe fn alloc_aligned(&mut self, request: usize, alignment: usize) -> *mut u8 {
        const MAX_ALIGN: usize = 1 << 17;
        const MIN_ALIGN: usize = ALLOCATION_ROUNDING_FACTOR;
        assert!(alignment < MAX_ALIGN);
        assert!(alignment >= MIN_ALIGN);
        assert!(alignment > core::mem::size_of::<*mut u8>());
        let ptr = self.alloc(request + alignment);
        if ptr.is_null() {
            return ptr;
        }
        let aligned_ptr = ((ptr as usize & !(alignment - 1)) + alignment) as *mut u8;
        let header_ptr = (aligned_ptr as *mut *mut u8).offset(-1);
        *header_ptr = ptr;
        assert!(is_ptr_aligned_by(aligned_ptr, alignment));
        aligned_ptr
    }

    unsafe fn dealloc_aligned(&mut self, ptr: *mut u8) {
        let header_ptr = (ptr as *mut *mut u8).offset(-1);
        self.dealloc(*header_ptr);
    }

    unsafe fn alloc(&mut self, request: usize) -> *mut u8 {
        let size = round_up(request, ALLOCATION_ROUNDING_FACTOR);
        assert!(size >= request);
        assert!(!self.free_list.is_null());
        let mut cur = self.free_list;
        let mut prev: *mut *mut AllocationHeader = &mut self.free_list as *mut *mut AllocationHeader;
        assert!(!cur.is_null());
        while cur != FREE_LIST_END_SENTINEL {
            if (*cur).len >= size {
                assert!((*cur).is_free());
                if (*cur).is_splittable(size) {
                    let splitee = (*cur).split_free(size);
                    *prev = splitee;
                    assert!((*splitee).is_free());
                } else {
                    *prev = (*cur).next_free;
                }
                (*cur).mark_used();
                assert!(!(*cur).is_free());
                assert!(!self.free_list.is_null());
                return (*cur).allocation();
            }
            prev = &mut (*cur).next_free;
            assert!(!prev.is_null());
            assert!(!(*prev).is_null());
            cur = (*cur).next_free;
            assert!(!cur.is_null());
        }
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8) {
        if ptr.is_null() {
            return
        }
        let header = AllocationHeader::from_ptr(ptr);
        loop {
            let next = (*header).next();
            assert!(next <= self.arena_end());
            if next == self.arena_end() {
                break;
            }
            if (*header).can_merge(&*next) {
                (*header).merge(&mut *next);
            } else {
                break;
            }
        }
        assert!(!(*header).is_free());
        self.push_free(header);
        assert!((*header).is_free());
    }
}

impl Default for Allocator {
    fn default() -> Self {
        Self { arena_start: core::ptr::null_mut(), arena_len: 0, free_list: FREE_LIST_END_SENTINEL }
    }
}

#[derive(Default)]
pub struct GlobalAllocator {
    allocator: spin::Mutex<Allocator>,
}

unsafe impl Sync for GlobalAllocator {}

impl GlobalAllocator {

    pub const fn new() -> Self {
        GlobalAllocator { allocator: spin::Mutex::new(Allocator { arena_start: core::ptr::null_mut(), arena_len: 0, free_list: FREE_LIST_END_SENTINEL }) }
    }

    pub fn init(&self, base: *mut u8, len: usize) {
        self.allocator.lock().init(base, len);
    }
}
unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.align() <= ALLOCATION_ROUNDING_FACTOR {
            (*self).allocator.lock().alloc(layout.size())
        } else {
            (*self).allocator.lock().alloc_aligned(layout.size(), layout.align())
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if layout.align() <= ALLOCATION_ROUNDING_FACTOR {
            (*self).allocator.lock().dealloc(ptr);
        } else {
            (*self).allocator.lock().dealloc_aligned(ptr);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_heap_alloc() {
        let mut heap: Allocator = Default::default();
        unsafe { assert_eq!(heap.alloc(1), core::ptr::null_mut()); }
    }
    #[test]
    fn test_free_null() {
        let mut heap: Allocator = Default::default();
        unsafe { heap.dealloc(core::ptr::null_mut()); }
    }
    #[test]
    fn test_simple_99_alloc_frees() {
        let mut arena = [0u8; 100 * (ALLOCATION_ROUNDING_FACTOR + core::mem::size_of::<AllocationHeader>())];
        let mut heap: Allocator = Default::default();
        heap.init(arena.as_mut_ptr(), arena.len());
        let mut allocd = [core::ptr::null_mut() as *mut u8; 100];
        // Doesn't go to 99 because of split fudge factor
        for i in 0..99 {
            let ptr = unsafe { heap.alloc(ALLOCATION_ROUNDING_FACTOR) };
            assert!(!ptr.is_null());
            allocd[i] = ptr;
        }
        let ptr = unsafe { heap.alloc(ALLOCATION_ROUNDING_FACTOR) };
        assert!(ptr.is_null());
        for ptr in allocd.iter() {
            unsafe { heap.dealloc(*ptr) };
        }
    }

    #[test]
    fn test_simple_100_alloc_frees() {
        // Magic +8 because of split fudge factor
        let mut arena = [0u8; 8 + 100 * (ALLOCATION_ROUNDING_FACTOR + core::mem::size_of::<AllocationHeader>())];
        let mut heap: Allocator = Default::default();
        heap.init(arena.as_mut_ptr(), arena.len());
        let mut allocd = [core::ptr::null_mut() as *mut u8; 100];
        // Doesn't go to 99 because of split fudge factor
        for i in 0..100 {
            let ptr = unsafe { heap.alloc(ALLOCATION_ROUNDING_FACTOR) };
            assert!(!ptr.is_null());
            allocd[i] = ptr;
        }
        let ptr = unsafe { heap.alloc(ALLOCATION_ROUNDING_FACTOR) };
        assert!(ptr.is_null());
        for ptr in allocd.iter() {
            unsafe { heap.dealloc(*ptr) };
        }
    }



    #[test]
    fn allocation_header_basic() {
        let mut headers = [
            AllocationHeader {
                next_free: core::ptr::null_mut(),
                len: core::mem::size_of::<AllocationHeader>(),
            },
            AllocationHeader {
                next_free: core::ptr::null_mut(),
                len: core::mem::size_of::<AllocationHeader>(),
            },
            AllocationHeader {
                next_free: core::ptr::null_mut(),
                len: core::mem::size_of::<AllocationHeader>(),
            },
        ];
        unsafe {
            assert_eq!(headers[0].allocation(), &mut headers[1] as *mut AllocationHeader as *mut u8);
            assert_eq!(headers[0].next(), &mut headers[2] as *mut AllocationHeader);
            assert!(!headers[0].is_splittable(1));
        }
    }

    fn seek_next_null(arr: &[*mut u8], index: usize) -> Option<usize> {
        for i in index..arr.len() {
            if arr[i].is_null() {
                return Some(i);
            }
        }
        for i in 0..index {
            if arr[i].is_null() {
                return Some(i);
            }
        }
        None
    }

    fn seek_next_non_null(arr: &[*mut u8], index: usize) -> Option<usize> {
        for i in index..arr.len() {
            if !arr[i].is_null() {
                return Some(i);
            }
        }
        for i in 0..index {
            if !arr[i].is_null() {
                return Some(i);
            }
        }
        None
    }

    #[test]
    fn random_allocations() {
        use  core::arch::x86_64;
        const ARENA_BYTES: usize = 1000 * (ALLOCATION_ROUNDING_FACTOR + core::mem::size_of::<AllocationHeader>());
        let mut arena = [0u8; ARENA_BYTES];
        let mut heap: Allocator = Default::default();
        heap.init(arena.as_mut_ptr(), arena.len());
        const MAX_ALLOCATIONS: usize = 10000;
        let mut last_allocation = 0;
        let mut total_allocs = 0;
        let mut last_free = 0;
        let mut allocations = [core::ptr::null_mut() as *mut u8; 100];
        const CHANCE_RANDOM_FREE: u64 = 4;
        for _ in 0..MAX_ALLOCATIONS {
            //let i = seek_next_null()
            let mut this_time = 3u64;
            unsafe { x86_64::_rdrand64_step(&mut this_time); } // ignore failure return value -- radically unlikely in testing.
            if total_allocs == allocations.len() || (this_time % CHANCE_RANDOM_FREE) == 0 {
                if let Some(this_free) = seek_next_non_null(&allocations, last_allocation) {
                    unsafe { heap.dealloc(allocations[this_free]); }
                    allocations[this_free] = core::ptr::null_mut();
                    last_free = this_free;
                    total_allocs -= 1;
                } else {
                    assert!(total_allocs == 0);
                }
            }
            let mut alloc_size = 13u16;
            unsafe { x86_64::_rdrand16_step(&mut alloc_size); } // ignore failure return value -- radically unlikely in testing.
            if let Some(this_alloc) = seek_next_null(&allocations, last_free) {
                let ptr = unsafe { heap.alloc(alloc_size as usize) };
                if !ptr.is_null() {
                    allocations[this_alloc] = ptr;
                    last_allocation = this_alloc;
                    total_allocs += 1;
                }
            } else {
                // Shouldn't happen, we should have freed on above.
                assert_eq!(total_allocs, allocations.len());
            }


        }
    }

    #[test]
    fn allocation_header_split_basic() {
        let mut headers = [
            AllocationHeader {
                next_free: FREE_LIST_END_SENTINEL,
                len: core::mem::size_of::<AllocationHeader>() * 4,
            },
            AllocationHeader {
                next_free: FREE_LIST_END_SENTINEL,
                len: core::mem::size_of::<AllocationHeader>() * 2,
            },
            AllocationHeader {
                next_free: FREE_LIST_END_SENTINEL,
                len: core::mem::size_of::<AllocationHeader>(),
            },
            AllocationHeader {
                next_free: FREE_LIST_END_SENTINEL,
                len: core::mem::size_of::<AllocationHeader>(),
            },
        ];
        unsafe {
            assert!(headers[0].is_splittable(0));
            assert!(headers[0].is_splittable(core::mem::size_of::<AllocationHeader>()));
            assert!(headers[1].is_splittable(0));
            assert!(!headers[1].is_splittable(1));
            assert_eq!(headers[0].split_free(core::mem::size_of::<AllocationHeader>()),
            &mut headers[2] as *mut AllocationHeader);
        }
    }

    #[test]
    fn test_round_up() {
        assert_eq!(round_up(0x0, 0x10), 0x10);
        assert_eq!(round_up(0x1, 0x10), 0x10);
        assert_eq!(round_up(0x11, 0x10), 0x20);
        assert_eq!(round_up(0x21, 0x10), 0x30);
        assert_eq!(round_up(0x100, 0x10), 0x100);
    }

    #[test]
    fn allocation_header_commutative() {
        let mut headers = [
            AllocationHeader {
                next_free: core::ptr::null_mut(),
                len: core::mem::size_of::<AllocationHeader>(),
            },
            AllocationHeader {
                next_free: core::ptr::null_mut(),
                len: core::mem::size_of::<AllocationHeader>(),
            },
            AllocationHeader {
                next_free: core::ptr::null_mut(),
                len: core::mem::size_of::<AllocationHeader>(),
            },
        ];
        unsafe {
            assert_eq!(AllocationHeader::from_ptr(headers[0].allocation()), &mut headers[0] as *mut AllocationHeader);
        };

    }

    #[test]
    fn test_aligned_alloc_dealloc() {
        // only supports alignments up to 2**16.
        const MAX_ALIGN_INCLUSIVE: usize = 16usize;
        let mut arena = [0u8; (1 << MAX_ALIGN_INCLUSIVE) * 2];
        let mut heap: Allocator = Default::default();
        heap.init(arena.as_mut_ptr(), arena.len());
        let mut allocations = [core::ptr::null_mut() as *mut u8; 32 - 6];
        for i in 6..MAX_ALIGN_INCLUSIVE {
            let alignment = 1 << i;
            let ptr = unsafe { heap.alloc_aligned(32, alignment) };
            assert!(!ptr.is_null());
            assert!(is_ptr_aligned_by(ptr, alignment));
            allocations[i-1] = ptr;
        }
        for i in 6..MAX_ALIGN_INCLUSIVE {
            unsafe { heap.dealloc_aligned(allocations[i-1]); }
        }
    }
}
