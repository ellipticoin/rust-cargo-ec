use crate::error;
use ellipticoin::{export, get_memory, sender, set_memory};
use wasm_rpc::Value;

use wasm_rpc::error::Error;
enum Namespace {
    Balances,
}

#[export]
mod $PACKAGE_NAME {
    pub fn constructor(initial_supply: u64) {
        set_balance(&sender(), initial_supply)
    }

    pub fn transfer(to: Vec<u8>, amount: u64) -> Result<Value, Error> {
        if get_balance(&sender()) >= amount {
            set_balance(&sender(), get_balance(&sender()) - amount);
            set_balance(&to, get_balance(&to) + amount);
            Ok(Value::Null)
        } else {
            Err(error::INSUFFICIENT_FUNDS)
        }
    }

    fn get_balance(address: &[u8]) -> u64 {
        get_memory([vec![Namespace::Balances as u8], address.to_vec()].concat())
    }

    fn set_balance(address: &[u8], balance: u64) {
        set_memory(
            [vec![Namespace::Balances as u8], address.to_vec()].concat(),
            balance,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ellipticoin::set_sender;
    use ellipticoin_test_framework::{alice, bob};

    #[test]
    fn test_constructor() {
        set_sender(alice());
        constructor(100);
        assert_eq!(get_balance(&alice()), 100);
    }

    #[test]
    fn test_transfer() {
        set_sender(alice());
        constructor(100);
        transfer(bob(), 20).unwrap();
        assert_eq!(get_balance(&alice()), 80);
        assert_eq!(get_balance(&bob()), 20);
    }

    #[test]
    fn test_transfer_insufficient_funds() {
        set_sender(alice());
        constructor(100);
        assert!(transfer(bob(), 120).is_err());
    }
}
