use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

// Define a struct to hold the parameters, named ScanStrategyParams
#[wasm_bindgen]
#[derive(Debug)]
pub struct ScanStrategyParams {
    instrument_key: String,
    strategy: String,
    access_token: String,
}

// Implement a constructor for the struct
#[wasm_bindgen]
impl ScanStrategyParams {
    #[wasm_bindgen(constructor)]
    pub fn new(
        instrument_key: String,
        strategy: String,
        access_token: String,
    ) -> ScanStrategyParams {
        ScanStrategyParams {
            instrument_key,
            strategy,
            access_token,
        }
    }
}

// The function that processes the parameters, now named scan_strategy
#[wasm_bindgen]
pub fn scan_strategy(params: &ScanStrategyParams) -> String {
    let result = format!(
        "Instrument: {}, Strategy: {}, Token: {}",
        params.instrument_key, params.strategy, params.access_token
    );
    log_to_console(&result);
    result
}

// Function to log a message to the console
fn log_to_console(message: &str) {
    // Convert the Rust string to a JavaScript string (JsValue)
    let js_message = JsValue::from_str(message);

    // Log the message using web_sys::console::log_1
    web_sys::console::log_1(&js_message);
}
