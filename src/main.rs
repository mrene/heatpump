use std::io::{self, Write};

use bytes::Bytes;

use crate::lennox::{packet::Packet, ControlState};

mod broadlink;
mod lennox;
mod pwm;

fn main() -> anyhow::Result<()> {

    // Read hex-encoded messages from stdin, convert them and print their decoded u64 hex value

    let phy = lennox::Phy::new();

    let stdin = std::io::stdin();
    for line in stdin.lines() {
        let bytes = Bytes::copy_from_slice(&hex::decode(line?)?);
        let recording = broadlink::Recording::from_bytes(bytes)?;

        let msg = phy.decode(recording.pulses.iter().map(|x| x.duration))?;
        println!("{:x} {:b}", msg, msg);

        let packet = Packet(msg);
        let state = TryInto::<ControlState>::try_into(&packet);
        println!("{:?}", state);
        io::stdout().flush()?;
    }

    Ok(())
}

