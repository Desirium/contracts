use anchor_lang::prelude::*;

declare_id!("CzY7h2jY3Fq8Rw3rSSi8PZksgbtZzn44aQR6EeYm2bMo");

#[program]
pub mod desirium_contract {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Hello, Solana!");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
