use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction};

declare_id!("FtMb8uteFxBR5E2kdrPRHvZu3zZDeuQ7M3DUoprLEQhQ");

#[program]
pub mod bank_app {
    use super::*;

    pub fn initialize_account(ctx: Context<InitializeAccount>) -> Result<()> {
        let bank_account = &mut ctx.accounts.bank_account;
        bank_account.owner = ctx.accounts.user.key();
        bank_account.balance = 0;
        bank_account.bump = ctx.bumps.bank_account;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let bank_account = &mut ctx.accounts.bank_account;
        let user = &ctx.accounts.user;

        require!(user.lamports() >= amount, BankError::InsufficientFunds);
        
        let ix = system_instruction::transfer(
            &user.key(),
            &bank_account.to_account_info().key(),
            amount,
        );
    
        invoke(
            &ix,
            &[
                user.to_account_info(),
                bank_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        bank_account.balance += amount;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let bank_account = &mut ctx.accounts.bank_account;
        let user = &ctx.accounts.user;

        require!(bank_account.balance >= amount, BankError::InsufficientFunds);

        **bank_account.to_account_info().lamports.borrow_mut() -= amount;
        **user.to_account_info().lamports.borrow_mut() += amount;

        bank_account.balance -= amount;
        Ok(())
    }

    pub fn get_balance(ctx: Context<GetBalance>) -> Result<u64> {
        let bank_account = &ctx.accounts.bank_account;
        Ok(bank_account.balance)
    }
}

#[derive(Accounts)]
pub struct InitializeAccount<'info> {
    #[account(
        init, 
        payer = user, 
        space = 8 + std::mem::size_of::<BankAccount>(),
        seeds = [b"bank_acc", user.key().as_ref()],
        bump,
    )]
    pub bank_account: Account<'info, BankAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        seeds = [b"bank_acc", user.key().as_ref()],
        bump = bank_account.bump,
    )]
    pub bank_account: Account<'info, BankAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [b"bank_acc", user.key().as_ref()],
        bump = bank_account.bump,
    )]
    pub bank_account: Account<'info, BankAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GetBalance<'info> {
    pub bank_account: Account<'info, BankAccount>,
}

#[account]
pub struct BankAccount {
    pub owner: Pubkey,
    pub balance: u64,
    pub bump: u8,
}

#[error_code]
pub enum BankError {
    #[msg("Insufficient funds for this operation.")]
    InsufficientFunds,
}

// pub enum AuthorizedError {
//     #[msg("Authorized fail")]
//     Unauthorized,
// }