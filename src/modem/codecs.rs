use bytes::Bytes;
use clap::Parser;
use thiserror::Error;

use crate::broadlink::Recording;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Parser, strum::EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum CodecType {
    Base64,
    Hex,
    Raw,
}

pub trait Codec {
    type Error;

    fn decode(&self, input: &str) -> Result<Recording, Self::Error>;
    fn encode(&self, recording: &Recording) -> Result<String, Self::Error>;
}

pub fn create_codec(ty: CodecType) -> Box<dyn Codec<Error=CodecError>> {
    match ty {
        CodecType::Base64 => Box::new(BroadlinkBase64),
        CodecType::Hex => Box::new(BroadlinkHex),
        CodecType::Raw => Box::new(Raw),
    }
}

pub struct BroadlinkHex;

#[derive(Error, Debug)]
pub enum CodecError {
    #[error("failed to decode hex string: {0}")]
    HexDecodeError(#[from] hex::FromHexError),
    #[error("failed to decode base64 string: {0}")]
    Base64DecodeError(#[from] base64::DecodeError),
    #[error("failed to parse broadlink message: {0}")]
    BroadlinkParseError(#[from] crate::broadlink::ParseError),
    #[error("failed to decode raw string")]
    RawParseError,
    #[error("empty input")]
    EmptyInput,
}

impl Codec for BroadlinkHex {
    type Error = CodecError;

    fn decode(&self, input: &str) -> Result<Recording, Self::Error> {
        let mut decoded = hex::decode(input)?;
        if decoded.len() == 0 {
            return Err(CodecError::EmptyInput);
        } else if decoded.len() % 2 != 0 {
            decoded.push(0);
        }
        
        Ok(Recording::from_bytes(Bytes::copy_from_slice(&decoded))?)
    }

    fn encode(&self, recording: &Recording) -> Result<String, Self::Error> {
        let encoded = recording.to_bytes();
        Ok(hex::encode(&encoded))
    }
}
pub struct BroadlinkBase64;
impl Codec for BroadlinkBase64 {
    type Error = CodecError;

    fn decode(&self, input: &str) -> Result<Recording, Self::Error> {
        let decoded = base64::decode(input)?;
        Ok(Recording::from_bytes(Bytes::copy_from_slice(&decoded))?)
    }

    fn encode(&self, recording: &Recording) -> Result<String, Self::Error> {
        let encoded = recording.to_bytes();
        Ok(base64::encode(&encoded))
    }
}

pub struct Raw;
impl Codec for Raw {
    type Error = CodecError;

    fn decode(&self, input: &str) -> Result<Recording, Self::Error> {

        // Support IrTransmogrifier's format which looks like `Freq=38400Hz[.....][...]`
        let input = if input.starts_with("Freq=") {
            let mut parts = input.splitn(2, '[');
            parts.next();
            let untrimmed = parts.next().ok_or(CodecError::RawParseError)?;
            untrimmed.split(']').next().ok_or(CodecError::RawParseError)?
        } else {
            input
        };


        let msg = irp::Message::parse(input).or(Err(CodecError::RawParseError))?;
        Ok(Recording {
            repeat_count: 0,
            transport: crate::broadlink::Transport::Ir,
            pulses: msg.raw.into_iter().map(|t| std::time::Duration::from_micros(t as _)).collect(),
        })
    }

    fn encode(&self, recording: &Recording) -> Result<String, Self::Error> {
        Ok(recording.to_raw_format())
    }
}


