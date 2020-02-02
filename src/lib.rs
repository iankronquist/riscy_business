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
mod interrupts;
mod math;
mod mmio;
mod mmu;
mod phys;
mod runtime;
mod range;
mod trap;
use core::slice;
use device_tree::DeviceTree;

/// Rust entry point called by the init hardware thread after we enter
/// supervisor mode.
/// * `_hardid` - The current hardware thread id.
/// * `device_tree_addr` - The address of the device tree passed to the kernel.
/// The kernel will use the device tree to configure itself.
#[no_mangle]
pub extern "C" fn rmain(_hartid: usize, device_tree_addr: usize) {
    let mut device_tree = DeviceTree::empty();
    unsafe {
        device_tree = DeviceTree::from_address(device_tree_addr).expect("Invalid device tree");
    }
    // FIXME: I'm not a fan of this API, it is very property specific.
    let (uart_base, uart_size) = device_tree
        .find_regs("uart")
        .expect("uart not found in device tree");
    let uart_mem = unsafe { slice::from_raw_parts_mut(uart_base as *mut u8, uart_size) };
    logger::LOGGER.lock().init(uart_mem);

    let heap_base = heap::get_base() as *mut u8;
    let heap_size = heap::get_size();
    #[cfg(not(test))]
    GLOBAL.init(heap_base, heap_size);
    device_tree.dump();
    let v = vec![1, 2, 3];

    log!("Hello riscv world {:?}", v);
    interrupts::info();
    //unsafe { asm!("ecall"); }
}
