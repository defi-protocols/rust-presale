use anchor_lang::prelude::*;

pub const VESTING_INFO_SIZE: usize = 32 + 32;
pub const PRESALE_INFO_SIZE: usize = 32 + 32 + 32 + 32 + 8 + 32 + 32 + 32 + 8 + 8 + 8 + 1 + 8 + 8;

#[account]
pub struct VestingInfo {

    pub signer: Pubkey,

    pub vesting_account: Pubkey,

}

#[account]
pub struct PresaleInfo {

    pub fraction_treasury: Pubkey,

    pub payment_treasury: Pubkey,

    pub access_treasury: Pubkey,

    pub authority: Pubkey,

    pub fractions_sold: u64,

    pub fraction_mint: Pubkey,

    pub payment_mint: Pubkey,

    pub access_mint: Pubkey,

    pub price: u64,

    pub max_amount: u64,

    pub presale_start: u64, // signifies when the presale started

    pub started: bool,

    pub presale_end: u64, // amount of time before purchases can no longer be made

    pub vesting_end: u64 // amount of time for vesting before unlock

}