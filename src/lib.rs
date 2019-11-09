#![no_std]
#![allow(unused)]
#![feature(asm)]
#![feature(const_fn)]

mod runtime;
mod mmio;
mod mutex;
#[macro_use]
mod log;
mod debug;
mod uart;

#[no_mangle]
fn rmain() {
    unsafe {
        uart::UART.init(0x1000_0000);
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
