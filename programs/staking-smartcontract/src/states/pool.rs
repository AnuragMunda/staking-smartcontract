use anchor_lang::prelude::*;


/// Constants
pub const POOL_SEED: &str = "POOL";

/**
 * Struct for Pool state
 */
#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub admin: Pubkey, // Admin address of the pool
    pub stake_mint: Pubkey, // Address of the staking token
    pub reward_mint: Pubkey, // Address of the reward token
    pub stake_vault: Pubkey, // Address of the vault for storing stake token

    pub reward_rate: u64, // Reward token per second
    pub total_stake: u128, // Total amount staked in the pool
    pub total_shares: u128, // The sum of all shares minted to all stakers, and it represents 100% of the pool.

    pub acc_reward_per_share: u128, // Total accumulated rewards per 1 staked token, stored as a scaled number
    pub last_update_time: i64, // Last timestamp when rewards were calculated

    pub paused: bool, // Is pool paused/unpaused
    pub bump: u8, // Random value to derive this pool pda
}