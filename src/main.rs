mod broadlink;
mod lennox;

use lennox::{decode_message, PWMDecoder};

fn main() -> anyhow::Result<()> {
    Ok(())
}

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
        use super::*;

        let off = include_str!("../captures/off.ir");
        let message = crate::broadlink::Recording::from_bytes(hex::decode(off).unwrap().into()).unwrap();
        let pulses = lennox::PWMDecoder.decode(message.pulses.iter()).unwrap();

        let msg = decode_message(pulses.iter().copied()).unwrap();
        println!("msg: {:x}", msg);
    }
}
