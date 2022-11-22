use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, Token, Mint, TokenAccount};
use anchor_lang::solana_program::sysvar::clock::Clock;

use crate::state::*;
use crate::errors::*;

#[derive(Accounts)]
pub struct StartPresale<'info> {

    #[account(
        mut,
        has_one = authority,
        owner = crate::id()
    )]
    pub presale_account: Account<'info, PresaleInfo>,

    pub authority: Signer<'info>

}


pub fn handler(ctx: Context<StartPresale>) -> ProgramResult {
    let presale_account = &mut ctx.accounts.presale_account;

    // Make sure presale is never started more than once
    if presale_account.started {
        return Err(PresaleError::PresaleAlreadyStarted.into());
    }

    presale_account.presale_start = Clock::get()?.unix_timestamp as u64;
    presale_account.started = true;
    Ok(())
}