use crate::debug;
use crate::mutex::Mutex;
use crate::uart;
use core::fmt::{Arguments, Error, Write};

pub struct Logger<'a> {
    device: Option<&'a mut dyn Write>,
}

impl<'a> Logger<'a> {
    pub const fn new() -> Self {
        Self { device: None }
    }
    pub fn set_sink(&mut self, w: &'a mut dyn Write) {
        self.device = Some(w);
    }
    pub fn log(&mut self, fmt: Arguments) {
        if let Some(writer) = &mut self.device {
            let _ = write!(writer, "{}\n", fmt);
        }
    }
    pub fn hexdump(&mut self, mem: &[u8]) {
        if let Some(writer) = &mut self.device {
            debug::hexdump(writer, mem);
        }
    }
}

pub static LOGGER: Mutex<Logger<'static>> = Mutex::new(Logger::new());

#[macro_export]
macro_rules! log {
    ($fmt:tt, $($arg:tt)*) => {
        {
            use log;
            let mut ul = log::LOGGER.lock();
            ul.log(format_args!($fmt, $($arg)*));
        }
    };
    ($fmt:tt) => {
        {
            use log;
            let mut ul = log::LOGGER.lock();
            ul.log(format_args!($fmt));
        }
    };
}

#[macro_export]
macro_rules! hexdump {
    ($bytes:expr) => {{
        use log;
        let mut ul = log::LOGGER.lock();
        ul.hexdump($bytes);
    }};
}
