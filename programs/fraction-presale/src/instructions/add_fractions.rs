use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, Token, Mint, TokenAccount};

use crate::state::*;
use crate::errors::*;

#[derive(Accounts)]
#[instruction(pda_bump: u8)]
pub struct AddFractions<'info> {

    // The constraints on this account ensure that we're dealing with the right treasuries
    // and that the instruction is being called by the correct authority
    #[account(
        mut,
        has_one = fraction_treasury,
        has_one = payment_treasury,
        has_one = authority,
        owner = crate::id()
    )]
    pub presale_account: Account<'info, PresaleInfo>,

    #[account(mut)]
    pub fraction_treasury: Account<'info, TokenAccount>,

    #[account(mut)]
    pub payment_treasury: Account<'info, TokenAccount>,

    // This is the token account whose fraction tokens will be transferred into the `fraction_treasury`
    #[account(
        mut,
        constraint = from_account.mint == fraction_treasury.mint,
        constraint = from_account.owner == authority.key()
    )]
    pub from_account: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"presale".as_ref(), presale_account.key().as_ref(), crate::id().as_ref()],
        bump = pda_bump,
    )]
    pub presale_pda: AccountInfo<'info>,

    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,

}


pub fn handler(ctx: Context<AddFractions>, amount: u64) -> ProgramResult {
    if amount == 0 { return Err(PresaleError::AmountIsZero.into()); }

    let from_account = &ctx.accounts.from_account;

    // Make sure there are enough tokens to transfer
    if from_account.amount < amount {
        return Err(PresaleError::NotEnoughTokensInAccount.into());
    }

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(), 
            Transfer {
                from: ctx.accounts.from_account.to_account_info(),
                to: ctx.accounts.fraction_treasury.to_account_info(),
                authority: ctx.accounts.authority.to_account_info()
            }
        ), 
        amount
    )?;

    Ok(())
}