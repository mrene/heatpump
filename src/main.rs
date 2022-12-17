use std::io::{self, Write};

use bytes::Bytes;
use clap::Parser;

use crate::{
    broadlink::Recording,
    lennox::{Phy, packet::Packet, ControlState},
};

mod broadlink;
mod lennox;
mod pwm;

#[derive(Clone, Parser, Debug)]
#[clap(version=env!("CARGO_PKG_VERSION"), author=env!("CARGO_PKG_AUTHORS"))]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}


#[derive(Clone, Parser, Debug)]
enum SubCommand {
    /// Decode hex-encoded commands in the broadlink format from stdin, and print them to stdout
    Decode,
    SetState(ControlState),
}

fn decode() -> anyhow::Result<()> {
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

fn set_state(state: ControlState) -> anyhow::Result<()> {
    // Encode ControlState into a broadlink-formatted message, and print it to stdout
    let packet: Packet = (&state).try_into()?;
    let phy = Phy::new();
    let encoded = phy.encode(packet.0)?;
    let recording = Recording {
        repeat_count: 0,
        transport: broadlink::Transport::Ir,
        pulses: encoded.into_iter().map(|x| broadlink::Pulse { duration: x }).collect(),
    };
    let recording_bytes = recording.to_bytes();
    println!("{}", hex::encode(recording_bytes));

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let opts: Opts = Opts::parse();
    match opts.subcmd {
        SubCommand::Decode => decode(),
        SubCommand::SetState(state) => set_state(state),
    }
}
