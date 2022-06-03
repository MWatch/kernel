

pub struct TscManager {
    tsc: TouchSenseController,
    left: LeftButton,
    middle: MiddleButton,
    right: RightButton,
    tsc_threshold: u16,
}

impl TscManager{

    pub fn new(tsc: TouchSenseController, threshold: u16, left: LeftButton, middle: MiddleButton, right: RightButton) -> Self {
        let mut tsc = tsc;
        tsc.listen(TscEvent::EndOfAcquisition);
        // tsc.listen(TscEvent::MaxCountError); // TODO

        Self {
            tsc,
            tsc_threshold: threshold,
            left,
            middle,
            right,
        }
    }

    /// Begin a new hardware (tsc) acquisition
    pub fn start(&mut self, pin: u8) -> Result<(), Error> {
        if self.tsc.in_progress() {
            return Err(Error::AcquisitionInProgress);
        }
        match pin {
            0 => self.tsc.start(&mut self.left),
            1 => self.tsc.start(&mut self.middle),
            2 => self.tsc.start(&mut self.right),
            _ => panic!("Invalid pin index")
        }
        Ok(())
    }

    /// Call when the aquisition is complete, this function read
    /// the registers and update the interal state
    pub fn result(&mut self, pin: u8) -> bool {
        let value = match pin {
            0 => self.tsc.read(&mut self.left).expect("Expected TSC pin 0"),
            1 => self.tsc.read(&mut self.middle).expect("Expected TSC pin 1"),
            2 => self.tsc.read(&mut self.right).expect("Expected TSC pin 2"),
            _ => panic!("Invalid pin index")
        };
        trace!("tsc[{}] {} < {}?", pin, value, self.tsc_threshold);
        //self.update_input(value < self.tsc_threshold);
        self.tsc.clear(TscEvent::EndOfAcquisition);

        value < self.tsc_threshold
    }

    /// returns the threshold value required to identify a touch
    pub fn threshold(&self) -> u16 {
        self.tsc_threshold
    }
}