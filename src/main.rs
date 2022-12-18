use std::io::{self, Write};

use bytes::Bytes;
use clap::Parser;
use irp::InfraredData;

use crate::{
    broadlink::Recording,
    lennox::{packet::Packet, ControlState, Phy},
};

mod broadlink;
mod lennox;
mod pwm;
mod smartir;

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

    /// Decodes a broadlink message into a series of pulse length (in microseconds)
    Broadlink,

    /// IRP decode
    Irp,

    /// Generate a SmartIR code file from all possible states
    SmartIR,
}

/// Read hex-encoded messages from stdin, convert them and print their decoded u64 hex value
fn decode() -> anyhow::Result<()> {
    let phy = Phy::new();

    let stdin = std::io::stdin();
    for line in stdin.lines() {
        let recording = Recording::from_bytes(Bytes::from(base64::decode(line?)?))?;
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

fn broadlink_decode() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    for line in stdin.lines() {
        let line = base64::decode(line?)?;
        //hex::decode(line?)?;
        let recording = Recording::from_bytes(Bytes::copy_from_slice(&line))?;
        // println!("{:?}", recording.pulses.iter().map(|p| p.as_micros()).collect::<Vec<_>>());

        let mut sign = false;
        recording.pulses.into_iter().for_each(|p| {
            sign = !sign;
            if sign {
                print!("+");
            } else {
                print!("-");
            }
            print!("{} ", p.as_micros());
        });
    }

    Ok(())
}

fn irp_decode() -> anyhow::Result<()> {
    use irp::Irp;

    const IRP_48_NEC1: &'static str = "{38.4k,564}<1,-1|1,-3>(16,-8,D:8,S:8,F:8,~F:8,E:8,~E:8,1,^108m,(16,-4,1,^108m)*)[D:0..255,S:0..255=255-D,F:0..255,E:0..255]";
    let nfa = Irp::parse(IRP_48_NEC1)
        .expect("irp parse")
        .compile()
        .expect("irp compile");

    let stdin = std::io::stdin();
    for line in stdin.lines() {
        let line = line?;
        let pulses = {
            let recording = Recording::from_bytes(Bytes::copy_from_slice(&hex::decode(line)?))?;
            InfraredData::from_u32_slice(&recording.to_pulses())
        };

        let res = {
            let mut decoder = nfa.decoder(100, 30, 20000);
            for pulse in pulses {
                decoder.input(pulse);
            }
            let mut decoded: Vec<_> = decoder.get().unwrap().into_iter().collect();
            decoded.sort_by_key(|f| f.0.clone());
            decoded
        };
        for (field, value) in res {
            print!("{}: {} ", field, value);
        }
        println!("");
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let opts: Opts = Opts::parse();
    match opts.subcmd {
        SubCommand::Decode => decode(),
        SubCommand::SetState(state) => set_state(state),
        SubCommand::Broadlink => broadlink_decode(),
        SubCommand::Irp => irp_decode(),
        SubCommand::SmartIR => smartir::gen_smartir(),
    }
}


#[cfg(test)]
mod test {
    #[test]
    fn test() {
        let b = r"JgDKAIyREjQSEhI0EjUTERESETUTERETETQSEhISETUSNBISEjQTNBISEhESNBM0EjQTNBM0EhISNBI0ExESERISEhESERISEhESNBM0EjQTNBISEhESNBM0EhISERIREhISNBI0E6qRkBM0ExESNBI0ExESEhI0EhISERI0EhISERI0EzQSEhE1EjQTERETETUSNBI1ETUSNRIREjUSNRESEhIREhESERMREhESETUSNRI0EjUSEhATETUSNRISERIQFA8TETYQNhEADQUAAAAAAAAAAAAAAAAAAA==";
        let d = base64::decode(b).unwrap();

    }
}