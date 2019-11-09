use crate::mutex::Mutex;
use core::fmt::{Write, Error, Arguments};

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
}

/*
impl<'a> Write for Logger<'a> {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        if let Some(writer) = self.device {
            writer.write_str(s)
        } else {
            Ok(())
        }
    }
}*/

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
            use debug;
            let mut ul = log::LOGGER.lock();
            ul.log(format_args!($fmt));
            //write!(*ul, "{}", format_args!($fmt));
        }
    };
}
