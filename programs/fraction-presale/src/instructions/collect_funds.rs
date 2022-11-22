use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, Token, Mint, TokenAccount};

use crate::state::*;

#[derive(Accounts)]
#[instruction(pda_bump: u8)]
pub struct CollectFunds<'info> {

    #[account(
        has_one = payment_treasury,
        has_one = authority,
        owner = crate::id()
    )]
    pub presale_account: Account<'info, PresaleInfo>,

    #[account(mut)]
    pub payment_treasury: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = to_account.mint == payment_treasury.mint,
        constraint = to_account.owner == authority.key(),
        owner = token::ID,
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


pub fn handler(ctx: Context<CollectFunds>, pda_bump: u8) -> ProgramResult {

    let amount_to_collect = ctx.accounts.payment_treasury.amount;
    if amount_to_collect > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(), 
                Transfer {
                    from: ctx.accounts.payment_treasury.to_account_info(),
                    to: ctx.accounts.to_account.to_account_info(),
                    authority: ctx.accounts.presale_pda.to_account_info()
                },
                &[&[b"presale".as_ref(), ctx.accounts.presale_account.key().as_ref(), ctx.program_id.as_ref(), &[pda_bump]]]
            ), 
            amount_to_collect
        )?;
    }

    Ok(())
}