#![no_std]
#![allow(unused)]
#![feature(asm)]
#![feature(const_fn)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(lang_items)] // Only here to make clippy happy...

#[macro_use]
extern crate alloc;
extern crate simplealloc;
extern crate simplespin as mutex;
use alloc::vec;

#[cfg(not(test))]
#[global_allocator]
static GLOBAL: simplealloc::GlobalAllocator = simplealloc::GlobalAllocator::new();

#[macro_use]
mod log;
mod uart;
use uart as logger;

mod constants;
mod debug;
mod device_tree;
mod heap;
mod mmio;
mod runtime;
mod trap;
use core::slice;
use device_tree::DeviceTree;

#[no_mangle]
pub extern "C" fn rmain(_: usize, device_tree_addr: usize) {
    let mut device_tree = DeviceTree::empty();
    unsafe {
        device_tree = DeviceTree::from_address(device_tree_addr).expect("Invalid device tree");
    }
    let uart_base = device_tree
        .find("uart")
        .expect("uart not found in device tree");
    let uart_size = 0x100;
    //let (uart_base, uart_size) = device_tree
    //    .find_regs("uart")
    //    .expect("uart not found in device tree");
    let uart_mem = unsafe { slice::from_raw_parts_mut(uart_base as *mut u8, uart_size) };
    logger::LOGGER.lock().init(uart_mem);

    let heap_base = heap::get_base() as *mut u8;
    let heap_size = heap::get_size();
    #[cfg(not(test))]
    GLOBAL.init(heap_base, heap_size);
    device_tree.dump();
    let v = vec![1, 2, 3];

    log!("Hello riscv world {:?}", v);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
