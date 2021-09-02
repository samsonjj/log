use std::io::Write;
use std::convert::TryFrom;


// The LOGGER static holds the global arc-Logger. There could be faster
// implementations of this concept, for for now this creates a solid, secure
// way of accessing the logger from multiple threads.
//
// For example, we could increase the speed by creating a second lock variable,
// and locking it only when we want to WRITE a new value to LOGGER. This would
// save access time on read operations of the LOGGER. (write/read referring to
// access of the variable, not to printing functions)
// static LOGGER: Logger = default_logger;

// should really just handle formatting (can only be done with variable number
// of arguments), then delegate to a real function
//
/// logs the given information if the log level is set to Info or lower.
macro_rules! info {
    ($($arg:tt)*) => ($crate::log(format!($(arg)*)))
}

use std::sync::{Arc, Mutex, Once};
use std::sync::atomic::{AtomicI8, Ordering};

// concurrent access implementation heavily influenced by
// <https://stackoverflow.com/questions/27791532/how-do-i-create-a-global-mutable-singleton>
//
// requirements:
// writable by single thread
// * how do we do this? Interior mutability? idk man
// not to mention writable on all threads
// * if this thing holds a list of outputs, that need to
//   be writing to, the whole thing must be mutable, right?
// readable by multiple threads
// * Arc
// accessible through crate API (rather than being passed in)
// * static

// static mut LOGGER: *const Arc<Mutex<Logger>> = 0 as *const Arc<Mutex<Logger>>;
// static INITIALIZED: AtomicI8 = AtomicI8::new(0);

static LOGGER: Arc<Mutex<Logger>> = Arc::new(Mutex::new(default_logger()));

fn set_logger(logger: Logger) {
    let mut data = LOGGER.lock().unwrap();
    *data = logger;
    // unsafe {
    //     let data = (*LOGGER).lock().unwrap();
    //     LOGGER = std::mem::transmute(Box::new(logger));
    // }
}

fn get_logger() -> Arc<Mutex<Logger>> {
    static ONCE: Once = Once::new();

    // if the logger hasn't been set yet, set it to the default logger
    // this will be more efficient (!?) than setting it up top?
    // ONCE.call_once(|| {
    //     match INITIALIZED.compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst) {
    //         Ok(_) => set_logger(default_logger()),
    //         _ => {}
    //     }
    // });
    
    Arc::clone(&LOGGER)

    // unsafe {
    //     (*LOGGER).clone()
    // }
}

// temp function for testing
// we should be able to access the logger data
pub fn use_logger(data: &str) {
    let r = get_logger();
    let mut logger = r.lock().unwrap();
    for mut output in &mut logger.outputs {
        output.write_all(data.as_bytes());
    }
}

/// controls the logging implementation. For now, we just call println!
fn log(data: String) {
    println!("{}", data);
}


// fn set_logger(logger: Logger) {
//     let mut data = LOGGER.lock().unwrap();
//     *data = logger;
//
//     // lock is dropped when we go out of scope
// }

fn default_logger() -> Logger {
    Logger {
        outputs: vec![Box::new(std::io::stdout())],
        level: LogLevel::Info
    }
}

#[derive(PartialOrd, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error
}

impl TryFrom<String> for LogLevel {
    type Error = ();

    fn try_from(val: String) -> Result<LogLevel, ()> {
        match &val.to_lowercase()[..] {
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            _ => Err(())
        }
    }
}

/// A multi-stream, verbose, leveled logger
pub struct Logger {
    pub outputs: Vec<Box<dyn Write + Sync + Send>>,
    pub level: LogLevel
}

impl Logger {
    pub fn new(&mut self) {
        self.outputs = vec![Box::new(std::io::stdout())];
        self.level = match std::env::var("LOG_LEVEL") {
            Ok(val) => LogLevel::try_from(val.to_owned()).unwrap_or(LogLevel::Info),
            _ => {
                println!("ERROR: error reading environment variable `LOG_LEVEL`.");
                println!("Setting logging level to `INFO`");
                LogLevel::Info
            }
        };
    }

    // pub fn info(&self, msg: String) {
    //     if self.level > LogLevel::Info { self.write(msg); }
    // }
    //
    // pub fn write(&self, msg: String) {
    //     self.outputs.iter().for_each(|mut o| {
    //         o.write_all(msg.as_bytes()).unwrap();
    //     });
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestWrite<'a> {
        data: &'a mut Vec<u8>
    }

    impl<'a> std::io::Write for TestWrite<'a> {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.data.write_all(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
    }

    #[test]
    fn my_test() {
        {
            let logger = Logger {
                outputs: vec![Box::new(TestWrite { data: &mut data })],
                level: LogLevel::Info
            };

            set_logger(logger);

            use_logger("hello!");

            assert_eq!(&data[..], "hello!".as_bytes());
        }
    }
}
