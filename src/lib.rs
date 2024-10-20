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
pub struct BearCallSpreadParams {
    optionchain: String,
    bid_ask_spread: bool,
    risk_reward_ratio: bool,
    breakeven_percentage_sort: bool,
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug)]
pub struct CreditSpread {
    sell_strike: f64,
    buy_strike: f64,
    spread: f64,
    net_credit: f64,
    max_profit: f64,
    max_loss: f64,
    breakeven: f64,
    breakeven_percentage: f64, // New key added
    type_: String,
}

#[wasm_bindgen]
pub fn bear_call_spread(params: JsValue) -> String {
    const NIFTY_LOTSIZE: f64 = 25.0;

    let params: BearCallSpreadParams = match from_value(params) {
        Ok(p) => p,
        Err(_) => return String::from("Failed to parse parameters"),
    };

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
                .iter()
                .enumerate()
                .flat_map(|(i, lower)| {
                    sorted_otm_strikes[i + 1..]
                        .iter()
                        .map(move |higher| (lower.clone(), higher.clone()))
                })
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
                    let breakeven = (lower.strike_price + (net_credit / NIFTY_LOTSIZE)).ceil();

                    // Calculate breakeven_percentage and trim it to 2 decimal places without rounding up
                    let breakeven_percentage = ((breakeven - lower.underlying_spot_price).abs()
                        / lower.underlying_spot_price)
                        * 100.0;
                    let breakeven_percentage_trimmed =
                        (breakeven_percentage * 100.0).floor() / 100.0;

                    Some(CreditSpread {
                        sell_strike: lower.strike_price,
                        buy_strike: higher.strike_price,
                        spread,
                        net_credit,
                        max_profit,
                        max_loss,
                        breakeven,
                        breakeven_percentage: breakeven_percentage_trimmed,
                        type_: String::from("CE"),
                    })
                })
                .collect();

            // Sort by breakeven_percentage in descending order if breakeven_percentage_sort is true
            if params.breakeven_percentage_sort {
                credit_spreads.sort_by(|a, b| {
                    b.breakeven_percentage
                        .partial_cmp(&a.breakeven_percentage)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }

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

#[wasm_bindgen]
pub fn bull_put_spread(params: JsValue) -> String {
    const NIFTY_LOTSIZE: f64 = 25.0;

    let params: BearCallSpreadParams = match from_value(params) {
        Ok(p) => p,
        Err(_) => return String::from("Failed to parse parameters"),
    };

    let optionchain = &params.optionchain;

    match serde_json::from_str::<Vec<Instrument>>(optionchain) {
        Ok(instruments) => {
            let otm_strikes: Vec<Instrument> = instruments
                .into_iter()
                .filter(|instrument| {
                    let is_otm = instrument.strike_price < instrument.underlying_spot_price;

                    let has_valid_market_data = instrument
                        .put_options
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
                b.strike_price
                    .partial_cmp(&a.strike_price)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let put_credit_spread_pairs: Vec<(Instrument, Instrument)> = sorted_otm_strikes
                .iter()
                .enumerate()
                .flat_map(|(i, higher)| {
                    sorted_otm_strikes[i + 1..]
                        .iter()
                        .map(move |lower| (higher.clone(), lower.clone()))
                })
                .collect();

            let mut credit_spreads: Vec<CreditSpread> = put_credit_spread_pairs
                .into_iter()
                .filter_map(|(higher, lower)| {
                    let higher_ltp = higher
                        .put_options
                        .as_ref()
                        .and_then(|data| data.market_data.as_ref())
                        .and_then(|market_data| market_data.ltp)
                        .unwrap_or(0.0);

                    let lower_ltp = lower
                        .put_options
                        .as_ref()
                        .and_then(|data| data.market_data.as_ref())
                        .and_then(|market_data| market_data.ltp)
                        .unwrap_or(0.0);

                    let spread = (higher.strike_price - lower.strike_price) * NIFTY_LOTSIZE;
                    let net_credit = (higher_ltp - lower_ltp) * NIFTY_LOTSIZE;
                    let max_profit = net_credit.ceil();
                    let max_loss = (spread - net_credit).ceil();
                    let breakeven = (lower.strike_price - (net_credit / NIFTY_LOTSIZE)).ceil();

                    // Calculate breakeven_percentage and trim it to 2 decimal places without rounding up
                    let breakeven_percentage = ((breakeven - lower.underlying_spot_price).abs()
                        / lower.underlying_spot_price)
                        * 100.0;
                    let breakeven_percentage_trimmed =
                        (breakeven_percentage * 100.0).floor() / 100.0;

                    Some(CreditSpread {
                        sell_strike: higher.strike_price,
                        buy_strike: lower.strike_price,
                        spread,
                        net_credit,
                        max_profit,
                        max_loss,
                        breakeven,
                        breakeven_percentage: breakeven_percentage_trimmed, // Set trimmed value
                        type_: String::from("PE"),
                    })
                })
                .collect();

            // Sort by breakeven_percentage in descending order if breakeven_percentage_sort is true
            if params.breakeven_percentage_sort {
                credit_spreads.sort_by(|a, b| {
                    b.breakeven_percentage
                        .partial_cmp(&a.breakeven_percentage)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }

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
