use std::{
    collections::HashMap,
    io::{self, Write},
    path::Path,
};

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

    /// Decode all possible codes from a SmartIR file
    IrpGrep,

    /// Generate a SmartIR code file from all possible states
    SmartIR,
    
    ReadIr,
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
        let line = line?;
        let line_decoded = hex::decode(&line).or_else(|_| base64::decode(&line));
        let mut line = match line_decoded {
            Ok(line) => line,
            Err(_) => {
                println!("{}", line);
                continue;
            }
        };
        if line.len() == 0 {
            continue;
        }
        line.push(0);
        let recording = Recording::from_bytes(Bytes::copy_from_slice(&line))?;
        // println!("{:?}", recording.pulses.iter().map(|p| p.as_micros()).collect::<Vec<_>>());

        // recording.repeat_count = 5;

        // println!("Base64: {}", base64::encode(recording.to_bytes().as_ref()));
        // println!("Hex: {}", hex::encode(recording.to_bytes().as_ref()));

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
        println!("\n");
    }

    Ok(())
}

const IRP_48_NEC1: &'static str = "{38.4k,564}<1,-1|1,-3>(16,-8,D:8,S:8,F:8,~F:8,E:8,~E:8,1,^108m,(16,-4,1,^108m)*)[D:0..255,S:0..255=255-D,F:0..255,E:0..255]";
const IRP_AP: &'static str = "{38.0k,522,msb}<1,-1|1,-3>((4476u,-4476u,A:48,1,-4476u)*,(4476u,-4476u,B:48,1,-101m))[A:0..281474976710656,B:0..281474976710656]";
const IRP_LENNOX: &'static str = "{38.0k,487,msb}<1,-1|1,-3>(9,-9,A:48,1,-9,9,-9,B:48,1,-9)[A:0..281474976710656,B:0..281474976710656]";
fn irp_grep() -> anyhow::Result<()> {
    use irp::Irp;

    let nfa = Irp::parse(IRP_LENNOX)
        .expect("irp parse")
        .compile()
        .expect("irp compile");

    let codefile: serde_json::Value = serde_json::from_reader(io::stdin())?;
    let codefile = codefile.as_object().unwrap();
    let commands = codefile.get("commands").unwrap();

    let mut tested = 0;
    let mut matched = 0;

    // Recurse through all commands
    let mut queue = Vec::new();
    queue.push(("root".to_string(), commands.as_object().unwrap()));

    while let Some((command_path, commands)) = queue.pop() {
        for (name, value) in commands {
            match value {
                serde_json::Value::Object(map) => {
                    queue.push((format!("{}/{}", command_path, name), map));
                }
                serde_json::Value::String(s) => {
                    tested += 1;
                    if let Ok(decoded) = decode_base64_irp(&nfa, s) {
                        matched += 1;
                        // dbg!(&decoded);
                        //{38.0k,487,msb}<1,-1|1,-3>(9,-9,A:48,1,-9,9,-9,B:48,1,-9){A=0xa18a40ffff54,B=0x5e75bf0000ab}
                        let a = *decoded.get("A").unwrap();
                        if a == 0xa18a40ffff54 {
                            //auto/min/17
                            println!("*** FOUND ** {}/{}", command_path, name);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if matched > 0 {
        if matched == tested {
            println!("***** All {} commands matched *****", matched);
        } else {
            println!("Matched {}/{} commands", matched, tested);
        }
    }

    Ok(())
}

fn irp_decode() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    for line in stdin.lines() {
        let line = line?;
        let line = hex::decode(&line).or_else(|_| base64::decode(&line))?;
        let recording = Recording::from_bytes(Bytes::from(line))?;

        let res = irp_decode_one(IRP_48_NEC1, &recording)?;
        // for (field, value) in res {
        //     print!("{}: {} ", field, value);
        // }
        println!("{}", serde_json::to_string(&res).unwrap());
        println!("");
    }

    Ok(())
}

fn irp_decode_one(protocol: &str, data: &Recording) -> anyhow::Result<HashMap<String, i64>> {
    use irp::Irp;

    let irp = Irp::parse(protocol)
        .expect("irp parse");

    dbg!(&irp);
    
    let nfa = irp.compile()
        .expect("irp compile");

    let pulses = InfraredData::from_u32_slice(&data.to_pulses());

    let mut decoder = nfa.decoder(100, 30, 20000);
    for pulse in pulses {
        decoder.input(pulse);
    }
    Ok(decoder.get().unwrap_or_default())
}

fn decode_base64_irp(nfa: &irp::NFA, data: &str) -> anyhow::Result<HashMap<String, i64>> {
    let pulses = {
        let recording = Recording::from_bytes(Bytes::copy_from_slice(&base64::decode(data)?))?;
        InfraredData::from_u32_slice(&recording.to_pulses())
    };

    let mut decoder = nfa.decoder(100, 30, 20000);
    for pulse in pulses {
        decoder.input(pulse);
    }

    decoder.get().ok_or(anyhow::anyhow!("no match"))
}

fn read_ir() -> anyhow::Result<()> {
    use rbroadlink::Device;
    use std::net::Ipv4Addr;

    // Create a device by IP
    // Note: Devices only support Ipv4 addresses
    let known_ip = Ipv4Addr::new(192, 168, 1, 235);
    let device = Device::from_ip(known_ip, None).expect("Could not connect to device!");
    match device {
        Device::Remote { remote } => {
            let mut ir = remote.learn_ir().expect("reading ir");
            ir.push(0);
            let recording = Recording::from_bytes(Bytes::from(ir))?;
            let decoded = irp_decode_one(IRP_48_NEC1, &recording)?;

            for (field, value) in decoded {
                print!("{}: {:x} ", field, value);
            }
            println!("");
        }
    };
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let opts: Opts = Opts::parse();
    match opts.subcmd {
        SubCommand::Decode => decode(),
        SubCommand::SetState(state) => set_state(state),
        SubCommand::Broadlink => broadlink_decode(),
        SubCommand::Irp => irp_decode(),
        SubCommand::IrpGrep => irp_grep(),
        SubCommand::SmartIR => smartir::gen_smartir(),
        SubCommand::ReadIr => read_ir(),
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
