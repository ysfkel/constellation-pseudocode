// This contract will be compatible with the Soroban Token interface.
// Most of the code will be similar to the "Token" Soroban example
// https://github.com/stellar/soroban-examples/tree/v20.0.0-rc2/token/src

// A Constellation Token holds balances of component tokens, which also follow the Soroban Token Interface.
// A Constellation Token is initialized with a list of component tokens and their units
// A Constellation Token can be only be minted or burned by the Constellation Minter Burner contract.

use crate::admin::{has_administrator, read_administrator, write_administrator};
use crate::allowance::{read_allowance, spend_allowance, write_allowance};
use crate::balance::{read_balance, receive_balance, spend_balance};
use crate::metadata::{read_decimal, read_name, read_symbol, write_metadata};
use crate::storage_types::{INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD};
use soroban_sdk::token::{self, Interface as _};
use soroban_sdk::{contract, contractimpl, Address, Env, String};
use soroban_sdk::Vec;
use soroban_token_sdk::metadata::TokenMetadata;
use soroban_token_sdk::TokenUtils;

fn check_nonnegative_amount(amount: i128) {
    if amount < 0 {
        panic!("negative amount is not allowed: {}", amount)
    }
}

#[contract]
pub struct ConstellationToken;

#[contractimpl]
impl ConstellationToken {
    pub fn initialize(
        e: Env,
        decimal: u32,
        components: Vec<String>,
        amounts: Vec<u32>,
        admin: Address, // Must be instance of ConstellationMinterBurner contract
        manager: Address, // For future use; manager can rebalance and charge fees
        name: String,
        symbol: String
    ) {
        if has_administrator(&e) {
            panic!("already initialized")
        }
        write_administrator(&e, &admin);

        write_metadata(
            &e,
            TokenMetadata {
                decimal,
                name,
                symbol,
            },
        )

        // Write <Vec> components and <Vec> amounts to persistent storage
        write_components(&e, components, amounts); // TODO: write_components Implementation
    }

    pub fn mint(e: Env, to: Address, amount: i128) {
        check_nonnegative_amount(amount);
        let admin = read_administrator(&e);
        // A user calls the mint() function of the Constellation Minter Burner contract
        // The MinterBurner will receive component tokens from the user
        // Then the MinterBurner will call ContellationToken.mint() with 'to' as the user address (issuance)
        admin.require_auth();

        e.storage()
            .instance()
            .bump(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        receive_balance(&e, to.clone(), amount);
        TokenUtils::new(&e).events().mint(admin, to, amount);
    }

    pub fn burn(e: Env, from: Address, amount: i128) {
        // 'from' will be the MinterBurner contract
        // A user calls the burn() function of the Constellation Minter Burner contract
        // The MinterBurner will receive the user's Constellation Tokens (redemption)
        // The MinterBurner will send the user component tokens
        // Then the MinterBurner will call ContellationToken.burn()
        from.require_auth();
        check_nonnegative_amount(amount);

        e.storage()
            .instance()
            .bump(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_balance(&e, from.clone(), amount);
        TokenUtils::new(&e).events().burn(from, amount);
    }

    pub fn getComponents(e: Env) -> Vec<String> {
        e.storage()
            .instance()
            .bump(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        read_components(&e); // TODO: read_components Implementation
    }

    // Must return values in the same order as getComponents()
    pub fn getAmounts(e: Env) -> Vec<u32> {
        e.storage()
            .instance()
            .bump(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        read_amounts(&e); // TODO: read_amounts Implementation
    }

    // TODO: Implement Dutch Auction contract
    // Auction params are set by the manager and include:
    // - An intermediate component in which prices are denominated
    // - Target components (incl. any components added or removed)
    // - Target amounts for each component
    // - A starting price for each target component
    // - A minimum price for each target component
    // - A price function that gradually lowers the acceptable price in terms of intermediate component (ex. linear, exponential, etc.) 
    AuctionClient.start_rebalance_auctions(&e, target_components, target_amounts, auction_params, intermediate_token);
    
    fn AuctionClient.start_rebalance_auctions(&e: Env, target_components: Vec<Address>, target_amounts: Vec<u32>, auction_params: Vec<AuctionParam>, intermediate_token: Address) {
        // Temporarily add intermediate token to components[] vector if not already included
        for i in 0..target_components.len() {
            // Start a Dutch auction for component[i]
            // During an auction users are allowed to swap (target_amount[i] - current_amount[i]) of component tokens for intermediate token in the direction of (target_amount[i] - current_amount[i])
            // component[i] auction stays open until the component reaches target amount.
            // If the component[i] auction reaches minimum price, the auction will remain open indefinitely until the manager starts a new rebalance
            // Note: Mint and Burn of ConstellationToken can still be performed while auctions are open
            TokenUtils::new(&e).events().start_rebalance_auction(component, target_amount, auction_params);
        }
    }

    // For future use: Allow the Constellation Token manager way to upgrade the associated MinterBurner contract
    // Initially will be disabled
    pub fn set_admin(e: Env, new_admin: Address) {
        let manager = read_manager(&e); // TODO: read_manager Implementation
        manager.require_auth();

        e.storage()
            .instance()
            .bump(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        // Validate that the new admin is an instance of the Constellation Minter Burner contract
        write_administrator(&e, &new_admin);
        TokenUtils::new(&e).events().set_admin(admin, new_admin);
    }
}

// End of ConstellationToken pseudocode
// Most of below is unchanged from from "Token" example

#[contractimpl]
impl token::Interface for ConstellationToken {
    fn allowance(e: Env, from: Address, spender: Address) -> i128 {
        e.storage()
            .instance()
            .bump(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        read_allowance(&e, from, spender).amount
    }

    fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        from.require_auth();

        check_nonnegative_amount(amount);

        e.storage()
            .instance()
            .bump(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        write_allowance(&e, from.clone(), spender.clone(), amount, expiration_ledger);
        TokenUtils::new(&e)
            .events()
            .approve(from, spender, amount, expiration_ledger);
    }

    fn balance(e: Env, id: Address) -> i128 {
        e.storage()
            .instance()
            .bump(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        read_balance(&e, id)
    }

    fn spendable_balance(e: Env, id: Address) -> i128 {
        e.storage()
            .instance()
            .bump(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        read_balance(&e, id)
    }

    fn transfer(e: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(amount);

        e.storage()
            .instance()
            .bump(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);
        TokenUtils::new(&e).events().transfer(from, to, amount);
    }

    fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();

        check_nonnegative_amount(amount);

        e.storage()
            .instance()
            .bump(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_allowance(&e, from.clone(), spender, amount);
        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);
        TokenUtils::new(&e).events().transfer(from, to, amount)
    }

    fn burn_from(e: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth();

        check_nonnegative_amount(amount);

        e.storage()
            .instance()
            .bump(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_allowance(&e, from.clone(), spender, amount);
        spend_balance(&e, from.clone(), amount);
        TokenUtils::new(&e).events().burn(from, amount)
    }

    fn decimals(e: Env) -> u32 {
        read_decimal(&e)
    }

    fn name(e: Env) -> String {
        read_name(&e)
    }

    fn symbol(e: Env) -> String {
        read_symbol(&e)
    }
}
