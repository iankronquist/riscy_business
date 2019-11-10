#![no_std]
#![allow(unused)]
#![feature(asm)]
#![feature(const_fn)]
#![feature(panic_info_message)]

mod mmio;
mod mutex;
mod runtime;
#[macro_use]
mod log;
mod debug;
mod device_tree;
mod uart;
use core::slice;
use device_tree::DeviceTree;

#[no_mangle]
fn rmain(_: usize, device_tree_addr: usize) {
    let mut device_tree = DeviceTree::empty();
    unsafe {
        device_tree =
            DeviceTree::from_address(device_tree_addr).expect("Invalid device tree");
    }
    let uart_base = device_tree
        .find("uart")
        .expect("uart not found in device tree");
    let uart_mem = unsafe { slice::from_raw_parts_mut(uart_base as *mut u8, 0x1000) };
    // FIXME these next bits shouldn't be unsafe. Figure out how to make them safe with RefCell & Mutex.
    unsafe {
        uart::UART.update(uart_mem);
        (*log::LOGGER.lock()).set_sink(&mut uart::UART);
    }
    log!("Hello riscv world");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
