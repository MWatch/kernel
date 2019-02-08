//! Battery management
//! 
//! 

use crate::{BatteryManagementIC, ChargeStatusPin, StandbyStatusPin};
use stm32l4xx_hal::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum State {
    Draining,
    Charging,
    Charged
}

pub struct BatteryManagement {
    bms: BatteryManagementIC,
    csp: ChargeStatusPin,
    ssp: StandbyStatusPin,
    state: State,
}

impl BatteryManagement {

    pub fn new(bms: BatteryManagementIC, csp: ChargeStatusPin, ssp: StandbyStatusPin) -> Self {
        Self {
            bms: bms,
            csp: csp,
            ssp: ssp,
            state: State::Draining,
        }
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn soc(&mut self) -> u16 {
        bodged_soc(self.bms.soc().unwrap()) // should we cache this value and instead only update when we process?
    }

    pub fn process(&mut self) {
        if self.csp.is_low() {
            self.state = State::Charging;
        } else if self.ssp.is_high() {
            self.state = State::Draining;
        } else {
            self.state = State::Charged;
        }
    }
}


fn bodged_soc(raw: u16) -> u16 {
    let rawf = raw as f32;
    let max = 94.0; // based on current battery
    let mut soc = ((rawf / max) * 100.0) as u16;
    if soc > 100 {
        soc = 100; // cap at 100
    }
    soc
}