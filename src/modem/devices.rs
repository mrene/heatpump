use std::{
    io::{BufRead, BufReader},
    net::Ipv4Addr,
    ops::DivAssign,
    str::FromStr,
};

use anyhow::anyhow;
use bytes::Bytes;
use clap::Parser;
use thiserror::Error;

use crate::broadlink::{self, Recording};

use super::codecs::{create_codec, Codec, CodecError, CodecType};

pub trait Device {
    type Error;

    fn send(&mut self, recording: &Recording) -> Result<(), Self::Error>;
    fn recv(&mut self) -> Result<Recording, Self::Error>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceType {
    /// Use a broadlink remote device
    Broadlink { addr: Ipv4Addr },

    /// Read/write lines to stdin/stdout
    Lines {
        codec_type: CodecType,
        // reader: Box<dyn std::io::Read>,
        // writer: Box<dyn std::io::Write>,
    },
}

impl FromStr for DeviceType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(':');
        let device_type = parts.next().unwrap();

        Ok(match device_type {
            "broadlink" => {
                let addr = parts
                    .next()
                    .ok_or_else(|| anyhow!("Missing device address"))?;
                DeviceType::Broadlink {
                    addr: Ipv4Addr::from_str(addr)?,
                }
            }
            "lines" => {
                let codec_type = parts.next().ok_or_else(|| anyhow!("Missing codec type"))?;
                DeviceType::Lines {
                    codec_type: CodecType::from_str(codec_type)?,
                }
            }
            _ => return Err(anyhow!("unknown device type: {}", device_type)),
        })

    }
}

pub fn create_device(ty: DeviceType) -> Box<dyn Device<Error = DeviceError>> {
    match ty {
        DeviceType::Broadlink { addr } => {
            use rbroadlink::Device;
            let device = Device::from_ip(addr, None).unwrap();
            Box::new(device)
        }
        DeviceType::Lines {
            codec_type,
            // reader,,
            // writer,
        } => Box::new(Lines::new(
            codec_type,
            Box::new(std::io::stdin()),
            Box::new(std::io::stdout()),
        )),
    }
}

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("this device wasn't a remote")]
    NotARemote,

    #[error("failed to parse broadlink message: {0}")]
    BroadlinkParseError(#[from] crate::broadlink::ParseError),

    #[error("codec error: {0}")]
    CodecError(#[from] CodecError),

    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("EOF")]
    EOF,
}

impl Device for rbroadlink::Device {
    type Error = DeviceError;

    fn send(&mut self, recording: &Recording) -> Result<(), Self::Error> {
        let device = match self {
            rbroadlink::Device::Remote { remote } => remote,
            _ => return Err(DeviceError::NotARemote),
        };

        // rbroadlink doesn't actually return errors and calls `.expect()` underneath, so the process already crashes if this fails
        device.send_code(recording.to_bytes().as_ref()).unwrap();

        Ok(())
    }

    fn recv(&mut self) -> Result<Recording, Self::Error> {
        let device = match self {
            rbroadlink::Device::Remote { remote } => remote,
            _ => return Err(DeviceError::NotARemote),
        };

        // rbroadlink doesn't actually return errors and calls `.expect()` underneath, so the process already crashes if this fails
        let msg = device.learn_ir().unwrap();
        Ok(broadlink::Recording::from_bytes(Bytes::from(msg))?)
    }
}

pub struct Lines {
    codec: Box<dyn Codec<Error = CodecError>>,
    reader: BufReader<Box<dyn std::io::Read>>,
    writer: Box<dyn std::io::Write>,
}
impl Lines {
    pub fn new(
        codec_type: CodecType,
        reader: Box<dyn std::io::Read>,
        writer: Box<dyn std::io::Write>,
    ) -> Self {
        Self {
            codec: create_codec(codec_type),
            reader: BufReader::new(reader),
            writer,
        }
    }
}
impl Device for Lines {
    type Error = DeviceError;

    fn send(&mut self, recording: &Recording) -> Result<(), Self::Error> {
        let encoded = self.codec.encode(recording)?;
        writeln!(self.writer, "{}", encoded).unwrap();
        Ok(())
    }

    fn recv(&mut self) -> Result<Recording, Self::Error> {
        let mut input = String::new();
        Ok(match self.reader.read_line(&mut input) {
            Ok(n) if n == 0 => return Err(DeviceError::EOF),
            Ok(_) => self.codec.decode(input.trim_end())?,
            Err(e) => return Err(DeviceError::IOError(e)),
        })
    }
}
