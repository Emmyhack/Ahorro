use anchor_lang::prelude::*;

declare_id!("Ez7nS3RhjdeYknDMJSrunJE1wbACMg7yN4YTgFmkHkQz");

#[program]
pub mod ahorro {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
