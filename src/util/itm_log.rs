//! ITM Logging

use log::{Level, LevelFilter, Metadata, Record};
use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use core::fmt;


pub struct ItmLogger {
    inner: Mutex<RefCell<cortex_m::peripheral::ITM>>
}

impl ItmLogger {

    pub fn new(itm: cortex_m::peripheral::ITM) -> Self {
        Self {
            inner: Mutex::new(RefCell::new(itm))
        }
    }
}

impl log::Log for ItmLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            
            cortex_m::interrupt::free(|cs| {
                // Obtain mutex protected itm
                // if let Some(mut itm) = self.inner.borrow(cs).try_borrow_mut().ok() {
                //     cortex_m::itm::write_fmt(&mut itm.stim[0], format_args!("\r\n{} - {}\r", record.level(), record.args()));
                // } else {
                //     // panic!("How this still be borrowing in cs?")
                // }
                {
                    let rf = self.inner.borrow(cs);
                    let mut itm = rf.borrow_mut();
                    cortex_m::itm::write_fmt(&mut itm.stim[0], format_args!("\r\n{} - {}\r", record.level(), record.args()));
                }
            });
        }
    }
    fn flush(&self) {}
}












// pub struct ItmLoggerUnsafe {
//     inner: UnsafeSync<cortex_m::peripheral::ITM>
// }

// impl ItmLoggerUnsafe {

//     pub fn new(itm: cortex_m::peripheral::ITM) -> Self {
//         Self {
//             inner: UnsafeSync::new(itm)
//         }
//     }
// }

// pub struct UnsafeSync<T> {
//     pub inner: T
// }

// impl<T> UnsafeSync<T> {
//     pub fn new(t: T) -> Self {
//         Self {
//             inner: t
//         }
//     }
// }

// unsafe impl<T> Sync for UnsafeSync<T> {}

// impl log::Log for ItmLoggerUnsafe {
//     fn enabled(&self, _metadata: &Metadata) -> bool {
//         true
//     }

//     fn log(&self, record: &Record) {
//         if self.enabled(record.metadata()) {
//             cortex_m::interrupt::free(|cs| {
//                 // // Obtain mutex protected itm
//                 // let mut itm = &mut self.inner.inner;
//                 // cortex_m::itm::write_fmt(&mut itm.stim[0], format_args!("\r\n{} - {}\r", record.level(), record.args()));
                
//             });
//         }
//     }
//     fn flush(&self) {}
// }