pub mod phy;
use clap::Parser;
pub use phy::*;
pub mod packet;

// The complete state sent to the heat pump
#[derive(Debug, Clone, Parser)]
pub struct ControlState {
    /// Power state
    #[clap(short, long)]
    pub power: bool,

    /// Operating mode
    #[clap(short, long)]
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
    #[clap(short, long)]
    pub temperature: Option<u8>,

    // Fan speed setting
    #[clap(short, long)]
    pub fan: Fan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumString)]
pub enum Fan {
    Min,
    Medium,
    Max,
    Auto,
    Zero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumString)]
pub enum Mode {
    Auto,
    Cool,
    Dry,
    Heat,
    Fan,
}
