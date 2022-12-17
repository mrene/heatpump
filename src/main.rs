use std::io::{self, Write};

use bytes::Bytes;

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
        io::stdout().flush()?;
    }

    Ok(())
}

