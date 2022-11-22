use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, Token, Mint, TokenAccount};

use crate::state::*;
use crate::errors::*;

#[derive(Accounts)]
#[instruction(vesting_pda_bump: u8)]
pub struct InitVesting<'info> {

    // The constraints on this account ensure that we're dealing with the right treasuries
    // and that the instruction is being called by the correct authority
    #[account(
        mut,
        has_one = fraction_treasury,
        has_one = payment_treasury,
        owner = crate::id()
    )]
    pub presale_account: Box<Account<'info, PresaleInfo>>,

    #[account(mut)]
    pub fraction_treasury: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub payment_treasury: Box<Account<'info, TokenAccount>>,

    #[account(
        init, 
        payer = signer, 
        token::mint = fraction_mint, 
        token::authority = user_vesting_pda
    )]
    pub vesting_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = signer,
        space = 8 + VESTING_INFO_SIZE,
        seeds = [b"vesting".as_ref(), signer.key().as_ref(), presale_account.key().as_ref(), crate::id().as_ref()],
        bump = vesting_pda_bump,
    )]
    pub user_vesting_pda: Box<Account<'info, VestingInfo>>,

    pub fraction_mint: Box<Account<'info, Mint>>,

    pub signer: Signer<'info>,

    pub token_program: Program<'info, Token>,

    pub rent: Sysvar<'info, Rent>,

    pub system_program: Program<'info, System>,

}

pub fn handler(ctx: Context<InitVesting>) -> ProgramResult {
    let user_vesting_pda = &mut ctx.accounts.user_vesting_pda;
    user_vesting_pda.signer = ctx.accounts.signer.key();
    user_vesting_pda.vesting_account = ctx.accounts.vesting_account.key();
    Ok(())
}