use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug)]
pub struct MarketData {
    ltp: Option<f64>,
    volume: Option<u64>,
    oi: Option<u64>,
    close_price: Option<f64>,
    bid_price: Option<f64>,
    bid_qty: Option<u64>,
    ask_price: Option<f64>,
    ask_qty: Option<u64>,
    prev_oi: Option<u64>,
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug)]
pub struct OptionGreeks {
    vega: Option<f64>,
    theta: Option<f64>,
    gamma: Option<f64>,
    delta: Option<f64>,
    iv: Option<f64>,
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug)]
pub struct OptionData {
    instrument_key: String,
    market_data: Option<MarketData>,
    option_greeks: Option<OptionGreeks>,
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug)]
pub struct Instrument {
    expiry: String,
    strike_price: f64,
    underlying_key: String,
    underlying_spot_price: f64,
    call_options: OptionData,
    put_options: OptionData,
}

#[wasm_bindgen]
pub fn print_instruments(json_str: &str) {
    match serde_json::from_str::<Vec<Instrument>>(json_str) {
        Ok(instruments) => {
            // Successfully parsed instruments, now log each one
            for instrument in instruments {
                let instrument_json = serde_json::to_string(&instrument)
                    .unwrap_or_else(|_| String::from("Failed to serialize instrument"));
                console::log_1(&JsValue::from_str(&instrument_json));
            }
        }
        Err(err) => {
            // Log the error to the browser console
            console::log_1(&JsValue::from_str(&format!(
                "Failed to parse JSON: {:?}",
                err
            )));
        }
    }
}
