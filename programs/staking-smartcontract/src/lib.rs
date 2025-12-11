use anchor_lang::prelude::*;

pub mod states;
pub mod instructions;

use crate::instructions::*;

declare_id!("7EwcQih3qmU9G95UTmxYbSfoyfvHME6hWLUuCb3Qef2Z");

#[program]
pub mod staking_smartcontract {

    use super::*;

    pub fn initialize_pool(ctx: Context<InitializePool>, reward_rate: u64) -> Result<()> {
        _initialize_pool(ctx, reward_rate)
    }
}
