use anchor_lang::prelude::*;

declare_id!("ZB1BxyVhCwFECQoV7bjoun2pMk1yPvz3PGVoKu4d4m5");

#[program]
pub mod adv_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
