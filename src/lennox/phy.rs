use crate::pwm::{Codec, CodecError, Rule};

use std::time::Duration;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum PulseType {
    Short,
    Long,
    FourThousand,
    FiveThousand,
    Huge,
}

#[derive(Error, Debug, Copy, Clone)]
pub enum PhyError {
    #[error("PWM error: {0}")]
    PWMError(#[from] CodecError<PulseType>),
    #[error("Decode error: {0}")]
    DecodeError(#[from] DecodeError),
}

const PREAMBLE: (PulseType, PulseType) = (PulseType::FourThousand, PulseType::FourThousand);

pub struct Phy {
    codec: Codec<PulseType>,
}
impl Phy {
    pub fn new() -> Self {
        let codec = Codec::new(
            [
                (PulseType::Short, Rule::new(Duration::from_micros(550))),
                (PulseType::Long, Rule::new(Duration::from_micros(1550))),
                (
                    PulseType::FourThousand,
                    Rule::new(Duration::from_micros(4000)),
                ),
                (
                    PulseType::FiveThousand,
                    Rule::new(Duration::from_micros(5150)),
                ),
                (PulseType::Huge, Rule::new(Duration::from_millis(100))),
            ]
            .into_iter(),
        );

        Self { codec }
    }

    pub fn encode(&self, bits: u64) -> Result<Vec<Duration>, PhyError> {
        let pulses = self.encode_pulses(bits);
        Ok(self.codec.encode(pulses.into_iter())?)
    }

    pub fn decode(&self, pulses: impl Iterator<Item = Duration>) -> Result<u64, PhyError> {
        let pulses = self.codec.decode(pulses)?;
        Ok(Phy::decode_bits(pulses.into_iter())?)
    }

    pub fn encode_pulses(&self, bits: u64) -> Vec<PulseType> {
        let mut pulses = Vec::with_capacity(2 * (48 * 2 + 2));

        Phy::append_bits(bits, false, &mut pulses);
        Phy::append_bits(bits ^ 0xFFFF_FFFF_FFFF, false, &mut pulses);

        pulses
    }

    /// Encode 48 bits into a sequence of pulses.
    fn append_bits(bits: u64, long_ending: bool, mut pulses: &mut Vec<PulseType>) {
        pulses.push(PREAMBLE.0);
        pulses.push(PREAMBLE.1);

        for bit in 0..48 {
            let val = bits & (1 << bit) != 0;
            match val {
                // 0
                true => {
                    pulses.push(PulseType::Short);
                    pulses.push(PulseType::Long);
                }

                // 1
                false => {
                    pulses.push(PulseType::Short);
                    pulses.push(PulseType::Short);
                }
            }
        }

        pulses.push(PulseType::Short);
        pulses.push(if long_ending {
            PulseType::Huge
        } else {
            PulseType::FiveThousand
        });
    }

    fn decode_bits(
        mut pulses: impl Iterator<Item = (PulseType, PulseType)>,
    ) -> Result<u64, DecodeError> {
        use PulseType::*;

        Phy::assert_preamble(&mut pulses)?;

        let mut ret: u64 = 0;
        for pulse in pulses {
            match pulse {
                (Short, Short) => {
                    // Append 0
                    ret <<= 1;
                }
                (Short, Long) => {
                    // Append 1
                    ret <<= 1;
                    ret |= 1;
                }
                (Short, FiveThousand | Huge) => break,
                any => return Err(DecodeError::InvalidCombination(any)),
            }
        }
        Ok(ret)
    }

    fn assert_preamble(
        mut pulses: impl Iterator<Item = (PulseType, PulseType)>,
    ) -> Result<(), DecodeError> {
        let next = pulses.next().ok_or(DecodeError::TruncatedMessage)?;
        if next != PREAMBLE {
            dbg!(next);
            return Err(DecodeError::InvalidPreamble);
        }

        Ok(())
    }

    pub fn decode_pulses(
        &self,
        mut pulses: impl Iterator<Item = (PulseType, PulseType)>,
    ) -> Result<u64, DecodeError> {
        let bits = Phy::decode_bits(&mut pulses)?;
        let repeated = Phy::decode_bits(&mut pulses)?;

        if bits ^ repeated != 0xFFFF_FFFF_FFFF {
            return Err(DecodeError::RepeatMismatch);
        }

        Ok(bits)
    }
}

#[derive(Error, Debug, Copy, Clone)]
pub enum DecodeError {
    #[error("invalid preamble")]
    InvalidPreamble,
    #[error("invalid combination of pulses: {0:?}")]
    InvalidCombination((PulseType, PulseType)),
    #[error("repeat mismatch")]
    RepeatMismatch,
    #[error("truncated message")]
    TruncatedMessage,
}

#[cfg(test)]
mod test {
    use crate::broadlink::{Pulse, Recording, Transport};

    use super::*;

    #[test]
    fn test_decode() {
        const MSG: u64 = 0xa12347ffffeb;
        let off = include_str!("../../captures/off.ir");
        let message = Recording::from_bytes(hex::decode(off).unwrap().into()).unwrap();

        let phy = Phy::new();
        let msg = phy
            .decode(message.pulses.iter().map(|x| x.duration))
            .unwrap();
        assert_eq!(msg, MSG);

        let encoded = phy.encode(MSG).unwrap();
        let recording = Recording {
            repeat_count: 0,
            transport: Transport::Ir,
            pulses: encoded.into_iter().map(|x| Pulse { duration: x }).collect(),
        };
        let recording_bytes = recording.to_bytes();
        assert_eq!(hex::encode(recording_bytes), off);
    }
}
