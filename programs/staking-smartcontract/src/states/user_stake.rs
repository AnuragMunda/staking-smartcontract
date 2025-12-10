use anchor_lang::prelude::*;

/**
 * Struct defining a user's stake
 */
#[account]
#[derive(InitSpace)]
pub struct UserStake {
    pub user: Pubkey, // The owner of this stake
    pub pool: Pubkey, // The staking pool address

    pub staked_balance: u128, // Token amount the user has currently staked
    pub reward_debt: u128, // Rewards already accounted for

    pub last_stake_time: i64, // The last time the user changes their stake position
    pub lock_until: i64, // Time period for which the staked is locked

    pub bump: u8, // Random value to derive user stake pda
}