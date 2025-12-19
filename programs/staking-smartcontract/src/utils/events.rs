use anchor_lang::prelude::*;


#[event]
pub struct InitializePoolEvent {
    pub pool: Pubkey,
    pub admin: Pubkey,
    pub reward_rate: u64,
}

#[event]
pub struct StakeEvent {
    pub user: Pubkey,
    pub pool: Pubkey,
    pub stake_amount: u64,
}

#[event]
pub struct ClaimRewardEvent {
    pub pool: Pubkey,
    pub user_ata: Pubkey,
    pub reward_claimed: u64,
}
