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
    pub user: Pubkey,
    pub reward_claimed: u64,
}

#[event]
pub struct UnstakeEvent {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub unstaked_amount: u64,
    pub reward_amount: u128,
}

#[event]
pub struct SetPauseEvent {
    pub paused: bool,
}
