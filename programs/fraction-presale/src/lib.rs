use anchor_lang::prelude::*;

pub mod errors;
pub mod state;
pub mod instructions;

use instructions::*;

declare_id!("EmcETFRC5ftDYwNn6cHB3zQioNH1z8cRSwx5MZC1BMBU");

#[program]
pub mod fraction_presale {

    use super::*;

    // Setup the presale account
    pub fn initialize_presale(
        ctx: Context<InitializePresale>, 
        pda_bump: u8,
        price: u64,
        max_amount: u64,
        presale_end: u64,
        vesting_end: u64
    ) -> ProgramResult {
        instructions::initialize_presale::handler(ctx, price, max_amount, presale_end, vesting_end)
    }

    // Add fraction to sell in the presale
    pub fn add_fractions_for_sale(ctx: Context<AddFractions>, pda_bump: u8, amount: u64) -> ProgramResult {
        instructions::add_fractions::handler(ctx, amount)
    }

    // Remove fractions to sell in the presale
    pub fn remove_fractions_for_sale(ctx: Context<RemoveFractions>, pda_bump: u8, amount: u64) -> ProgramResult {
        instructions::remove_fractions::handler(ctx, pda_bump, amount)
    }

    pub fn start_presale(ctx: Context<StartPresale>) -> ProgramResult {
        instructions::start_presale::handler(ctx)
    }

    // Withdraw funds used to purchase fractions
    pub fn collect_funds(ctx: Context<CollectFunds>, pda_bump: u8) -> ProgramResult {
        instructions::collect_funds::handler(ctx, pda_bump)
    }

    pub fn init_vesting_account(ctx: Context<InitVesting>, vesting_pda_bump: u8) -> ProgramResult {
        instructions::init_vesting_account::handler(ctx)
    }

    // Buy fractions with redeem tokens
    pub fn purchase_fractions(ctx: Context<PurchaseFractions>, presale_pda_bump: u8, vesting_pda_bump: u8, amount: u64) -> ProgramResult {
        instructions::purchase_fractions::handler(ctx, presale_pda_bump, amount)
    }

    // Unlock fractions after vesting period
    pub fn unlock_fractions(ctx: Context<UnlockFractions>, vesting_pda_bump: u8) -> ProgramResult {
        instructions::unlock_fractions::handler(ctx, vesting_pda_bump)
    }

}


