use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, MintTo, TokenAccount, TokenInterface, TransferChecked};

use crate::states::{POOL_SEED, Pool, UserStake};
use crate::utils::{SCALING_FACTOR, StakingError, sync_reward_vars, user_pending_reward, UnstakeEvent};

/// @dev Function to unstake / withdraw the staked tokens
pub fn _unstake(ctx: Context<Unstake>, shares: u128) -> Result<()> {
    require!(!ctx.accounts.pool.paused, StakingError::Paused);

    let now = Clock::get()?.unix_timestamp;
    let pool = &mut ctx.accounts.pool;
    let user_stake = &mut ctx.accounts.user_stake;
    let user_stake_ata = &ctx.accounts.user_stake_ata;
    let user_reward_ata = &ctx.accounts.user_reward_ata;
    let stake_mint = &ctx.accounts.stake_mint;
    let reward_mint = &ctx.accounts.reward_mint;
    let stake_vault = &ctx.accounts.stake_vault;

    require!(shares > 0, StakingError::InvalidAmount);
    require!(user_stake.shares >= shares, StakingError::InsufficientShares);

    // Sync the reward states
    sync_reward_vars(pool, now)?;

    // Check if there are pending rewards, if yes -- then send it to user
    let pending_reward = user_pending_reward(user_stake, pool)?;

    if pending_reward > 0u128 {
        let pending_reward_u64 = pending_reward.try_into().map_err(|_| StakingError::Overflow)?;
        // Seeds that will be used for signing the transaction
        let binding = ctx.accounts.stake_mint.key();
        let signer_seeds: &[&[&[u8]]] = &[&[POOL_SEED.as_bytes(), binding.as_ref(), &[ctx.bumps.pool]]];

        // Prepare and call the mint function
        let cpi_accounts = MintTo {
            mint: reward_mint.to_account_info(),
            to: user_reward_ata.to_account_info(),
            authority: pool.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();

        let cpi_context = CpiContext::new(cpi_program, cpi_accounts).with_signer(signer_seeds);
        token_interface::mint_to(cpi_context, pending_reward_u64)?;
    }

    // Compute amount of stake tokens to return --> shares * total_staked / total_shares
    let prod = shares.checked_mul(pool.total_stake).ok_or(StakingError::Overflow)?;
    let amount_u128 = prod.checked_div(pool.total_shares).ok_or(StakingError::Overflow)?;
    let amount_u64 = amount_u128.try_into().map_err(|_| StakingError::Overflow)?;

    let binding = pool.key();
    let stake_vault_seeds: &[&[&[u8]]] = &[&[b"stake_vault", binding.as_ref(), &[ctx.bumps.stake_vault]]];

    // Prepare and transfer the unstaked shares
    let cpi_transfer_accounts = TransferChecked {
        from: stake_vault.to_account_info(),
        to: user_stake_ata.to_account_info(),
        mint: stake_mint.to_account_info(),
        authority: pool.to_account_info(),
    };

    let cpi_transfer_program = ctx.accounts.token_program.to_account_info();

    let cpi_transfer_context = CpiContext::new(cpi_transfer_program, cpi_transfer_accounts).with_signer(stake_vault_seeds);

    token_interface::transfer_checked(cpi_transfer_context, amount_u64, stake_mint.decimals)?;

    // Update states
    pool.total_stake = pool.total_stake.checked_sub(amount_u128).ok_or(StakingError::Overflow)?;
    pool.total_shares = pool.total_shares.checked_sub(shares).ok_or(StakingError::Overflow)?;

    user_stake.shares = user_stake.shares.checked_sub(shares).ok_or(StakingError::Overflow)?;

    let debt_prod = user_stake.shares.checked_mul(pool.acc_reward_per_share).ok_or(StakingError::Overflow)?;
    user_stake.reward_debt = debt_prod.checked_div(SCALING_FACTOR).ok_or(StakingError::Overflow)?;

    emit!(UnstakeEvent {
        pool: pool.key(),
        user: ctx.accounts.user.key(),
        unstaked_amount: amount_u64,
        reward_amount: pending_reward,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(mut)]
    pub user_stake: Account<'info, UserStake>,

    #[account(
        mut,
        seeds = [POOL_SEED.as_bytes(), stake_mint.key().as_ref()],
        bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(mut, constraint = stake_mint.key() == pool.stake_mint)]
    pub stake_mint: InterfaceAccount<'info, Mint>,

    #[account(mut, constraint = reward_mint.key() == pool.reward_mint)]
    pub reward_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [b"stake_vault", pool.key().as_ref()],
        bump,
    )]
    pub stake_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub user_stake_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub user_reward_ata: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}