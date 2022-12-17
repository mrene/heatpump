mod broadlink;
mod lennox;
mod pwm;

use anyhow::anyhow;
use bytes::Bytes;
use rbroadlink::Device;
use std::net::Ipv4Addr;

///broadlink_cli --type 0x5216 --host 192.168.1.235 --mac ec0bae9fe2ef

fn main() -> anyhow::Result<()> {
    // let known_ip = Ipv4Addr::new(192, 168, 1, 235);
    // let device = Device::from_ip(known_ip, None).expect("Could not connect to device!");

    // let device = match device {
    //     Device::Remote { remote } => remote,
    //     _ => return Err(anyhow!("Not a remote!")),
    // };

    // let decode = |msg: &[u8]| -> u64 {
    //     let message =
    //         crate::broadlink::Recording::from_bytes(Bytes::copy_from_slice(msg)).unwrap();
    //     let pulses = lennox::PWMDecoder.decode(message.pulses.iter()).unwrap();
    //     decode_message(pulses.iter().copied()).unwrap()
    // };

    // while let Ok(msg) = device.learn_ir() {
    //     println!("{:x}", decode(&msg));
    // }

    Ok(())
}

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
        // use super::*;
        // use crate::lennox::decode_message;

        // let off = include_str!("../captures/off.ir");
        // let message =
        //     crate::broadlink::Recording::from_bytes(hex::decode(off).unwrap().into()).unwrap();
        // let pulses = lennox::PWMDecoder.decode(message.pulses.iter()).unwrap();
        // let msg = decode_message(pulses.iter().copied()).unwrap();
        // assert_eq!(msg, 0xa12347ffffeb)
    }
}
