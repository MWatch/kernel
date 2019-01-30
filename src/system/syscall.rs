//! Syscall - Internal commands for the operation of the kernel

#[derive(Copy, Clone, Debug)]
pub enum Syscall {
    /// Enable or disable bluetooth
    Bluetooth(bool),
    /// Run or stop the current application
    ApplicationRun(bool),
    /// Pause / Resume the current application
    ApplicationPause(bool), 
}