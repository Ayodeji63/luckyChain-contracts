// Imported packages

use alloc::string::String;
use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;
use core::marker::PhantomData;
use stylus_sdk::{evm, msg, prelude::*};

pub trait Erc20Params {
    /// Immutable token name
    const NAME: &'static str;

    /// Immutable token symbol
    const SYMBOL: &'static str;

    /// Immutable token decimal places
    const DECIMALS: u8;
}

sol_storage! {
    /// Erc20 implements all ERC-20 methods
    pub struct Erc20<T> {
        mapping(address => uint256) balances;
        /// Maps users to a mapping of each spender's allowance
        mapping(address => mapping(address => uint256)) allowances;
        /// The total supply of the token
        uint256 total_supply;
        /// Used to allow [`Erc20Params`]
        PhantomData<T> phantom;
    }
}

// Declare events and solidity error types
sol! {
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);

    error InsufficientBalance(address from, uint256 have, uint256 want);
    error InsufficientAllowance(address owner, address spender, uint256 have, uint256 want);
}

/// Represents the way methods may fail.
#[derive(SolidityError)]
pub enum Erc20Error {
    InsufficientBalance(InsufficientBalance),
    InsufficientAllowance(InsufficientAllowance),
}

impl<T: Erc20Params> Erc20<T> {
    /// Movement of funds between 2 accounts
    /// (invoked by the external transfer() and transfer_from() functions)

    pub fn _transfer(&mut self, from: Address, to: Address, value: U256) -> Result<(), Erc20Error> {
        let mut sender_balance = self.balances.setter(from);
        let old_sender_balance = sender_balance.get();
        if old_sender_balance < value {
            return Err(Erc20Error::InsufficientBalance(InsufficientBalance {
                from,
                have: old_sender_balance,
                want: value,
            }));
        }
        sender_balance.set(old_sender_balance - value);

        // Increasing receiver balance
        let mut to_balance = self.balances.setter(to);
        let new_to_balance = to_balance.get() + value;
        to_balance.set(new_to_balance);

        // Emitting the transfer event
        evm::log(Transfer { from, to, value });
        Ok(())
    }

    /// Mints `value` tokens to `address`
    pub fn mint(&mut self, address: Address, value: U256) -> Result<(), Erc20Error> {
        // Increasing balance
        let mut balance = self.balances.setter(address);
        let new_balance = balance.get() + value;
        balance.set(new_balance);

        // Increasing total supply
        self.total_supply.set(self.total_supply.get() + value);

        // Emitting the transfer event
        evm::log(Transfer {
            from: Address::ZERO,
            to: address,
            value,
        });

        Ok(())
    }

    /// Burns `value` tokens from `address`
    pub fn burn(&mut self, address: Address, value: U256) -> Result<(), Erc20Error> {
        // Decreasing balance
        let mut balance = self.balances.setter(address);
        let old_balance = balance.get();
        if old_balance < value {
            return Err(Erc20Error::InsufficientBalance(InsufficientBalance {
                from: address,
                have: old_balance,
                want: value,
            }));
        }
        balance.set(old_balance - value);

        // Decreasing the total supply
        self.total_supply.set(self.total_supply.get() - value);

        // Emitting the transfer event
        evm::log(Transfer {
            from: address,
            to: Address::ZERO,
            value,
        });

        Ok(())
    }
}

#[public]
impl<T: Erc20Params> Erc20<T> {
    /// Immutable token name
    pub fn name() -> String {
        T::NAME.into()
    }

    /// Immutable token symbol
    pub fn symbol() -> String {
        T::SYMBOL.into()
    }

    /// Immutable token decimals
    pub fn decimals() -> u8 {
        T::DECIMALS
    }

    /// Total supply of tokens
    pub fn total_supply(&self) -> U256 {
        self.total_supply.get()
    }

    /// Balance of `address`
    pub fn balance_of(&self, owner: Address) -> U256 {
        self.balances.get(owner)
    }

    /// Transfers `value` tokens from msg::sender() to `to`
    pub fn transfer(&mut self, to: Address, value: U256) -> Result<bool, Erc20Error> {
        self._transfer(msg::sender(), to, value)?;
        Ok(true)
    }

    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Erc20Error> {
        let mut sender_allowances = self.allowances.setter(from);
        let mut allowance = sender_allowances.setter(msg::sender());
        let old_allowance = allowance.get();
        if old_allowance < value {
            return Err(Erc20Error::InsufficientAllowance(InsufficientAllowance {
                owner: from,
                spender: msg::sender(),
                have: old_allowance,
                want: value,
            }));
        }

        // Decrease allowance
        allowance.set(old_allowance - value);

        // Calls the internal transfer function
        self._transfer(from, to, value)?;
        Ok(true)
    }

    /// Approve the spenditure of `value` tokens of msg::sender() to `spender()`.
    pub fn approve(&mut self, spender: Address, value: U256) -> bool {
        self.allowances.setter(msg::sender()).insert(spender, value);
        evm::log(Approval {
            owner: msg::sender(),
            spender,
            value,
        });
        true
    }

    pub fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self.allowances.getter(owner).get(spender)
    }
}

// 5810098e367422376897bb2645c5ada5850a99aeec0505a58d38853ebd7f9f31

// cast send --rpc-url 'https://arbitrum-sepolia.blockpi.network/v1/rpc/public' --private-key 0x5810098e367422376897bb2645c5ada5850a99aeec0505a58d38853ebd7f9f31 0x84a4d5235341c6d439f4ff4925f88afb775374f5 "transfer(address, uint256)(bool)" "0x1909Facb64DD70A7C3A68838745bD17549804e74" 1000000000000000000
// https://arbitrum-sepolia.blockpi.network/v1/rpc/public
// cargo stylus deploy -e https://arbitrum-sepolia.blockpi.network/v1/rpc/public --private-key 0x5810098e367422376897bb2645c5ada5850a99aeec0505a58d38853ebd7f9f31