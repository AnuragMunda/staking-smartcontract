use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::states::pool::*;
use crate::utils::{InitializePoolEvent, StakingError};

/// @notice Instruction to initialize the pool
/// @params reward_rate Reward per second
pub fn _initialize_pool(
    ctx: Context<InitializePool>,
    reward_rate: u64,
) -> Result<()> {
    require!(reward_rate > 0u64, StakingError::InvalidAmount);

    let pool = &mut ctx.accounts.pool;

    pool.admin = ctx.accounts.admin.key();
    pool.stake_mint = ctx.accounts.stake_mint.key();
    pool.reward_mint = ctx.accounts.reward_mint.key();
    pool.stake_vault = ctx.accounts.stake_vault.key();
    pool.reward_rate = reward_rate;
    pool.total_stake = 0u128;
    pool.total_shares = 0u128;
    pool.acc_reward_per_share = 0u128;
    pool.last_update_time = Clock::get()?.unix_timestamp;
    pool.paused = false;
    pool.bump = ctx.bumps.pool;

    emit!(InitializePoolEvent {
        pool: pool.key(),
        admin: ctx.accounts.admin.key(),
        reward_rate,
    });

    Ok(())
}


//------------------------------------ ACCOUNTS ------------------------------------//

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + Pool::INIT_SPACE,
        seeds = [POOL_SEED.as_bytes(), stake_mint.key().as_ref()],
        bump
    )]
    pub pool: Account<'info, Pool>,

    /// CHECK: stake mint - validated
    pub stake_mint: UncheckedAccount<'info>,
    
    #[account(
        init,
        payer = admin,
        mint::decimals = 9,
        mint::authority = pool.key(),
        mint::freeze_authority = admin.key(),
    )]
    pub reward_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = admin,
        token::mint = stake_mint,
        token::authority = pool,
        seeds = [b"stake_vault", pool.key().as_ref()],
        bump
    )]
    pub stake_vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}