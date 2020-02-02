/// Useful math functions.

pub fn align_down_by(n: usize, alignment: usize) -> usize {
    n & !(alignment - 1)
}

pub fn align_up_by(n: usize, alignment: usize) -> usize {
    if is_aligned_by(n, alignment) {
        n
    } else {
        align_down_by(n + alignment, alignment)
    }
}

extern "C" {
    pub static __kernel_end: u8;
    pub static __kernel_start: u8;
    pub static __text_start: u8;
    pub static __text_end: u8;
}

pub fn is_power_of_two(x: usize) -> bool {
    x & (x - 1) == 0
}

pub fn is_aligned_by(ptr: usize, alignment: usize) -> bool {
    assert!(is_power_of_two(alignment));
    (ptr & (alignment - 1)) == 0
}
