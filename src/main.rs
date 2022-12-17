use std::io::{self, Write};

use bytes::Bytes;
use clap::Parser;

use crate::{
    broadlink::Recording,
    lennox::{packet::Packet, ControlState, Phy},
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
    /// Decode hex-encoded commands in the broadlink format from stdin, and print them to stdout
    Decode,

    /// Encodes a state message from the given arguments, outputs it to stdout in broadlink hex format
    SetState(ControlState),
}

/// Read hex-encoded messages from stdin, convert them and print their decoded u64 hex value
fn decode() -> anyhow::Result<()> {
    let phy = Phy::new();

    let stdin = std::io::stdin();
    for line in stdin.lines() {
        let recording = Recording::from_bytes(Bytes::copy_from_slice(&hex::decode(line?)?))?;
        let msg = phy.decode(recording.pulses.iter().copied())?;
        println!("Recv: {:x} {:b}", msg, msg);

        let state = Packet(msg).to_control_state();
        println!("Decode: {:?}", state);

        io::stdout().flush()?;
    }

    Ok(())
}

/// Encode ControlState into a broadlink-formatted message, and print it to stdout
fn set_state(state: ControlState) -> anyhow::Result<()> {
    let packet: Packet = Packet::from_control_state(&state)?;
    let pulses = Phy::new().encode(packet.0)?;
    let recording_bytes = Recording::new_ir(pulses).to_bytes();

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
