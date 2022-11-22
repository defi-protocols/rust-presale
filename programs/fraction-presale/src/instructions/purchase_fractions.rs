use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, Token, Mint, TokenAccount};
use anchor_lang::solana_program::sysvar::clock::Clock;

use crate::state::*;
use crate::errors::*;

#[derive(Accounts)]
#[instruction(presale_pda_bump: u8, vesting_pda_bump: u8)]
pub struct PurchaseFractions<'info> {

    #[account(
        mut,
        has_one = fraction_treasury,
        has_one = payment_treasury,
        has_one = access_treasury,
        constraint = presale_account.access_mint == access_account.mint,
        owner = crate::id()
    )]
    pub presale_account: Box<Account<'info, PresaleInfo>> ,

    #[account(mut)]
    pub fraction_treasury: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub payment_treasury: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub access_treasury: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = from_account.mint == payment_treasury.mint,
        constraint = from_account.owner == signer.key(),
    )]
    pub from_account: Box<Account<'info, TokenAccount>>,

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

    #[account(
        seeds = [b"presale".as_ref(), presale_account.key().as_ref(), crate::id().as_ref()],
        bump = presale_pda_bump
    )]
    pub presale_pda: AccountInfo<'info>,

    #[account(mut, constraint = access_account.owner == signer.key())]
    pub access_account: Box<Account<'info, TokenAccount>>,

    pub signer: Signer<'info>,

    pub token_program: Program<'info, Token>,

}

const ACCESS_TOKEN_UNIT: u64 = 1;

pub fn handler(ctx: Context<PurchaseFractions>, presale_pda_bump: u8, amount: u64) -> ProgramResult {
    if amount == 0 { return Err(PresaleError::AmountIsZero.into()); }
    
    let presale_account = &mut ctx.accounts.presale_account;
    let fraction_treasury = &ctx.accounts.fraction_treasury;
    let from_account = &ctx.accounts.from_account;
    let access_account= &ctx.accounts.access_account;

    // Make sure the presale has actually started
    if !presale_account.started {
        return Err(PresaleError::PresaleHasNotStarted.into());
    }

    // Make sure the presale has not ended yet
    let current_timestamp = Clock::get()?.unix_timestamp as u64;
    if current_timestamp >= presale_account.presale_end {
        return Err(PresaleError::PresaleIsFinished.into());
    }

    // Make sure the user has at least 1 access token
    if access_account.amount < ACCESS_TOKEN_UNIT {
        return Err(PresaleError::MissingAccessToken.into());
    }

    if amount > presale_account.max_amount {
        return Err(PresaleError::AmountTooLarge.into());
    }

    // Make sure there are enough fraction tokens still for sale
    if fraction_treasury.amount < amount {
        return Err(PresaleError::NotEnoughTokensInFractionTreasury.into());
    }
    
    // Make sure the user has enough funds to make the purchase
    let payment_amount = match (presale_account.price as u128).checked_mul(amount as u128) {
        Some(val) => match val.checked_div(1e9 as u128) {
            Some(val) => val as u64,
            None => return Err(PresaleError::NumericalOverflowError.into()),
        },
        None => return Err(PresaleError::NumericalOverflowError.into()),
    };

    if from_account.amount < payment_amount {
        return Err(PresaleError::InsufficientFunds.into());
    }

    // User presents their access token
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(), 
            Transfer {
                from: ctx.accounts.access_account.to_account_info(),
                to: ctx.accounts.access_treasury.to_account_info(),
                authority: ctx.accounts.signer.to_account_info()
            }
        ), 
        ACCESS_TOKEN_UNIT
    )?;

    // User makes the payment for the fraction tokens
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(), 
            Transfer {
                from: ctx.accounts.from_account.to_account_info(),
                to: ctx.accounts.payment_treasury.to_account_info(),
                authority: ctx.accounts.signer.to_account_info()
            }
        ), 
        payment_amount
    )?;
    
    // User receives fraction tokens into their vesting account
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(), 
            Transfer {
                from: fraction_treasury.to_account_info(),
                to: ctx.accounts.vesting_account.to_account_info(),
                authority: ctx.accounts.presale_pda.to_account_info()
            },
            &[&[b"presale".as_ref(), presale_account.key().as_ref(), ctx.program_id.as_ref(), &[presale_pda_bump]]]
        ), 
        amount
    )?;

    presale_account.fractions_sold += amount;

    Ok(())
}