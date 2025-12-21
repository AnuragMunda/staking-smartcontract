use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{self, Mint, MintTo, TokenAccount, TokenInterface};

use crate::states::{POOL_SEED, Pool, UserStake};
use crate::utils::{SCALING_FACTOR, StakingError, sync_reward_vars, user_pending_reward, ClaimRewardEvent};


/// @dev Function to claim pending rewards
pub fn _claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
    require!(!ctx.accounts.pool.paused, StakingError::Paused);

    let now = Clock::get()?.unix_timestamp;
    let user_ata = &ctx.accounts.user_reward_ata;
    let pool = &mut ctx.accounts.pool;
    let user_stake = &mut ctx.accounts.user_stake;
    let reward_mint = &ctx.accounts.reward_mint;

    // Sync the reward states
    sync_reward_vars(pool, now)?;

    // Calculate the reward pending to be claimed
    let pending_reward = user_pending_reward(&user_stake, &pool)?;
    if pending_reward == 0u128 {
        return Ok(());
    }

    // Converting to use for minting
    let pending_reward_u64: u64 = pending_reward.try_into().map_err(|_| StakingError::Overflow)?;

    // Seeds that will be used for signing the transaction
    let binding = ctx.accounts.stake_mint.key();
    let signer_seeds: &[&[&[u8]]] = &[&[POOL_SEED.as_bytes(), binding.as_ref(), &[ctx.bumps.pool]]];

    // Prepare and call the mint function
    let cpi_accounts = MintTo {
        mint: reward_mint.to_account_info(),
        to: user_ata.to_account_info(),
        authority: pool.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();

    let cpi_context = CpiContext::new(cpi_program, cpi_accounts).with_signer(signer_seeds);
    token_interface::mint_to(cpi_context, pending_reward_u64)?;

    // Update the reward debt
    let prod = user_stake.shares.checked_mul(pool.acc_reward_per_share).ok_or(StakingError::Overflow)?;
    let new_reward_debt = prod.checked_div(SCALING_FACTOR).ok_or(StakingError::Overflow)?;
    user_stake.reward_debt = new_reward_debt;

    emit!(ClaimRewardEvent {
        pool: pool.key(),
        user: ctx.accounts.user.key(),
        reward_claimed: pending_reward_u64,
    });
    
    Ok(())
}

//------------------------------------ ACCOUNTS ------------------------------------//

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [POOL_SEED.as_bytes(), stake_mint.key().as_ref()],
        bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(mut)]
    pub user_stake: Account<'info, UserStake>,

    /// CHECK: stake mint
    pub stake_mint: UncheckedAccount<'info>,

    #[account(mut)]
    pub reward_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [b"stake_vault", pool.key().as_ref()],
        bump,
    )]
    pub stake_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = reward_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_reward_ata: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}