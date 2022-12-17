pub mod phy;
pub use phy::*;
pub mod packet;

// The complete state sent to the heat pump
#[derive(Debug, Clone)]
pub struct ControlState {
    // Power state
    pub power: bool,

    pub mode: Mode,

    // Silence FP
    // pub silence: bool,

    // Timer On
    // pub timer: bool,

    // Status leds on the front panel
    // pub led: bool,

    // Turbo mode, temporarily boosts cooling/heating for ~10mins
    // pub turbo: bool,

    // Current set temperature in Celsius, or None if it is only in fan mode
    pub temperature: Option<u8>,

    // Fan speed setting
    pub fan: Fan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Fan {
    Min,
    Medium,
    Max,
    Auto
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    Auto,
    Cool,
    Dry,
    Heat,
    Fan
}