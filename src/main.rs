mod broadlink;

fn main() {
    let off = include_str!("../captures/off.ir");
    println!("{:?}", hex::decode(off).unwrap());
}
