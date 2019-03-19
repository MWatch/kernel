//! Syscall
//! 
//! All possible system calls via the serial interface will be parsed and executed here


use crate::types::hal::datetime::{Date, Time};
use core::str::FromStr;
use crate::types::hal::prelude::*;
use crate::system::system::System;


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Error {
    ParseError,
    UnknownSyscall
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Syscall {
    /// Set the date - example: 
    /// "D0/12/02/2019"
    ///  day in week, date, month, year                         
    Date(Date),
    /// Set the time - example:
    /// "T12:21:11"
    /// hours, minutes, seconds
    Time(Time),
}

impl FromStr for Syscall {
    type Err = Error;

    /// Converts a string to a syscall
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // first charater is the type
        let t = s.as_bytes()[0];
        let s: &str = &s[1..]; // remove first byte after we have the type
        match t {
            b'D' => Ok(Syscall::Date(Syscall::date_from_str(s)?)),
            b'T' => Ok(Syscall::Time(Syscall::time_from_str(s)?)),
            _ => Err(Error::UnknownSyscall)
        }
    }
}

impl Syscall {

    pub fn execute(self, system: &mut System) {
        match self {
            Syscall::Date(date) => {
                info!("Setting the date to {:?}", date);
                system.rtc().set_date(&date);
            },
            Syscall::Time(time) => {
                info!("Setting the time to {:?}", time);
                system.rtc().set_time(&time);
            },
        }
    }

    pub fn date_from_str(s: &str) -> Result<Date, Error> {
        let mut vals = [0u32; 4];
        for (idx, number) in s.split('/').enumerate() {
            match number.parse() {
                Ok(val) => vals[idx] = val,
                Err(e) => {
                    error!("Failed to convert {} into a integer due to {:?}", number, e);
                    return Err(Error::ParseError)
                }
            }
        }
        Ok(Date::new(vals[0].day(), vals[1].date(), vals[2].month(), vals[3].year()))
    }

    pub fn time_from_str(s: &str) -> Result<Time, Error> {
        let mut vals = [0u32; 3];
        for (idx, number) in s.split(':').enumerate() {
            match number.parse() {
                Ok(val) => vals[idx] = val,
                Err(e) => {
                    error!("Failed to convert {} into a integer due to {:?}", number, e);
                    return Err(Error::ParseError)
                }
            }
        }
        Ok(Time::new(vals[0].hours(), vals[1].minutes(), vals[2].seconds(), false))
    }
}


#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn syscall_date_works() {
        let actual = Date::new(1.day(), 1.date(), 4.month(), 2019.year());

        let working = Syscall::from_str("D01/01/04/2019").unwrap();
        match working {
            Syscall::Date(d) => {
                assert_eq!(actual, d);
            }
            _ => panic!("wrong syscall type")
        }

        let wrong = Syscall::from_str("D02/01/04/2019").unwrap();
        match wrong {
            Syscall::Date(d) => {
                assert_ne!(actual, d);
            }
            _ => panic!("wrong syscall type")
        }
    }

    #[test]
    fn syscall_time_works() {
        let actual = Time::new(0.hours(), 0.minutes(), 0.seconds(), false);

        let working = Syscall::from_str("T00:00:00").unwrap();
        match working {
            Syscall::Time(t) => {
                assert_eq!(actual, t);
            }
            _ => panic!("wrong syscall type")
        }

        let working = Syscall::from_str("T01:00:00").unwrap();
        match working {
            Syscall::Time(t) => {
                assert_ne!(actual, t);
            }
            _ => panic!("wrong syscall type")
        }
    }
}