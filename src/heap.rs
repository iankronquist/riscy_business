use super::constants::{MB, PAGE_SIZE};
extern "C" {
    static mut __kernel_end: u8;
}

fn is_power_of_two(x: usize) -> bool {
    x & (x - 1) == 0
}

fn is_aligned_by(ptr: usize, alignment: usize) -> bool {
    assert!(is_power_of_two(alignment));
    (ptr & (alignment - 1)) == 0
}

fn round_next(start: usize, alignment: usize) -> usize {
    assert!(is_power_of_two(alignment));
    if is_aligned_by(start, alignment) {
        return start;
    }
    let r = (start & !(alignment - 1)) + alignment;
    assert!(is_aligned_by(r, alignment));
    r
}

pub fn get_base() -> usize {
    unsafe { round_next(&mut __kernel_end as *mut u8 as usize, PAGE_SIZE) }
}

pub fn get_size() -> usize {
    16 * MB
}
