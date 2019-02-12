
use crate::hal::datetime::{Date, Time};
use core::str::FromStr;


#[derive(Debug, Copy, Clone)]
pub enum Error {
    ParseError,
    UnknownSyscall
}

#[derive(Debug, Copy, Clone)]
pub enum Syscall {
    /// Set the date
    Date(Date),
    /// Set the time
    Time(Time),
    /// Turn on or off the bluetooth
    Bluetooth(bool),
}

impl FromStr for Syscall {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // first charater is the type
        let t = s.as_bytes()[0];
        let s = &s[1..]; // remove first byte after we have the type
        match t {
            b'D' => Ok(Syscall::Date(Syscall::date_from_str(s)?)),
            b'T' => Ok(Syscall::Date(Syscall::time_from_str(s)?)),
            b'B' => Ok(Syscall::Date(Syscall::bluetooth_from_str(s)?)),
            _ => Err(Error::UnknownSyscall)
        }
    }
}

impl Syscall {

    pub fn execute(self /* probs need system or something passed into it */) -> Result<(), Error> {
        match self {
            Syscall::Date(_date) => {},
            Syscall::Time(_time) => {},
            Syscall::Bluetooth(_val) => {},
        }
        Ok(())
    }

    pub fn date_from_str(s: &str) -> Result<Date, Error> {
        unimplemented!();
    }

    pub fn time_from_str(s: &str) -> Result<Date, Error> {
        unimplemented!();
    }
    pub fn bluetooth_from_str(s: &str) -> Result<Date, Error> {
        unimplemented!();
    }
}