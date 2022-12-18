use serde::{Deserialize, Serialize};
use serde_json::json;
use strum::IntoEnumIterator;

use crate::{
    broadlink::Recording,
    lennox::{packet::Packet, ControlState, Fan, Mode, Phy},
};

/*
{
   "manufacturer":"Lennox",
   "supportedModels":[
      "2018",
      "2019"
   ],
   "supportedController":"Broadlink",
   "commandsEncoding":"Base64",
   "minTemperature":16.0,
   "maxTemperature":30.0,
   "precision":1,
   "operationModes":[
      "cool",
      "heat_cool",
      "dry",
      "fan"
   ],
   "fanModes":[
      "auto",
      "high",
      "low",
      "mid"
   ],
   */

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CodeFile {
    pub manufacturer: String,
    pub supported_models: Vec<String>,
    pub supported_controller: String,
    pub commands_encoding: String,
    pub min_temperature: f32,
    pub max_temperature: f32,
    pub precision: u8,
    pub operation_modes: Vec<String>,
    pub fan_modes: Vec<String>,
    pub commands: serde_json::Value,
}

/*

- Remote model: RG57A6/BGEFU1
- Heat pump: MWMA018S4-2P
*/

/// Generates a SmartIR code file from all possible states
pub fn gen_smartir() -> anyhow::Result<()> {
    let commands: serde_json::Value = {
        // Commands are nested to represent all possible states, the hierarchy used in other models is:
        // mode -> fan -> temperature

        let mut all_commands = serde_json::Map::new();

        for mode in Mode::iter() {
            let mode_map = all_commands
                .entry(mode.as_ref().to_lowercase())
                .or_insert(serde_json::Map::new().into());
            let mode_map = mode_map.as_object_mut().unwrap();

            for fan in Fan::iter() {
                if fan == Fan::Zero {
                    continue;
                }

                match mode {
                    Mode::Heat | Mode::Dry | Mode::Cool | Mode::Auto => {
                        let fan_map = mode_map
                            .entry(fan.as_ref().to_lowercase())
                            .or_insert(serde_json::Map::new().into());
                        let fan_map = fan_map.as_object_mut().unwrap();

                        for temperature in 17..=30 {
                            let state = ControlState {
                                power: true,
                                mode,
                                fan,
                                temperature: if mode == Mode::Fan {
                                    None
                                } else {
                                    Some(temperature)
                                },
                            };

                            fan_map
                                .insert(format!("{}", temperature), encode_state(&state)?.into());
                        }
                    }
                    Mode::Fan => {
                        let state = ControlState {
                            power: true,
                            mode,
                            fan,
                            temperature: None,
                        };

                        mode_map.insert(fan.as_ref().to_lowercase(), encode_state(&state)?.into());
                    }
                }
            }
        }

        // Add "Off" state
        let off_state = ControlState {
            power: false,
            mode: Mode::Auto,
            fan: Fan::Auto,
            temperature: None,
        };
        all_commands.insert("off".into(), encode_state(&off_state)?.into());

        all_commands.into()
    };

    let code_file = CodeFile {
        manufacturer: "Lennox".into(),
        supported_models: vec!["MWMA018S4-2P".into(), "RG57A6/BGEFU1".into()],
        supported_controller: "Broadlink".into(),
        commands_encoding: "Base64".into(),
        min_temperature: 17.0,
        max_temperature: 30.0,
        precision: 1,
        operation_modes: Mode::iter().map(|m| m.as_ref().to_lowercase()).collect(),
        fan_modes: Fan::iter()
            .filter(|&m| m != Fan::Zero)
            .map(|m| m.as_ref().to_lowercase())
            .collect(),
        commands,
    };

    println!("{}", serde_json::to_string_pretty(&code_file)?);

    Ok(())
}

fn encode_state(state: &ControlState) -> anyhow::Result<String> {
    let packet: Packet = Packet::from_control_state(state)?;
    let pulses = Phy::new().encode(packet.0)?;
    let recording_bytes = Recording::new_ir(pulses).to_bytes();
    Ok(base64::encode(recording_bytes))
}

#[cfg(test)]
mod test {
    use super::gen_smartir;

    #[test]
    fn test_generate() {
        gen_smartir().unwrap();
    }
}
