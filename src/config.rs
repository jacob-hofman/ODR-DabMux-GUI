use std::fs;
use anyhow::Context;
use serde::{Deserialize, Serialize};

type Protection = u8;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Service {
    pub sid: u32,
    pub ecc: u8,
    pub label: String,
    pub shortlabel: String,
    pub input_port: u16,
    pub bitrate: u32,
    pub protection: Protection,
}

impl Service {
    pub fn sid_hex(&self) -> String {
        format!("{:04X}", self.sid)
    }

    pub fn ecc_hex(&self) -> String {
        format!("{:02X}", self.ecc)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub instance_name: String,
    pub tist: bool,
    pub tist_offset: i32,
    // TODO tai_clock_bulletins
    pub ensemble_id: u16,
    pub ensemble_ecc: u8,
    pub ensemble_label: String,
    pub ensemble_shortlabel: String,
    pub output_edi_port: u16,
    pub services: Vec<Service>,
}

impl Config {
    pub fn ensemble_id_hex(&self) -> String {
        format!("{:04X}", self.ensemble_id)
    }

    pub fn ensemble_ecc_hex(&self) -> String {
        format!("{:02X}", self.ensemble_ecc)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            instance_name: "CHANGEME".to_owned(),
            tist: true,
            tist_offset: 0,
            ensemble_id: 0x4FFF,
            ensemble_ecc: 0xE1,
            ensemble_label: "OpenDigitalRadio".to_owned(),
            ensemble_shortlabel: "ODR".to_owned(),
            output_edi_port: 8951,
            services: vec![
               Service {
                   sid: 0x4DAA,
                   ecc: 0xE1,
                   label: "nothing".to_owned(),
                   shortlabel: "no".to_owned(),
                   input_port: 9001,
                   bitrate: 128,
                   protection: 2
               }
            ],
        }
    }
}

const CONFIGFILE : &str = "odr-dabmux-gui-config.toml";

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        if std::path::Path::new(CONFIGFILE).exists() {
            let file_contents = fs::read_to_string(CONFIGFILE)?;
            toml::from_str(&file_contents).context("parsing config file")
        }
        else {
            Ok(Default::default())
        }
    }

    pub fn store(&self) -> anyhow::Result<()> {
        fs::write(CONFIGFILE, toml::to_string_pretty(&self)?)
            .context("writing config file")
    }
}
