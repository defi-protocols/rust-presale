use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, Token, Mint, TokenAccount};

use crate::state::*;
use crate::errors::*;

#[derive(Accounts)]
#[instruction(vesting_pda_bump: u8)]
pub struct UnlockFractions<'info> {

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
        mut,
        constraint = to_account.mint == fraction_treasury.mint,
        constraint = to_account.owner == signer.key(),
    )]
    pub to_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = vesting_account.mint == fraction_treasury.mint,
        constraint = vesting_account.owner == user_vesting_pda.key(),
    )]
    pub vesting_account: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"vesting".as_ref(), signer.key().as_ref(), presale_account.key().as_ref(), crate::id().as_ref()],
        bump = vesting_pda_bump,
        has_one = vesting_account,
        has_one = signer,
        owner = crate::id()
    )]
    pub user_vesting_pda: Box<Account<'info, VestingInfo>>,

    pub signer: Signer<'info>,

    pub token_program: Program<'info, Token>,

}


pub fn handler(ctx: Context<UnlockFractions>, vesting_pda_bump: u8) -> ProgramResult {

    let presale_account = &mut ctx.accounts.presale_account;
    let vesting_account = &ctx.accounts.vesting_account;

    // Make sure there are enough tokens to transfer
    if vesting_account.amount == 0 {
        return Err(PresaleError::VestingAccountIsEmpty.into());
    }
    
    let current_timestamp = Clock::get()?.unix_timestamp as u64;
    if current_timestamp < presale_account.vesting_end {
        return Err(PresaleError::VestingPeriodNotFinished.into());
    }

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(), 
            Transfer {
                from: vesting_account.to_account_info(),
                to: ctx.accounts.to_account.to_account_info(),
                authority: ctx.accounts.user_vesting_pda.to_account_info()
            },
            &[&[b"vesting".as_ref(), ctx.accounts.signer.key().as_ref(), ctx.accounts.presale_account.key().as_ref(), ctx.program_id.as_ref(), &[vesting_pda_bump]]]
        ), 
        vesting_account.amount
    )?;

    Ok(())
}