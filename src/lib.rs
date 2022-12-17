use std::io::{self, Write};

use bytes::Bytes;

use crate::{
    broadlink::Recording,
    lennox::{packet::Packet, ControlState},
};

mod broadlink;
mod lennox;
mod pwm;
