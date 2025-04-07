use anchor_lang::prelude::*;

declare_id!("FVqYs1jivsjSBZYYGmyro4hf8hE25YrrTDNCCgZ2NjPc");

#[program]
pub mod my_program2 {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
    pub fn sum(ctx: Context<Sum>, a: u64, b: u64) -> Result<()> {
        msg!("Sum = {:?}", a + b);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct Sum {}
