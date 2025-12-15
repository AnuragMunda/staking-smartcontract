use anchor_lang::prelude::*;

#[event]
pub struct StakeEvent {
    pub user: Pubkey,
    pub pool: Pubkey,
    pub stake_amount: u64,
}