use anchor_lang::prelude::*;

declare_id!("Insurance111111111111111111111111111111111111");

#[program]
pub mod insurance {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}