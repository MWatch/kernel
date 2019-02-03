//! Input

use mwatch_kernel_api::InputEvent;
use crate::{LeftButton, MiddleButton, RightButton, TouchSenseController};
use crate::hal::tsc::Event as TscEvent;

pub const LEFT: u8 = 1;
pub const MIDDLE: u8 = 2;
pub const RIGHT: u8 = 4;
pub const LEFT_MIDDLE: u8 = LEFT | MIDDLE;
pub const RIGHT_MIDDLE: u8 = RIGHT | MIDDLE;
pub const LEFT_RIGHT: u8 = LEFT | RIGHT;
pub const ALL: u8 = LEFT | MIDDLE | RIGHT;
pub const NONE: u8 = 0;
const NUM_SAMPLES: u16 = 25;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Error {
    NoInput,
    InvalidInputVector(u8),
    InvalidInputPin,
    AcquisitionInProgress
}

pub struct InputManager
{
    raw_vector: u8,
    last_vector: u8,
    pin_idx: u8,
    tsc_threshold: u16,
    tsc: TouchSenseController,
    left: LeftButton,
    middle: MiddleButton,
    right: RightButton,
}

impl InputManager {

    pub fn new(tsc: TouchSenseController, left: LeftButton, middle: MiddleButton, right: RightButton) -> Self {
        // Acquire for rough estimate of capacitance
        let mut middle = middle;
        let mut tsc = tsc;

        let mut baseline = 0;
        for _ in 0..NUM_SAMPLES {
            baseline += tsc.acquire(&mut middle).unwrap();
        }
        let threshold = ((baseline / NUM_SAMPLES) / 100) * 90;

        tsc.listen(TscEvent::EndOfAcquisition);
        // tsc.listen(TscEvent::MaxCountError); // TODO

        Self {
            tsc: tsc,
            tsc_threshold: threshold,
            raw_vector: 0,
            last_vector: 0,
            pin_idx: 0,
            left: left,
            middle: middle,
            right: right,
        }
    }

    pub fn update_input(&mut self, active: bool) {
        self.raw_vector |= match self.pin_idx {
            0 => (active as u8) << 0,
            1 => (active as u8) << 1,
            2 => (active as u8) << 2,
            _ => panic!("Invalid pin index")
        };
        // update the index once the input has been set
        self.pin_idx += 1;
        if self.pin_idx > 2 {
            self.pin_idx = 0;
        }
    }

    pub fn output(&mut self) -> Result<InputEvent, Error> {
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
    }

    pub fn start_new(&mut self) -> Result<(), Error> {
        if self.tsc.in_progress() {
            return Err(Error::AcquisitionInProgress);
        }
        match self.pin_idx {
            0 => self.tsc.start(&mut self.left),
            1 => self.tsc.start(&mut self.middle),
            2 => self.tsc.start(&mut self.right),
            _ => panic!("Invalid pin index")
        }
        Ok(())
    }

    pub fn process_result(&mut self) {
        let value = match self.pin_idx {
            0 => self.tsc.read(&mut self.left).unwrap(),
            1 => self.tsc.read(&mut self.middle).unwrap(),
            2 => self.tsc.read(&mut self.right).unwrap(),
            _ => panic!("Invalid pin index")
        };
        self.update_input(value < self.tsc_threshold);
    }
}