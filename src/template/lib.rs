#![feature(proc_macro_hygiene)]

#[cfg(not(test))]
extern crate wee_alloc;

#[cfg(not(test))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
extern crate alloc;
#[cfg(not(test))]
extern crate ellipticoin;
extern crate wasm_rpc;
extern crate wasm_rpc_macros;

#[cfg(test)]
extern crate ellipticoin_test_framework;
#[cfg(test)]
extern crate mock_ellipticoin as ellipticoin;
mod error;
pub mod $PACKAGE_NAME;
