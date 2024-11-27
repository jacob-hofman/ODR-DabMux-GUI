/*
 * A Configuration and Control UI for ODR-DabMux
 * Copyright (C) 2024 Matthias P. Braendli
 *
 * This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public
 * License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later
 * version.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied
 * warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.
 */
use anyhow::anyhow;
use serde::Deserialize;
use serde_json::Value;
use log::info;

const ZMQ_TIMEOUT : i64 = 2000;

pub struct DabMux {
    ctx : zmq::Context,
    rc_endpoint : String,
    stats_endpoint : String,
}

pub struct Param {
    pub module : String,
    pub param : String,
    pub value : String,
}


impl DabMux {
    pub fn new() -> Self {
        let ctx = zmq::Context::new();
        Self {
            ctx,
            rc_endpoint : "tcp://127.0.0.1:12722".to_owned(),
            stats_endpoint : "tcp://127.0.0.1:12720".to_owned(),
        }
    }

    fn value_to_params(v: Value) -> anyhow::Result<Vec<Param>> {
        let root = v.as_object().ok_or(anyhow!("RC data is not a JSON object"))?;

        let mut all_params = Vec::new();

        for (module_name, params_value) in root {
            let params = params_value.as_object().ok_or(anyhow!("RC module {} is not a JSON object", module_name))?;

            // ODR-DabMux doesn't allow setting only label through the RC, so we have to merge them together
            if let (Some(Value::String(l)), Some(Value::String(sl))) = (params.get("label").clone(), params.get("shortlabel").clone()) {
                let value = format!("{},{}", l, sl);
                all_params.push(
                    Param {
                        module: module_name.to_owned(),
                        param: "label".to_owned(),
                        value
                    });
            }

            for (param_name, value_json) in params {
                if !(param_name == "label" || param_name == "shortlabel") {
                    let value = match value_json {
                        Value::Null => "null".to_owned(),
                        Value::Bool(b) => if *b { "1".to_owned() } else { "0".to_owned() },
                        Value::Number(n) => n.to_string(),
                        Value::String(s) => s.clone(),
                        Value::Array(_) => return Err(anyhow!(format!("Unexpected array in {}.{}", module_name, param_name))),
                        Value::Object(_) => return Err(anyhow!(format!("Unexpected object in {}.{}", module_name, param_name))),
                    };

                    all_params.push(
                        Param {
                            module: module_name.to_owned(),
                            param: param_name.to_owned(),
                            value
                        });
                }
            }
        }

        Ok(all_params)
    }

    fn poll_multipart(sock: &zmq::Socket) -> anyhow::Result<Vec<String>> {
        let mut items = [ sock.as_poll_item(zmq::POLLIN), ];
        zmq::poll(&mut items, ZMQ_TIMEOUT).unwrap();
        if items[0].is_readable() {
            let mut parts = Vec::new();
            for part in sock.recv_multipart(0)? {
                let p = String::from_utf8(part)?;
                parts.push(p);
            }
            Ok(parts)
        }
        else {
            Err(anyhow!("ZMQ timeout"))
        }
    }

    fn poll_message(sock: &zmq::Socket) -> anyhow::Result<String> {
        let parts = Self::poll_multipart(&sock)?;
        if parts.len() == 1 {
            Ok(parts[0].clone())
        }
        else {
            info!("multipart returned: {}", parts.join(","));
            return Err(anyhow!("unexpected multipart answer"));
        }
    }

    pub fn get_rc_parameters(&mut self) -> anyhow::Result<Vec<Param>> {
        let sock = self.ctx.socket(zmq::REQ)?;
        sock.connect(&self.rc_endpoint)?;
        sock.send("showjson", 0)?;

        let msg = Self::poll_message(&sock)?;

        // JSON structure:
        // { "module1": { "param1": "value", "param2": "value" }, "module2": { ... } }
        let v: Value = serde_json::from_str(&msg)?;
        Self::value_to_params(v)
    }

    pub fn set_rc_parameter(&mut self, module: &str, param: &str, value: &str) -> anyhow::Result<()> {
        let sock = self.ctx.socket(zmq::REQ)?;
        sock.connect(&self.rc_endpoint)?;
        sock.send_multipart(["set", module, param, value], 0)?;

        let resp = Self::poll_multipart(&sock)?;

        //eprintln!("SET_RC: {}", j);
        if resp.len() > 0 && resp[0] == "ok" {
            Ok(())
        }
        else {
            if resp.len() > 1 && resp[0] == "fail" {
                Err(anyhow!(format!("Failed to set RC: {}", resp[1])))
            }
            else {
                Err(anyhow!("Failed to set RC: unknown error"))
            }
        }
    }

    pub fn get_stats(&mut self) -> anyhow::Result<Stats> {
        let sock = self.ctx.socket(zmq::REQ)?;
        sock.connect(&self.stats_endpoint)?;
        sock.send("info", 0)?;

        let info_json : Value = serde_json::from_str(&Self::poll_message(&sock)?)?;

        if let Some(service) = info_json.get("service")
            .and_then(|v| v.as_str())
        {
            if !service.starts_with("ODR-DabMux") {
                info!("stats info service is {}", service);
                return Err(anyhow!("Wrong service in stats"));
            }

            let version = info_json.get("version")
                .and_then(|v| v.as_str())
                .or(Some("UNKNOWN"))
                .unwrap()
                .to_owned();

            sock.send("values", 0)?;
            let values_json : Value = serde_json::from_str(&Self::poll_message(&sock)?)?;
            match values_json.get("values")
                .and_then(|v| v.as_object()) {
                Some(v) => {
                    let mut input_stats : Vec<(String, InputStat)> = Vec::new();

                    for (k, v) in v {
                        let is = v.get("inputstat")
                            .ok_or(anyhow!("inputstat missing"))?;
                        let stat : InputStat = serde_json::from_value(is.clone())?;
                        input_stats.push((k.clone(), stat));
                    }

                    input_stats.sort_by_key(|v| v.0.clone());

                    Ok(Stats { version, input_stats })
                },
                None => Err(anyhow!("values isn't an object")),
            }
        }
        else {
            Err(anyhow!("Missing service in stats response"))
        }
    }
}

#[derive(Debug)]
pub struct Stats {
    pub version : String,
    pub input_stats : Vec<(String, InputStat)>,
}

#[derive(Debug, Deserialize)]
pub struct InputStat {
    pub max_fill : u32,
    pub min_fill : u32,
    pub num_underruns : u64,
    pub num_overruns : u64,
    pub peak_left : i32,
    pub peak_right : i32,
    pub peak_left_slow : i32,
    pub peak_right_slow : i32,
    pub state : Option<String>,
    pub version : Option<String>,
    pub uptime : Option<u64>,
    pub last_tist_offset : i32,
}
