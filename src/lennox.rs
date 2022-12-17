use crate::broadlink::{Pulse};
use bytes::{Bytes, BytesMut};
use std::time::Duration;
use thiserror::Error;

const TIME_SHORT: Duration = Duration::from_micros(550);
const TIME_LONG: Duration = Duration::from_micros(1550);
const TIME_4000: Duration = Duration::from_micros(4000);
const TIME_5000: Duration = Duration::from_micros(5150);

// This one seem to be artifacts of the recording process
const TIME_HUGE: Duration = Duration::from_millis(100);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PulseType {
    Short,
    Long,
    FourThousand,
    FiveThousand,
    Huge,
}

pub struct PWMDecoder;

#[derive(Error, Debug, Copy, Clone)]
pub enum PWMError {
    #[error("the pulse list was missing an off pulse value")]
    MissingOffPulse,
    #[error("invalid pulse length: {0:?}")]
    InvalidPulseLength(Duration),
}

impl PWMDecoder {
    pub fn decode<'a>(
        &self,
        mut raw_pulses: impl Iterator<Item = &'a Pulse>,
    ) -> Result<Vec<(PulseType, PulseType)>, PWMError> {
        let mut pulses = Vec::new();

        // Pulses are encoded with time on and time off, zip them so we can process them together
        while let Some(on) = raw_pulses.next() {
            let off = raw_pulses.next().ok_or(PWMError::MissingOffPulse)?;
            // println!(
            //     "on: {} off: {}",
            //     on.duration.as_micros(),
            //     off.duration.as_micros()
            // );
            let on = PWMDecoder::match_pulse(on)?;
            let off = PWMDecoder::match_pulse(off)?;
            // println!("on: {:?} off: {:?}", on, off);
            pulses.push((on, off));
        }

        Ok(pulses)
    }

    fn match_pulse(pulse: &Pulse) -> Result<PulseType, PWMError> {
        if pulse
            .duration
            .within_bounds(&TIME_SHORT, &Duration::from_micros(200))
        {
            Ok(PulseType::Short)
        } else if pulse
            .duration
            .within_bounds(&TIME_LONG, &Duration::from_micros(500))
        {
            Ok(PulseType::Long)
        } else if pulse
            .duration
            .within_bounds(&TIME_4000, &Duration::from_micros(500))
        {
            Ok(PulseType::FourThousand)
        } else if pulse
            .duration
            .within_bounds(&TIME_5000, &Duration::from_micros(500))
        {
            Ok(PulseType::FiveThousand)
        } else if pulse
            .duration
            .within_bounds(&TIME_HUGE, &Duration::from_micros(2000))
        {
            Ok(PulseType::Huge)
        } else {
            Err(PWMError::InvalidPulseLength(pulse.duration))
        }
    }
}

pub trait TimingWithinBounds {
    fn within_bounds(&self, other: &Self, tolerance: &Self) -> bool;
}

impl TimingWithinBounds for Duration {
    fn within_bounds(&self, other: &Self, tolerance: &Self) -> bool {
        let diff = self
            .checked_sub(*other)
            .unwrap_or_else(|| other.checked_sub(*self).unwrap());
        diff <= *tolerance
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

fn decode_bits(
    mut pulses: impl Iterator<Item = (PulseType, PulseType)>,
) -> Result<u64, DecodeError> {
    use PulseType::*;

    let mut ret: u64 = 0;
    println!("DEC: ");
    while let Some(pulse) = pulses.next() {
        match pulse {
            (Short, Short) => {
                ret <<= 1;
                print!("0");
            }
            (Short, Long) => {
                ret <<= 1;
                ret |= 1;
                print!("1");
            }
            (Short, FiveThousand | Huge) => break,
            comb => return Err(DecodeError::InvalidCombination(comb)),
        }
    }
    println!("");
    Ok(ret)
}

fn assert_preamble(
    mut pulses: impl Iterator<Item = (PulseType, PulseType)>,
) -> Result<(), DecodeError> {
    use PulseType::*;

    if pulses.next().ok_or(DecodeError::TruncatedMessage)? != (FourThousand, FourThousand) {
        return Err(DecodeError::InvalidPreamble);
    }

    Ok(())
}

pub fn decode_message(
    mut pulses: impl Iterator<Item = (PulseType, PulseType)>,
) -> Result<u64, DecodeError> {

    let bits = {
        assert_preamble(&mut pulses)?;
        decode_bits(&mut pulses)?
    };
    
    let repeated = {
        assert_preamble(&mut pulses)?;
        decode_bits(&mut pulses)?
    };

    if bits ^ repeated != 0xFFFF_FFFF_FFFF {
        return Err(DecodeError::RepeatMismatch);
    }

    Ok(bits)
}
