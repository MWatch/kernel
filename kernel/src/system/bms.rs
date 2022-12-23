//! Battery management
//! 
//! 

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Draining,
    Charging,
    Charged
}

pub trait BatteryManagement {
    fn state(&self) -> State;
    fn soc(&mut self) -> u16;
}