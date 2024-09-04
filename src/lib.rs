use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::from_value;
use wasm_bindgen::prelude::*;
use web_sys::console;

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OptionGreeks {
    vega: Option<f64>,
    theta: Option<f64>,
    gamma: Option<f64>,
    delta: Option<f64>,
    iv: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OptionData {
    instrument_key: String,
    market_data: Option<MarketData>,
    option_greeks: Option<OptionGreeks>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    breakeven: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BearCallSpreadParams {
    optionchain: String,
    bid_ask_spread: bool,
    risk_reward_ratio: bool,
}

#[wasm_bindgen]
pub fn bear_call_spread(params: JsValue) -> String {
    const NIFTY_LOTSIZE: f64 = 25.0;

    // Deserialize the params from JsValue
    let params: BearCallSpreadParams = match from_value(params) {
        Ok(p) => p,
        Err(_) => return String::from("Failed to parse parameters"),
    };

    // Extract the option chain JSON string from the params
    let optionchain = &params.optionchain;

    match serde_json::from_str::<Vec<Instrument>>(optionchain) {
        Ok(instruments) => {
            let otm_strikes: Vec<Instrument> = instruments
                .into_iter()
                .filter(|instrument| {
                    let is_otm = instrument.strike_price > instrument.underlying_spot_price;

                    let has_valid_market_data = instrument
                        .call_options
                        .as_ref()
                        .and_then(|data| data.market_data.as_ref())
                        .map_or(false, |market_data| {
                            let ltp_is_some = market_data.ltp.is_some();
                            let bid_ask_diff_ok =
                                match (market_data.bid_price, market_data.ask_price) {
                                    (Some(bid), Some(ask)) => (ask - bid).abs() <= 2.0,
                                    _ => false,
                                };
                            ltp_is_some && (!params.bid_ask_spread || bid_ask_diff_ok)
                        });

                    is_otm && has_valid_market_data
                })
                .collect();

            let mut sorted_otm_strikes = otm_strikes;
            sorted_otm_strikes.sort_by(|a, b| {
                a.strike_price
                    .partial_cmp(&b.strike_price)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let call_credit_spread_pairs: Vec<(Instrument, Instrument)> = sorted_otm_strikes
                .windows(2)
                .map(|window| (window[0].clone(), window[1].clone()))
                .collect();

            let mut credit_spreads: Vec<CreditSpread> = call_credit_spread_pairs
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
                    let max_profit = net_credit.ceil();
                    let max_loss = (spread - net_credit).ceil();
                    let breakeven = (lower.strike_price + (net_credit / NIFTY_LOTSIZE)).ceil(); // Round up breakeven

                    Some(CreditSpread {
                        sell_strike: lower.strike_price,
                        buy_strike: higher.strike_price,
                        spread,
                        net_credit,
                        max_profit,
                        max_loss,
                        breakeven,
                    })
                })
                .collect();

            // Apply risk-reward ratio filter if enabled
            if params.risk_reward_ratio {
                credit_spreads.retain(|spread| spread.max_loss <= 3.0 * spread.max_profit);
            }

            serde_json::to_string(&credit_spreads)
                .unwrap_or_else(|_| String::from("Failed to serialize credit spreads"))
        }
        Err(err) => {
            console::log_1(&JsValue::from_str(&format!(
                "Failed to parse JSON: {:?}",
                err
            )));
            String::from("Failed to parse JSON")
        }
    }
}
