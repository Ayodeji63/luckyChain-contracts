#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
extern crate alloc;

// Modules and imports
mod erc20;

use crate::erc20::{Erc20, Erc20Error, Erc20Params};
use alloy_primitives::{Address, U256};
use stylus_sdk::{msg, prelude::*};

/// Immutabl definitions
struct StylusTokenParams;
impl Erc20Params for StylusTokenParams {
    const NAME: &'static str = "StylusToken";
    const SYMBOL: &'static str = "STK";
    const DECIMALS: u8 = 18;
}

// Define the entry point as

sol_storage! {
    #[entrypoint]
    struct StylusToken {
        #[borrow]
        Erc20<StylusTokenParams> erc20;
    }
}

// 0x0E2C063754a1c157F288aE04e1E69bbb3c73eEf8

#[public]
#[inherit(Erc20<StylusTokenParams>)]
impl StylusToken {
    pub fn mint(&mut self, value: U256) -> Result<(), Erc20Error> {
        self.erc20.mint(msg::sender(), value)?;
        Ok(())
    }

    /// Mints tokens to another address
    pub fn mint_to(&mut self, to: Address, value: U256) -> Result<(), Erc20Error> {
        self.erc20.mint(to, value)?;
        Ok(())
    }

    pub fn burn(&mut self, value: U256) -> Result<(), Erc20Error> {
        self.erc20.burn(msg::sender(), value)?;
        Ok(())
    }
}

// cargo stylus deploy -e https://sepolia-rollup.arbitrum.io/rpc --private-key 0x5810098e367422376897bb2645c5ada5850a99aeec0505a58d38853ebd7f9f31

// cast call --rpc-url 'https://sepolia-rollup.arbitrum.io/rpc' --private-key 0x5810098e367422376897bb2645c5ada5850a99aeec0505a58d38853ebd7f9f31 0x5ABFbDE3f44b41799D265683dacB4aBA1DEB249D "name()(string)"

// cast send --rpc-url 'https://sepolia-rollup.arbitrum.io/rpc' --private-key 0x5810098e367422376897bb2645c5ada5850a99aeec0505a58d38853ebd7f9f31 0x5ABFbDE3f44b41799D265683dacB4aBA1DEB249D "mint()(uint256)" 10