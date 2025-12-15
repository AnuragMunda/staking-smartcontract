use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenInterface, TokenAccount, TransferChecked};

use crate::states::{Pool, UserStake, USER_STAKE_SEED};
use crate::utils::{StakeEvent, StakingError};


const SCALING_FACTOR: u128 = 1_000_000_000_000u128; // 1e12

/// @dev Function to add stakes into the pool
/// @param `stake_amount` The amount to deposit
pub fn _stake(ctx: Context<Stake>, stake_amount: u64) -> Result<()> {
    require!(!ctx.accounts.pool.paused, StakingError::Paused);

    let now = Clock::get()?.unix_timestamp;

    let user = &ctx.accounts.user;
    let pool = &mut ctx.accounts.pool;
    let user_stake_ata = &ctx.accounts.user_stake_ata;
    let stake_mint = &ctx.accounts.stake_mint;
    let stake_vault = &ctx.accounts.stake_vault;
    let user_stake = &mut ctx.accounts.user_stake;
    
    // Sync rewards before changing balances
    sync_reward_vars(pool, now)?;

    let stake_amount_u128: u128 = stake_amount as u128;

    let shares: u128 = if pool.total_shares == 0 || pool.total_stake == 0 {
        stake_amount_u128
    } else {
        // shares = stake_amount * total_shares / total_stake
        let prod = stake_amount_u128.checked_mul(pool.total_shares).ok_or(StakingError::Overflow)?;
        prod.checked_div(pool.total_stake).ok_or(StakingError::Overflow)?
    };

    // Transfer from user --> stake_vault
    let cpi_accounts = TransferChecked {
        mint: stake_mint.to_account_info(),
        from: user_stake_ata.to_account_info(),
        to: stake_vault.to_account_info(),
        authority: user.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_context = CpiContext::new(cpi_program, cpi_accounts);

    token_interface::transfer_checked(cpi_context, stake_amount, stake_mint.decimals)?;

    // Update pool
    pool.total_stake = pool.total_stake.checked_add(stake_amount_u128).ok_or(StakingError::Overflow)?;
    pool.total_shares = pool.total_shares.checked_add(shares).ok_or(StakingError::Overflow)?;

    // if new account, set owner/pool
    if user_stake.owner == Pubkey::default() {
        user_stake.owner = user.key();
        user_stake.pool = pool.key();
    } else {
        require!(user_stake.owner == user.key(), StakingError::InvalidOwner);
        require!(user_stake.pool == pool.key(), StakingError::InvalidPool);
    }

    // compute pending rewards and update reward_debt
    user_stake.staked_balance = user_stake.staked_balance.checked_add(stake_amount_u128).ok_or(StakingError::Overflow)?;

    // set new reward_debt = user.shares * reward_per_share / SCALING
    let prod = user_stake.shares.checked_mul(pool.acc_reward_per_share).ok_or(StakingError::Overflow)?;
    user_stake.reward_debt = prod.checked_div(SCALING_FACTOR).ok_or(StakingError::Overflow)?;

    emit!(StakeEvent {
        user: user.key(),
        pool: pool.key(),
        stake_amount,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, has_one = stake_vault)]
    pub pool: Account<'info, Pool>,

    #[account(
        mut, 
        constraint = user_stake_ata.mint == pool.stake_mint
    )]
    pub user_stake_ata: InterfaceAccount<'info, TokenAccount>, // user's token account for stake token

    #[account(constraint = stake_mint.key() == pool.stake_mint)]
    pub stake_mint: InterfaceAccount<'info, Mint>,

    #[account(mut, address = pool.stake_vault)]
    pub stake_vault: InterfaceAccount<'info, TokenAccount>,
    
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + UserStake::INIT_SPACE,
        seeds = [USER_STAKE_SEED.as_bytes(), pool.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub user_stake: Account<'info, UserStake>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}


//------------------------------------ Helper Functions ------------------------------------//


/// @dev Syncs the reward variables with respect to the elapsed time since last update
fn sync_reward_vars(pool: &mut Account<Pool>, now: i64) -> Result<()> {
    if now <= pool.last_update_time {
        return Ok(());
    }

    // Calculate the time passed since last update
    let elapsed_time = (now - pool.last_update_time) as u128;
    if elapsed_time == 0 || pool.total_shares == 0 || pool.reward_rate == 0 {
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