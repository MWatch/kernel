
use crate::hal::datetime::{Date, Time};
use core::str::FromStr;
use crate::hal::prelude::*;
use crate::system::system::System;


#[derive(Debug, Copy, Clone)]
pub enum Error {
    ParseError,
    UnknownSyscall
}

#[derive(Debug, Copy, Clone)]
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
        for (idx, number) in s.split("/").enumerate() {
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
        for (idx, number) in s.split(":").enumerate() {
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