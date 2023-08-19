use std::time::Duration;

/**
 * Implements encoding/decoding of payloads sent to a broadlink IR device
 * Inspired from: https://github.com/haimkastner/broadlink-ir-converter/blob/master/src/index.ts
 * Payload format from: https://github.com/mjg59/python-broadlink/blob/master/protocol.md
 */
use bytes::{Buf, BufMut, Bytes, BytesMut};
use thiserror::Error;

trait BroadlinkDuration {
    fn to_broadlink(self) -> u16;
    fn from_broadlink(broadlink_pulse: u16) -> Self;
}

impl BroadlinkDuration for std::time::Duration {
    fn to_broadlink(self) -> u16 {
        // Round through float to avoid rounding errors in conversion
        (self.as_micros() as f64 * 269.0 / 8192.0).round() as u16
    }

    fn from_broadlink(broadlink_pulse: u16) -> Self {
        // Round through float to avoid rounding errors in conversion
        Self::from_nanos(
            ((broadlink_pulse as f64) * 8192000.0 / 269.0).round() as _
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Transport {
    Ir = 0x26,
    Rf433 = 0xb2,
    Rf315 = 0xd7,
}

/*
Offset	Contents
0x00	0x02
0x01-0x03	0x00
0x04	0x26 = IR, 0xb2 for RF 433Mhz, 0xd7 for RF 315Mhz
0x05	repeat count, (0 = no repeat, 1 send twice, .....)
0x06-0x07	Length of the following data in little endian
0x08 ....	Pulse lengths in 2^-15 s units (µs * 269 / 8192 works very well)
....	For IR codes, the pulse lengths should be paired as ON, OFF
 */
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recording {
    pub repeat_count: u8,
    pub transport: Transport,
    // On-off pulse durations
    pub pulses: Vec<Duration>,
}

#[derive(Error, Debug, Copy, Clone)]
pub enum ParseError {
    #[error("invalid transport type: {0}")]
    InvalidTransport(u8),
}

impl Recording {

    pub fn new_ir(pulses: Vec<Duration>) -> Self {
        Self {
            repeat_count: 0,
            transport: Transport::Ir,
            pulses,
        }
    }

    pub fn to_pulses(&self) -> Vec<u32> {
        self.pulses.iter().map(|p| p.as_micros() as _).collect()
    }

    pub fn to_raw_format(&self) -> String {
        use std::fmt::Write;

        let mut sign = false;
        let mut out = String::new();
        self.pulses.iter().for_each(|p| {
            sign = !sign;
            if sign {
                write!(out, "+").unwrap();
            } else {
                write!(out, "-").unwrap();
            }
            write!(out, "{} ", p.as_micros()).unwrap();
        });
        out
    }

    pub fn to_bytes(&self) -> Bytes {
        let mut b = BytesMut::new();
        b.put_u8(self.transport as u8);
        b.put_u8(self.repeat_count);

        let mut pulses_buf = BytesMut::new();
        for pulse in &self.pulses {
            let pulse = pulse.to_broadlink();
            if pulse < 256 {
                pulses_buf.put_u8(pulse as _);
            } else {
                pulses_buf.put_u8(0);
                pulses_buf.put_u16(pulse);
            }
        }

        b.put_u16_le(pulses_buf.len() as _);
        b.put(pulses_buf);
        b.freeze()
    }

    pub fn from_bytes(buf: Bytes) -> Result<Self, ParseError> {
        let mut buf = buf;

        let transport = match buf.get_u8() {
            0x26 => Transport::Ir,
            0xb2 => Transport::Rf433,
            0xd7 => Transport::Rf315,
            x => return Err(ParseError::InvalidTransport(x)),
        };

        let repeat_count = buf.get_u8();
        let pulse_count = buf.get_u16_le() as usize;

        let mut pulses = Vec::with_capacity(pulse_count);
        let mut remain = pulse_count;
        while remain > 0 {
            let mut value: u16 = buf.get_u8() as u16;
            remain -= 1;

            if value == 0 {
                // This indicates that the value didn't fit in a single byte and is stored as a u16_be
                if buf.len() < 2 {
                    break;
                }
                value = buf.get_u16();
                remain -= 2;
            }

            pulses.push(Duration::from_broadlink(value));
        }

        if pulses.len() % 2 != 0 {
            pulses.push(Duration::from_millis(100));
        }

        Ok(Recording {
            repeat_count,
            transport,
            pulses,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_decode() {
        let message = hex_literal::hex!("2600ca008b8f1035101211341013101210121112103510121112103510121112101211340f360f121134111210121112103510351035103510341134113411341134113410351035103510351035103510351035103510341134113411121035101210350f3510a88c8e11121035101211341134113410351012113411341112103510351035101210121134111210351035103510121013101210121112101210130f13101210131012101211121012101310121013101210121112101211121035101211341112101210000d05");
        let pulses = [
            4233, 4354, 487, 1614, 487, 548, 517, 1583, 487, 578, 487, 548, 487, 548, 517, 548,
            487, 1614, 487, 548, 517, 548, 487, 1614, 487, 548, 517, 548, 487, 548, 517, 1583, 456,
            1644, 456, 548, 517, 1583, 517, 548, 487, 548, 517, 548, 487, 1614, 487, 1614, 487,
            1614, 487, 1614, 487, 1583, 517, 1583, 517, 1583, 517, 1583, 517, 1583, 517, 1583, 487,
            1614, 487, 1614, 487, 1614, 487, 1614, 487, 1614, 487, 1614, 487, 1614, 487, 1614, 487,
            1614, 487, 1583, 517, 1583, 517, 1583, 517, 548, 487, 1614, 487, 548, 487, 1614, 456,
            1614, 487, 5116, 4263, 4324, 517, 548, 487, 1614, 487, 548, 517, 1583, 517, 1583, 517,
            1583, 487, 1614, 487, 548, 517, 1583, 517, 1583, 517, 548, 487, 1614, 487, 1614, 487,
            1614, 487, 548, 487, 548, 517, 1583, 517, 548, 487, 1614, 487, 1614, 487, 1614, 487,
            548, 487, 578, 487, 548, 487, 548, 517, 548, 487, 548, 487, 578, 456, 578, 487, 548,
            487, 578, 487, 548, 487, 548, 517, 548, 487, 548, 487, 578, 487, 548, 487, 578, 487,
            548, 487, 548, 517, 548, 487, 548, 517, 548, 487, 1614, 487, 548, 517, 1583, 517, 548,
            487, 548, 487, 35965,
        ];
        let message = Bytes::copy_from_slice(&message);

        let decoded = Recording::from_bytes(message.clone()).unwrap();
        assert_eq!(decoded.transport, Transport::Ir);
        assert_eq!(decoded.repeat_count, 0);

        for (i, (pulse, &ref_pulse)) in decoded.pulses.iter().zip(pulses.iter()).enumerate() {
            assert_eq!(
                pulse.as_micros() as u16,
                ref_pulse,
                "pulse {} does not match",
                i
            );
        }
        assert_eq!(decoded.pulses.len(), pulses.len());

        let encoded = decoded.to_bytes();
        assert_eq!(hex::encode(encoded), hex::encode(message));
    }
}
