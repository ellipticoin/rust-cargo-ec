pub use wasm_rpc::error::{Error, ErrorStruct};
pub const INSUFFICIENT_FUNDS: ErrorStruct<'static> = Error {
    code: 2,
    message: "insufficient funds",
};
