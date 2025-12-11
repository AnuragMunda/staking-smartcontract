use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::states::pool::*;

/// @notice Instruction to initialize the pool
/// @params reward_rate Reward per second
pub fn _initialize_pool(
    ctx: Context<InitializePool>,
    reward_rate: u64,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    pool.admin = ctx.accounts.admin.key();
    pool.stake_mint = ctx.accounts.stake_mint.key();
    pool.reward_mint = ctx.accounts.reward_mint.key();
    pool.stake_vault = ctx.accounts.stake_vault.key();
    pool.reward_vault = ctx.accounts.reward_vault.key();
    pool.reward_rate = reward_rate;
    pool.total_stake = 0u128;
    pool.total_shares = 0u128;
    pool.acc_reward_per_share = 0u128;
    pool.last_update_time = Clock::get()?.unix_timestamp;
    pool.paused = false;
    pool.bump = ctx.bumps.pool;

    Ok(())
}

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
    /// CHECK: reward mint - validated
    pub reward_mint: UncheckedAccount<'info>,

    #[account(
        init,
        payer = admin,
        token::mint = stake_mint,
        token::authority = pool,
        seeds = [b"stake_vault", pool.key().as_ref()],
        bump
    )]
    pub stake_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = admin,
        token::mint = reward_mint,
        token::authority = pool,
        seeds = [b"reward_vault", pool.key().as_ref()],
        bump
    )]
    pub reward_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}