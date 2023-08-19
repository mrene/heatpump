/*

Generic IR modem
----------------

Devices/Broadlink/Base64/Hex/Pronto/Raw/irp::Message <> Recording

Recording -> Modem -> HashMap

HashMap -> Modem -> Recording -> ...


CLI:

# Sources/sinks
# broadlink:base64, broadlink:hex:

# Reads IR messages from one source and write it to another
copy  -i [input/broadlink:base64] -o [output/broadlink:base64]

# Reads IR messages from input, demodulates it according to a specified protocol and outputs decoded payloads to stdout
demod -i [input] -p [protocol]

# Reads JSON-encoded payloads from stdin, modulates it according to protocol and writes them to [output]
mod -o [output] -o [protocol]

*/

pub mod codecs;
pub use codecs::{create_codec, Codec, CodecError, CodecType};

pub mod devices;
pub use devices::{create_device, Device, DeviceError, DeviceType};

pub mod irp;