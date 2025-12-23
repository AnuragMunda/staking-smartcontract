use anchor_lang::prelude::*;

use crate::states::Pool;
use crate::utils::SetPauseEvent;

/// @dev Pauses the functions of the program
pub fn _set_pause(ctx: Context<SetPause>, paused: bool) -> Result<()> {
    ctx.accounts.pool.paused = paused;
    emit!(SetPauseEvent {
        paused,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct SetPause<'info> {
    pub admin: Signer<'info>,

    #[account(mut, has_one = admin)]
    pub pool: Account<'info, Pool>,
}