use anchor_lang::prelude::*;

use crate::states::Pool;
use crate::utils::{sync_reward_vars, SetRewardEvent};

/// @dev Set the reward per rate value -- ONLY ADMIN
pub fn _set_reward(ctx: Context<SetReward>, reward_rate: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    // Sync the reward state before updating
    let now = Clock::get()?.unix_timestamp;
    sync_reward_vars(pool, now)?;

    pool.reward_rate = reward_rate;

    // emit the event
    emit!(SetRewardEvent {
        pool: pool.key(),
        reward_rate,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct SetReward<'info> {
    pub admin: Signer<'info>,

    #[account(mut, has_one = admin)]
    pub pool: Account<'info, Pool>,
}