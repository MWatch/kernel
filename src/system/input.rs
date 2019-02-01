//! Input

use mwatch_kernel_api::InputEvent;

pub const LEFT: u8 = 1;
pub const MIDDLE: u8 = 2;
pub const RIGHT: u8 = 4;
pub const LEFT_MIDDLE: u8 = LEFT | MIDDLE;
pub const RIGHT_MIDDLE: u8 = RIGHT | MIDDLE;
pub const LEFT_RIGHT: u8 = LEFT | RIGHT;
pub const ALL: u8 = LEFT | MIDDLE | RIGHT;
pub const NONE: u8 = 0;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Error {
    NoInput,
    InvalidInputVector(u8)
}

pub struct InputManager {
    raw_vector: u8,
}

impl InputManager {

    pub fn new() -> Self {
        Self {
            raw_vector: 0,
        }
    }

    pub fn update_input(&mut self, input: u8, active: bool) {
        self.raw_vector |= match input {
            LEFT => (active as u8) << 0,
            MIDDLE => (active as u8) << 1,
            RIGHT => (active as u8) << 2,
            _ => {
                warn!("Ignoring vector input with value {:?}", input);
                NONE // do nothing
            },
        };
    }

    pub fn output(&mut self) -> Result<InputEvent, Error> {
        if self.raw_vector != NONE {
            let result = match self.raw_vector {
                ALL => Ok(InputEvent::Multi),
                LEFT_RIGHT => Ok(InputEvent::Dual),
                LEFT_MIDDLE => Ok(InputEvent::LeftMiddle),
                RIGHT_MIDDLE => Ok(InputEvent::RightMiddle),
                LEFT => Ok(InputEvent::Left),
                MIDDLE => Ok(InputEvent::Middle),
                RIGHT => Ok(InputEvent::Right),
                0 => Err(Error::NoInput), // no input
                _ => Err(Error::InvalidInputVector(self.raw_vector)),
            };
            self.raw_vector = 0;
            result
        } else {
            Err(Error::NoInput)
        }
    }

    //TODO: take the input tsc's in and initiate aquisictions though here
}