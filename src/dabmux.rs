use anyhow::anyhow;
use serde_json::Value;

const ZMQ_TIMEOUT : i64 = 2000;

pub struct DabMux {
    ctx : zmq::Context,
    rc_endpoint : String,
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
            rc_endpoint : "tcp://127.0.0.1:12722".to_owned()
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

    pub fn get_rc_parameters(&mut self) -> anyhow::Result<Vec<Param>> {
        let sock = self.ctx.socket(zmq::REQ)?;
        sock.connect(&self.rc_endpoint)?;
        sock.send("showjson", 0)?;

        let mut msg = zmq::Message::new();
        let mut items = [
            sock.as_poll_item(zmq::POLLIN),
        ];
        zmq::poll(&mut items, ZMQ_TIMEOUT).unwrap();
        if items[0].is_readable() {
            sock.recv(&mut msg, 0)?;
            let msg = msg.as_str().ok_or(anyhow!("RC response is not a str"))?;

            // JSON structure:
            // { "module1": { "param1": "value", "param2": "value" }, "module2": { ... } }
            let v: Value = serde_json::from_str(msg)?;
            Self::value_to_params(v)
        }
        else {
            Err(anyhow!("Timeout reading RC"))
        }
    }

    pub fn set_rc_parameter(&mut self, module: &str, param: &str, value: &str) -> anyhow::Result<Value> {
        let sock = self.ctx.socket(zmq::REQ)?;
        sock.connect(&self.rc_endpoint)?;
        sock.send_multipart(["set", module, param, value], 0)?;

        let mut items = [
            sock.as_poll_item(zmq::POLLIN),
        ];
        zmq::poll(&mut items, ZMQ_TIMEOUT).unwrap();
        if items[0].is_readable() {
            let mut parts = sock.recv_multipart(0)?;

            let j : serde_json::Value = parts.drain(..)
                .map(|p| match String::from_utf8(p)
                    {
                        Ok(s) => s,
                        Err(_) => "???".to_owned(),
                    })
            .collect();

            let j_arr = j.as_array().unwrap();

            //eprintln!("SET_RC: {}", j);
            if j_arr[0].as_str() == Some("ok") {
                Ok(j)
            }
            else {
                if j_arr.len() > 0 && j_arr[1].is_string() {
                    Err(anyhow!(format!("Failed to set RC: {}", j_arr[1].as_str().unwrap())))
                }
                else {
                    Err(anyhow!("Failed to set RC: unknown error"))
                }
            }
        }
        else {
            Err(anyhow!("Timeout reading RC"))
        }
    }
}
