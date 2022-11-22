use anchor_lang::prelude::*;
use anchor_lang::error;

#[error]
pub enum PresaleError {

    #[msg("The user doesn't have enough tokens in their account")]
    NotEnoughTokensInAccount,

    #[msg("There are not enough tokens in the fraction treasury to satisfy the request")]
    NotEnoughTokensInFractionTreasury,

    #[msg("The presale cannot be started more than once")]
    PresaleAlreadyStarted,

    #[msg("Amount must be greater than zero")]
    AmountIsZero,

    #[msg("The presale has not yet started")]
    PresaleHasNotStarted,

    #[msg("The presale has already finished")]
    PresaleIsFinished,

    #[msg("The user is missing a valid access token")]
    MissingAccessToken,

    #[msg("The user requested to purchase an amount greater than the maximum allowed")]
    AmountTooLarge,

    #[msg("The user doesn't have enough funds in their account to make the purchase")]
    InsufficientFunds,

    #[msg("The vesting period has not finished")]
    VestingPeriodNotFinished,

    #[msg("There are no tokens vested in the account")]
    VestingAccountIsEmpty,

    #[msg("Numerical Overflow Error")]
    NumericalOverflowError,

}