use embedded_hal::{digital::v2::*, blocking::i2c::*};
use mwatch_kernel::system::bms::State;


pub struct BatteryManagement<I, C, S> {
    bms: max17048::Max17048<I>,
    csp: C,
    ssp: S,
    state: State,
}

impl<E, E2, I, C, S> BatteryManagement<I, C, S> 
where C: InputPin<Error = E>,
      S: InputPin<Error = E>,
      I: WriteRead<Error = E2> + Write<Error = E2>,
      E: core::fmt::Debug,
      E2: core::fmt::Debug
{

    /// Creates a new instance of BatteryManagement singleton
    pub fn new(bms: max17048::Max17048<I>, csp: C, ssp: S) -> Self {
        Self {
            bms,
            csp,
            ssp,
            state: State::Draining,
        }
    }

    /// Returns the current state of battery
    pub fn state(&self) -> State {
        self.state
    }

    /// Returns the current state of charge (%) of the battery
    pub fn soc(&mut self) -> u16 {
        //TODO should we cache this value and instead only update when we process?
        bodged_soc(self.bms.soc().unwrap_or_else(|err| {
            error!("Failed to read soc from bms: {:?}", err);
            100 // return 100 percent
        }))
    }

    /// internal processing of the bms
    pub fn process(&mut self) {
        if self.csp.is_low().unwrap() {
            self.state = State::Charging;
        } else if self.ssp.is_high().unwrap() {
            self.state = State::Draining;
        } else {
            self.state = State::Charged;
        }
    }
}

/// Maxim does not have the charge algorithm parameters
/// publically available, hence we have to bodge the values
/// for our specific battery size
fn bodged_soc(raw: u16) -> u16 {
    let rawf = f32::from(raw);
    let max = 94.0; // based on current battery
    let mut soc = ((rawf / max) * 100.0) as u16;
    if soc > 100 {
        soc = 100; // cap at 100
    }
    soc
}