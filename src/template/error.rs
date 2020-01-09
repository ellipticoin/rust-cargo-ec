pub use wasm_rpc::error::Error;
lazy_static! {
    pub static ref INSUFFICIENT_FUNDS: Error =
        (1, "insufficient funds".to_string());
}
