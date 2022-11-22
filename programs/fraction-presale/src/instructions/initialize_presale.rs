use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, Token, Mint, TokenAccount};

use crate::state::*;

#[derive(Accounts)]
#[instruction(pda_bump: u8)]
pub struct InitializePresale<'info> {

    // This accounts contains all the information about the presale
    #[account(
        init,
        payer = authority,
        space = 8 + PRESALE_INFO_SIZE
    )]
    pub presale_account: Box<Account<'info, PresaleInfo>>,

    // This account will hold the access tokens used for purchases
    #[account(
        init, 
        payer = authority, 
        token::mint = access_mint, 
        token::authority = presale_pda
    )]
    pub access_treasury: Box<Account<'info, TokenAccount>>,

    // This account will hold the fraction tokens for sale
    #[account(
        init, 
        payer = authority, 
        token::mint = fraction_mint, 
        token::authority = presale_pda
    )]
    pub fraction_treasury: Box<Account<'info, TokenAccount>>,

    // Funds used for purchasing the fraction tokens will be held here
    #[account(
        init, 
        payer = authority, 
        token::mint = payment_mint, 
        token::authority = presale_pda
    )]
    pub payment_treasury: Box<Account<'info, TokenAccount>>,

    // Authority PDA for `fraction_treasury` and `payment_treasury`
    #[account(
        seeds = [b"presale".as_ref(), presale_account.key().as_ref(), crate::id().as_ref()],
        bump = pda_bump,
    )]
    pub presale_pda: AccountInfo<'info>,

    #[account(constraint = fraction_mint.key() != payment_mint.key())]
    pub fraction_mint: Box<Account<'info, Mint>>,

    pub payment_mint: Box<Account<'info, Mint>>,

    pub access_mint: Box<Account<'info, Mint>>,

    // Authority for the presale account
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,

    pub rent: Sysvar<'info, Rent>,

    pub system_program: Program<'info, System>,

}

pub fn handler(
    ctx: Context<InitializePresale>,
    price: u64,
    max_amount: u64,
    presale_end: u64,
    vesting_end: u64
) -> ProgramResult {

    let presale_account = &mut ctx.accounts.presale_account;

    // Set public keys for the important accounts this presale account works with
    // to make sure that people don't pass in the wrong accounts for future instructions
    presale_account.fraction_treasury = ctx.accounts.fraction_treasury.key();
    presale_account.payment_treasury = ctx.accounts.payment_treasury.key();
    presale_account.access_treasury = ctx.accounts.access_treasury.key();
    presale_account.authority = ctx.accounts.authority.key();

    // Set presale params
    presale_account.presale_end = presale_end;
    presale_account.vesting_end = vesting_end;
    presale_account.fractions_sold = 0;
    presale_account.fraction_mint = ctx.accounts.fraction_mint.key();
    presale_account.access_mint = ctx.accounts.access_mint.key();
    presale_account.payment_mint = ctx.accounts.payment_mint.key();
    presale_account.price = price;
    presale_account.max_amount = max_amount;

    Ok(())
}