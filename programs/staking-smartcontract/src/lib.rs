use anchor_lang::prelude::*;

pub mod states;
pub mod instructions;
pub mod utils;

use crate::instructions::*;

declare_id!("7EwcQih3qmU9G95UTmxYbSfoyfvHME6hWLUuCb3Qef2Z");

#[program]
pub mod staking_smartcontract {

    use super::*;

    pub fn initialize_pool(ctx: Context<InitializePool>, reward_rate: u64) -> Result<()> {
        _initialize_pool(ctx, reward_rate)
    }

    pub fn stake(ctx: Context<Stake>, stake_amount: u64) -> Result<()> {
        _stake(ctx, stake_amount)
    }

    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        _claim_reward(ctx)
    }

    pub fn unstake(ctx: Context<Unstake>, shares: u128) -> Result<()> {
        _unstake(ctx, shares)
    }

    pub fn set_pause(ctx: Context<SetPause>, paused: bool) -> Result<()> {
        _set_pause(ctx, paused)
    }
}
