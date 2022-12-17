use super::{ControlState, Fan, Mode};
use bitfield::bitfield;
use thiserror::Error;

#[derive(Error, Clone, Copy, Debug)]
pub enum EncodeError {
    #[error("Temperature out of range. Must be between 17C and 30C")]
    TemperatureOutOfRange,

    #[error("Mode value wasn't recognized")]
    ModeOutOfRange(u8),

    #[error("Fan value wasn't recognized")]
    FanOutOfRange(u8),

    #[error("Unexpected fixed value in packet.")]
    UnexpectedFixedValues,
}

bitfield! {
    pub struct Packet(u64);
    impl Debug;
    pub u8, cmd_type, set_cmd_type : 47, 40;
    pub power, set_power : 39;
    pub sleep, set_sleep : 38;
    pub u8, fan_raw, set_fan_raw: 37, 35;
    pub u8, mode_raw, set_mode_raw: 34, 32;
    u8, unknown, set_unknown : 31, 29;
    pub u8, temperature_raw, set_temperature_raw: 28, 24;
    u16, ones, set_ones : 23, 8;
    pub u8, checksum, set_checksum : 7, 0;
}

impl Packet {
    const TEMP_NONE: u8 = 0b1110;

    // Modes
    const MODE_AUTO: u8 = 0b010;
    const MODE_COOL: u8 = 0b000;
    const MODE_DRY: u8 = 0b001;
    const MODE_HEAT: u8 = 0b011;
    const MODE_FAN: u8 = 0b100;

    // Fans
    const FAN_MIN: u8 = 0b001;
    const FAN_MEDIUM: u8 = 0b010;
    const FAN_MAX: u8 = 0b011;
    const FAN_AUTO: u8 = 0b100;

    const ONES: u16 = 0xFFFF;
    const UNKNOWN: u8 = 0b010;
    const CMD_TYPE: u8 = 0b10100001;

    pub fn new() -> Self {
        let mut p = Packet(0);
        p.set_cmd_type(Packet::CMD_TYPE);
        p.set_ones(Packet::ONES);
        p.set_unknown(Packet::UNKNOWN);
        p
    }

    // Returns the temperature in Celsius, or None if it is only in fan mode
    pub fn temperature(&self) -> Option<u8> {
        if self.temperature_raw() == Packet::TEMP_NONE {
            return None;
        } else {
            return Some(self.temperature_raw() + 17);
        }
    }

    pub fn set_temperature(&mut self, temp: Option<u8>) -> Result<(), EncodeError> {
        let temp = match temp {
            Some(temp) if temp < 17 || temp > 30 => return Err(EncodeError::TemperatureOutOfRange),
            Some(temp) => temp - 17,
            None => Packet::TEMP_NONE,
        };

        self.set_temperature_raw(temp);
        Ok(())
    }

    pub fn mode(&self) -> Result<Mode, EncodeError> {
        Ok(match self.mode_raw() {
            Packet::MODE_AUTO => Mode::Auto,
            Packet::MODE_COOL => Mode::Cool,
            Packet::MODE_DRY => Mode::Dry,
            Packet::MODE_HEAT => Mode::Heat,
            Packet::MODE_FAN => Mode::Fan,
            _ => return Err(EncodeError::ModeOutOfRange(self.mode_raw())),
        })
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.set_mode_raw(match mode {
            Mode::Auto => Packet::MODE_AUTO,
            Mode::Cool => Packet::MODE_COOL,
            Mode::Dry => Packet::MODE_DRY,
            Mode::Heat => Packet::MODE_HEAT,
            Mode::Fan => Packet::MODE_FAN,
        })
    }

    pub fn fan(&self) -> Result<Fan, EncodeError> {
        Ok(match self.fan_raw() {
            Packet::FAN_MIN => Fan::Min,
            Packet::FAN_MEDIUM => Fan::Medium,
            Packet::FAN_MAX => Fan::Max,
            Packet::FAN_AUTO => Fan::Auto,
            _ => return Err(EncodeError::FanOutOfRange(self.fan_raw())),
        })
    }

    pub fn set_fan(&mut self, fan: Fan) {
        self.set_fan_raw(match fan {
            Fan::Min => Packet::FAN_MIN,
            Fan::Medium => Packet::FAN_MEDIUM,
            Fan::Max => Packet::FAN_MAX,
            Fan::Auto => Packet::FAN_AUTO,
        })
    }
}

impl TryFrom<ControlState> for Packet {
    type Error = EncodeError;

    fn try_from(state: ControlState) -> Result<Self, EncodeError> {
        let mut packet = Packet::new();
        packet.set_temperature(state.temperature)?;
        packet.set_power(state.power);
        packet.set_mode(state.mode);
        packet.set_fan(state.fan);
        // TODO: Compute checksum
        packet.set_checksum(0);
        Ok(packet)
    }
}

impl TryFrom<Packet> for ControlState {
    type Error = EncodeError;

    fn try_from(packet: Packet) -> Result<Self, EncodeError> {
        if packet.cmd_type() != Packet::CMD_TYPE
            || packet.unknown() != Packet::UNKNOWN
            || packet.ones() != Packet::ONES
        {
            return Err(EncodeError::UnexpectedFixedValues);
        }

        Ok(ControlState {
            power: packet.power(),
            mode: packet.mode()?,
            fan: packet.fan()?,
            // silence: false,
            // timer: false,
            // led: false,
            // turbo: false,
            temperature: packet.temperature(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode() {
        // Power
        let off = 0xa12347ffffeb;
        let on = 0xa1a347ffff6b;

        assert_eq!(Packet(off).power(), false);
        assert_eq!(Packet(on).power(), true);
        assert_eq!(Packet(on).mode().unwrap(), Mode::Heat);
        assert_eq!(Packet(on).fan().unwrap(), Fan::Auto);

        // Temperatures
        assert_eq!(Packet(0xa1a348ffff65).temperature(), Some(25));
        assert_eq!(Packet(0xa1a349ffff64).temperature(), Some(26));
        assert_eq!(Packet(0xa1a34affff66).temperature(), Some(27));
        assert_eq!(Packet(0xa1a34bffff67).temperature(), Some(28));
        assert_eq!(Packet(0xa1a34cffff61).temperature(), Some(29));
        assert_eq!(Packet(0xa1a34dffff60).temperature(), Some(30));

        dbg!(Packet(0xa1a34dffff60));
        let state: ControlState = Packet(0xa1a34dffff60).try_into().unwrap();
        dbg!(state);
    }

    fn rev(input: u8) -> u8 {
        let mut output: u8 = 0;
        for i in 0..8 {
            let is_set = (input & (1 << i)) != 0;
            output |= (is_set as u8) << (7-i);
        }
        output
    }

    fn checksum(p: &Packet) -> u8 {
        let mut p = Packet(p.0);
        p.set_checksum(0);

        let mut sum: u64 = 0x00;
        for &v in p.0.to_be_bytes().iter() {
            sum = sum.wrapping_add(rev(v) as _);
        }
        rev((256 - sum % 256) as u8)
    }

    #[test]
    pub fn test_checksum() {
        let known_packets: &[u64; 7] = &[ 0xa12347ffffeb, 0xa1a347ffff6b, 0xa1a348ffff65, 0xa1a349ffff64, 0xa1a34affff66, 0xa1e34dffff20, 0xa1a34dffff60 ];
        let actual_checksums = known_packets.map(|p| Packet(p).checksum());
        let computed_checksums = known_packets.map(|p| checksum(&Packet(p)));

        dbg!(actual_checksums.iter().zip(computed_checksums).map(|(&c,a)| c.checked_sub(a).unwrap_or(a.checked_sub(c).unwrap())).collect::<Vec<_>>());
        assert_eq!(actual_checksums, computed_checksums);
    }

    #[test]
    pub fn test_rev() {
        let i = 0b1000_1000;
        let o = 0b0001_0001;
        assert_eq!(rev(i), o);
    }
}