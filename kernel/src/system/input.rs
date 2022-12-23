//! Input
//!
//! Here we multiplex all the hardware inputs (3) to create a series of
//! unique output combinations (7)

pub const LEFT: u8 = 1;
pub const MIDDLE: u8 = 2;
pub const RIGHT: u8 = 4;
pub const LEFT_MIDDLE: u8 = LEFT | MIDDLE;
pub const RIGHT_MIDDLE: u8 = RIGHT | MIDDLE;
pub const LEFT_RIGHT: u8 = LEFT | RIGHT;
pub const ALL: u8 = LEFT | MIDDLE | RIGHT;
pub const NONE: u8 = 0;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputEvent {
    Left,
    Middle,
    Right,
    Dual,
    Multi,
    LeftMiddle,
    RightMiddle,
}

pub const MAX_PIN_IDX: u8 = 2;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Error {
    NoInput,
    InvalidInputVector(u8),
    InvalidInputPin,
    Incomplete,
}

/// Input manager, assumes control over the tsc peripheral and handles the raw inputs
pub struct InputManager {
    raw_vector: u8,
    last_vector: u8,
    count: usize,
}

impl InputManager {
    /// Creates a new instance of the InputManager
    pub fn new() -> Self {
        Self {
            raw_vector: 0,
            last_vector: 0,
            count: 0,
        }
    }

    /// Update thes the internal state of the manager with the raw hardware input
    pub fn update_input(&mut self, active: bool) {
        if active {
            self.raw_vector |= 1 << self.count;
        } else {
            self.raw_vector &= !(1 << self.count);
        }
        self.count += 1;
    }

    /// Based on the current state of the inputmanager's internal vector, produce an output
    pub fn output(&mut self) -> Result<InputEvent, Error> {
        if self.count > MAX_PIN_IDX as usize {
            self.count = 0;
            if self.raw_vector != self.last_vector {
                let result = match self.raw_vector {
                    ALL => Ok(InputEvent::Multi),
                    LEFT_RIGHT => Ok(InputEvent::Dual),
                    LEFT_MIDDLE => Ok(InputEvent::LeftMiddle),
                    RIGHT_MIDDLE => Ok(InputEvent::RightMiddle),
                    LEFT => Ok(InputEvent::Left),
                    MIDDLE => Ok(InputEvent::Middle),
                    RIGHT => Ok(InputEvent::Right),
                    NONE => Err(Error::NoInput), // no input
                    _ => Err(Error::InvalidInputVector(self.raw_vector)),
                };
                self.last_vector = self.raw_vector;
                result
            } else {
                Err(Error::NoInput)
            }
        } else {
            Err(Error::Incomplete)
        }
    }

    pub fn current_pin(&self) -> usize {
        self.count
    }
}
