use anchor_lang::prelude::*;

#[error_code]
pub enum StakingError {
    #[msg("Math overflow")]
    Overflow,
    #[msg("Invalid owner for the account")]
    InvalidOwner,
    #[msg("Invalid pool for this account")]
    InvalidPool,
    #[msg("Paused")]
    Paused,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Insufficient shares")]
    InsufficientShares,
}