use anchor_lang::prelude::*;

use crate::states::{Pool, UserStake};
use crate::utils::StakingError;

//------------------------------------ Helper Functions / Utils ------------------------------------//

pub const SCALING_FACTOR: u128 = 1_000_000_000_000u128; // 1e12

/// @dev Syncs the reward variables with respect to the elapsed time since last update
pub fn sync_reward_vars(pool: &mut Account<Pool>, now: i64) -> Result<()> {
    if now <= pool.last_update_time {
        return Ok(());
    }

    // Calculate the time passed since last update
    let elapsed_time = (now - pool.last_update_time) as u128;
    if pool.total_shares == 0 || pool.reward_rate == 0 {
        pool.last_update_time = now;
        return Ok(());
    }

    // Calculate new rewards for the elapsed time
    let new_rewards = (pool.reward_rate as u128)
        .checked_mul(elapsed_time)
        .ok_or(StakingError::Overflow)?;

    // Calculate the reward for one share and then add it increment the accumulated reward per share value
    // reward_per_share += new_rewards * SCALING_FACTOR / total_shares
    let prod = new_rewards.checked_mul(SCALING_FACTOR).ok_or(StakingError::Overflow)?;
    let increment = prod.checked_div(pool.total_shares).ok_or(StakingError::Overflow)?;

    pool.acc_reward_per_share = pool.acc_reward_per_share.checked_add(increment).ok_or(StakingError::Overflow)?;
    pool.last_update_time = now;

    Ok(())
}

/// @dev Calculates the pending reward to be claimed by a user
pub fn user_pending_reward(user_stake: &Account<UserStake>, pool: &Account<Pool>) -> Result<u128> {
    if user_stake.shares == 0 {
        return Ok(0u128);
    }

    let prod = user_stake.shares.checked_mul(pool.acc_reward_per_share).ok_or(StakingError::Overflow)?;
    let acc_reward = prod.checked_div(SCALING_FACTOR).ok_or(StakingError::Overflow)?;

    if acc_reward <= user_stake.reward_debt {
        return Ok(0u128);
    }

    Ok(acc_reward.checked_sub(user_stake.reward_debt).ok_or(StakingError::Overflow)?)
}