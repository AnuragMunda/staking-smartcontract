use anchor_lang::prelude::*;


/// Constants
pub const USER_STAKE_SEED: &str = "USER_STAKE";

/**
 * Struct defining a user's stake
 */
#[account]
#[derive(InitSpace)]
pub struct UserStake {
    pub owner: Pubkey, // The owner of this stake
    pub pool: Pubkey, // The staking pool address
    
    pub shares: u128, // User shares
    pub reward_debt: u128, // Rewards already accounted for

    pub last_stake_time: i64, // The last time the user changes their stake position

    pub bump: u8, // Random value to derive user stake pda
}