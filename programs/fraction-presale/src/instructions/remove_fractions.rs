use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, Token, Mint, TokenAccount};

use crate::state::*;
use crate::errors::*;

#[derive(Accounts)]
#[instruction(pda_bump: u8)]
pub struct RemoveFractions<'info> {

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

    #[account(
        mut,
        constraint = to_account.mint == fraction_treasury.mint,
        constraint = to_account.owner == authority.key()
    )]
    pub to_account: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"presale".as_ref(), presale_account.key().as_ref(), crate::id().as_ref()],
        bump = pda_bump
    )]
    pub presale_pda: AccountInfo<'info>,

    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,

}


pub fn handler(ctx: Context<RemoveFractions>, pda_bump: u8, amount: u64) -> ProgramResult {
    if amount == 0 { return Err(PresaleError::AmountIsZero.into()); }

    let fraction_treasury = &ctx.accounts.fraction_treasury;

    // Make sure there are enough tokens to transfer
    if fraction_treasury.amount < amount {
        return Err(PresaleError::NotEnoughTokensInFractionTreasury.into());
    }

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(), 
            Transfer {
                from: ctx.accounts.fraction_treasury.to_account_info(),
                to: ctx.accounts.to_account.to_account_info(),
                authority: ctx.accounts.presale_pda.to_account_info()
            },
            &[&[b"presale".as_ref(), ctx.accounts.presale_account.key().as_ref(), ctx.program_id.as_ref(), &[pda_bump]]]
        ), 
        amount
    )?;

    Ok(())
}