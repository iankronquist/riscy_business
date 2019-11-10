#![cfg(not(test))]
use crate::log;
use core::panic::PanicInfo;
use core::sync::atomic;

// Prevent recursive panicking.
static HAVE_PANICKED: atomic::AtomicBool = atomic::AtomicBool::new(false);

/// This function is called on panic.
#[panic_handler]
#[no_mangle]
pub fn panic(info: &PanicInfo) -> ! {
    if !HAVE_PANICKED.compare_and_swap(false, true, atomic::Ordering::SeqCst) {
        unsafe {
            log::LOGGER.bust_lock();
        }

        log!("PANIC: {:#?} {:#?}\n", info.message(), info.location());
        // debug::break_point();
    }

    loop {
        unsafe {
            asm!("csrw mie, zero; wfi;"::::"volatile");
        }
    }
}

#[no_mangle]
pub fn abort() -> ! {
    panic!("Abort!");
}

#[alloc_error_handler]
fn alloc_error_handler(_: core::alloc::Layout) -> ! {
    panic!("OOM!");
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn eh_personality() {
    //error!("PANIC: eh_personality\n");
}
