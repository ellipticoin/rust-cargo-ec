pub use wasm_rpc::error::{Error, ErrorStruct};
pub const INSUFFICIENT_FUNDS: ErrorStruct<'static> = Error {
    code: 1,
    message: "insufficient funds",
};
