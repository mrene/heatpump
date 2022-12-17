use std::io::{self, Write};

use bytes::Bytes;

use crate::{
    broadlink::Recording,
    lennox::{Phy, packet::Packet, ControlState},
};

mod broadlink;
mod lennox;
mod pwm;

fn main() -> anyhow::Result<()> {
    // Read hex-encoded messages from stdin, convert them and print their decoded u64 hex value

    let phy = Phy::new();

    let stdin = std::io::stdin();
    for line in stdin.lines() {
        let recording = Recording::from_bytes(Bytes::copy_from_slice(&hex::decode(line?)?))?;
        let msg = phy.decode(recording.pulses.iter().map(|x| x.duration))?;
        println!("{:x} {:b}", msg, msg);

        let state: Result<ControlState, _> = (&Packet(msg)).try_into();
        println!("{:?}", state);

        io::stdout().flush()?;
    }

    Ok(())
}
