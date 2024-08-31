use serde::{Deserialize, Serialize};
use serde_json;
use wasm_bindgen::prelude::*;
use web_sys::console;

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

#[derive(Serialize, Deserialize, Debug)]
pub struct OptionGreeks {
    vega: Option<f64>,
    theta: Option<f64>,
    gamma: Option<f64>,
    delta: Option<f64>,
    iv: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OptionData {
    instrument_key: String,
    market_data: Option<MarketData>,
    option_greeks: Option<OptionGreeks>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Instrument {
    expiry: String,
    strike_price: f64,
    underlying_key: String,
    underlying_spot_price: f64,
    call_options: Option<OptionData>,
    put_options: Option<OptionData>,
}

#[wasm_bindgen]
pub fn print_instruments(json_str: &str) {
    match serde_json::from_str::<Vec<Instrument>>(json_str) {
        Ok(instruments) => {
            // Filter instruments where strike_price > underlying_spot_price
            // and market_data is not empty and ltp is not null
            let otm_strikes: Vec<&Instrument> = instruments
                .iter()
                .filter(|&instrument| {
                    // Check if the strike price is greater than the underlying spot price
                    let is_otm = instrument.strike_price > instrument.underlying_spot_price;

                    // Check if the market_data is not empty and ltp is not null
                    let has_valid_market_data = instrument
                        .call_options
                        .as_ref()
                        .and_then(|data| data.market_data.as_ref())
                        .map_or(false, |market_data| market_data.ltp.is_some());

                    is_otm && has_valid_market_data
                })
                .collect();

            // Log each filtered instrument to the browser console
            for instrument in otm_strikes {
                let instrument_json = serde_json::to_string(instrument)
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
