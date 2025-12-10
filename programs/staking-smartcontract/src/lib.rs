use anchor_lang::prelude::*;

declare_id!("7EwcQih3qmU9G95UTmxYbSfoyfvHME6hWLUuCb3Qef2Z");

#[program]
pub mod staking_smartcontract {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
