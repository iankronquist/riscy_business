#![cfg(not(test))]
use core::sync::atomic;
use core::panic::PanicInfo;

// Prevent recursive panicking.
static HAVE_PANICKED: atomic::AtomicBool = atomic::AtomicBool::new(false);

/// This function is called on panic.
#[panic_handler]
#[no_mangle]
pub fn panic(_info: &PanicInfo) -> ! {
    if !HAVE_PANICKED.compare_and_swap(false, true, atomic::Ordering::Acquire) {
       // unsafe {
       //     logger::bust_locks();
       // }

       // error!("PANIC: {:#?} {:#?}\n", info.message(), info.location());
       // debug::break_point();
    }

    loop {
        //unsafe { //asm!("cli; hlt;"); halt(); }
    }
}

#[no_mangle]
pub fn abort() -> ! {
    panic!("Abort!");
}
