/// Macros for logging.
/// These are independent of the logger implementation, although we expect
/// logger implementations to be writers wrapped in locks.

#[macro_export]
macro_rules! log {
    ($fmt:tt, $($arg:tt)*) => {
        {
            use core::fmt::Write;
            use crate::logger;
            writeln!(&mut *logger::LOGGER.lock(), "{}", format_args!($fmt, $($arg)*));
        }
    };
    ($fmt:tt) => {
        {
            use core::fmt::Write;
            use crate::logger;
            writeln!(&mut *logger::LOGGER.lock(), "{}", format_args!($fmt));
        }
    };
}

#[macro_export]
macro_rules! hexdump {
    ($bytes:expr) => {{
        use crate::debug;
        use crate::logger;
        use core::fmt::Write;
        debug::hexdump(&mut *logger::LOGGER.lock(), $bytes);
    }};
}
