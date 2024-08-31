use serde::{Deserialize, Serialize};
use serde_json;
use wasm_bindgen::prelude::*;
use web_sys::console;

#[derive(Serialize, Deserialize, Debug, Clone)] // Derive Clone trait
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

#[derive(Serialize, Deserialize, Debug, Clone)] // Derive Clone trait
pub struct OptionGreeks {
    vega: Option<f64>,
    theta: Option<f64>,
    gamma: Option<f64>,
    delta: Option<f64>,
    iv: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)] // Derive Clone trait
pub struct OptionData {
    instrument_key: String,
    market_data: Option<MarketData>,
    option_greeks: Option<OptionGreeks>,
}

#[derive(Serialize, Deserialize, Debug, Clone)] // Derive Clone trait
pub struct Instrument {
    expiry: String,
    strike_price: f64,
    underlying_key: String,
    underlying_spot_price: f64,
    call_options: Option<OptionData>,
    put_options: Option<OptionData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreditSpread {
    sell_strike: f64,
    buy_strike: f64,
    spread: f64,
    net_credit: f64,
    max_profit: f64,
    max_loss: f64,
}

#[wasm_bindgen]
pub fn get_credit_spreads(json_str: &str) -> String {
    const NIFTY_LOTSIZE: f64 = 25.0;

    match serde_json::from_str::<Vec<Instrument>>(json_str) {
        Ok(instruments) => {
            // Filter instruments where strike_price > underlying_spot_price
            // and market_data is not empty and ltp is not null
            let otm_strikes: Vec<Instrument> = instruments
                .into_iter()
                .filter(|instrument| {
                    let is_otm = instrument.strike_price > instrument.underlying_spot_price;

                    let has_valid_market_data = instrument
                        .call_options
                        .as_ref()
                        .and_then(|data| data.market_data.as_ref())
                        .map_or(false, |market_data| market_data.ltp.is_some());

                    is_otm && has_valid_market_data
                })
                .collect();

            // Sort the otm_strikes by strike_price in ascending order
            let mut sorted_otm_strikes = otm_strikes;
            sorted_otm_strikes.sort_by(|a, b| {
                a.strike_price
                    .partial_cmp(&b.strike_price)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            // Create pairs of consecutive items
            let call_credit_spread_pairs: Vec<(Instrument, Instrument)> = sorted_otm_strikes
                .windows(2)
                .map(|window| (window[0].clone(), window[1].clone()))
                .collect();

            // Create CreditSpread objects
            let credit_spreads: Vec<CreditSpread> = call_credit_spread_pairs
                .into_iter()
                .filter_map(|(lower, higher)| {
                    let lower_ltp = lower
                        .call_options
                        .as_ref()
                        .and_then(|data| data.market_data.as_ref())
                        .and_then(|market_data| market_data.ltp)
                        .unwrap_or(0.0);

                    let higher_ltp = higher
                        .call_options
                        .as_ref()
                        .and_then(|data| data.market_data.as_ref())
                        .and_then(|market_data| market_data.ltp)
                        .unwrap_or(0.0);

                    let spread = (higher.strike_price - lower.strike_price) * NIFTY_LOTSIZE;
                    let net_credit = (lower_ltp - higher_ltp) * NIFTY_LOTSIZE;
                    let max_profit = net_credit.ceil(); // Round up to zero decimal places
                    let max_loss = (spread - net_credit).ceil(); // Round up to zero decimal places

                    Some(CreditSpread {
                        sell_strike: lower.strike_price,
                        buy_strike: higher.strike_price,
                        spread,
                        net_credit,
                        max_profit,
                        max_loss,
                    })
                })
                .collect();

            // Convert the credit_spreads array to a JSON string
            serde_json::to_string(&credit_spreads)
                .unwrap_or_else(|_| String::from("Failed to serialize credit spreads"))
        }
        Err(err) => {
            // Log the error to the browser console
            console::log_1(&JsValue::from_str(&format!(
                "Failed to parse JSON: {:?}",
                err
            )));
            String::from("Failed to parse JSON")
        }
    }
}
