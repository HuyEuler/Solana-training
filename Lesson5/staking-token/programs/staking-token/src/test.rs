use std::time;

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("86KPGwgV3d5KJ4VXkgeiN2FpeQZErRDZqFj946H7LifD");

#[program]
pub mod staking_token {
    use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;
    use super::*;

    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        // Vault PDA sẽ được cấp lamports từ user
        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        let destination = &ctx.accounts.to_ata;
        let source = &ctx.accounts.from_ata;
        let token_program = &ctx.accounts.token_program;
        let authority = &ctx.accounts.from;

        let cpi_accounts = Transfer {
            from: source.to_account_info().clone(),
            to: destination.to_account_info().clone(),
            authority: authority.to_account_info().clone(),
        };
        let cpi_program = token_program.to_account_info();

        token::transfer(
            CpiContext::new(cpi_program, cpi_accounts),
            amount,
        )?;

        let pda = &mut ctx.accounts.data_account;
        pda.amount_token += amount;
        pda.owner = authority.key();
        pda.bump = ctx.bumps.data_account;
        pda.reward = amount * LAMPORTS_PER_SOL;

        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()>{
        Ok(())
    }

    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        let pda = &mut ctx.accounts.data_account;
        let vault = &ctx.accounts.vault;
        let user = &ctx.accounts.user;
        let reward = pda.reward;

        let vault_lamports = **vault.to_account_info().lamports.borrow();
        require!(pda.amount_token > 0, CustomError::NoStake);
        require!(vault_lamports >= reward, CustomError::NotEnoughLamports);
        require!(user.key() == pda.owner, CustomError::AuthorizedFail);

        **vault.to_account_info().try_borrow_mut_lamports()? -= reward;
        **user.to_account_info().try_borrow_mut_lamports()? += reward;
        // reward = (current time - last time reward) * rps ;
        Ok(())
    }

    pub fn update_reward(ctx: Context<UpdateReward>) -> Result<()> {
        
    }

}

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    /// CHECK: This PDA vault account will hold SOL and verified by seeds
    #[account(
        mut,
        seeds = [b"vault"],
        bump
    )]
    pub vault: AccountInfo<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(
        init_if_needed,
        payer = from,
        seeds = [b"data", from.key().as_ref()],
        bump,
        space = 8 + 8 + 32 + 8 + 1,
    )]
    pub data_account: Account<'info, StakeAccount>, 
    #[account(mut)]
    pub from: Signer<'info>,
    #[account(mut)]
    pub from_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(
        seeds = [b"data", user.key().as_ref()],
        bump,
    )]
    pub data_account: Account<'info, StakeAccount>, 
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(
        mut,
        seeds = [b"data", user.key().as_ref()],
        bump = data_account.bump,
    )]
    pub data_account: Account<'info, Data>, 
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK: This PDA vault account is derived from fixed seeds and bump
    #[account(
        mut,
        seeds = [b"vault"],
        bump
    )]
    pub vault: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct StakeAccount {
    pub owner: Pubkey,
    pub amount_token: u64,
    pub pending_reward: u64,
    pub last_time_update: time,
    pub bump: u8,
}

#[derive(Accounts)]
pub struct InitVault<'info> {
    #[account(
        init_if_needed,
        payer = admin,
        seeds = [b"vault", admin.key().as_ref()],
        bump,
        space = 8 + std::mem::size_of::<VaultState>(),
    )]
    pub vault_state : Account<'info, VaultState>,
    #[account(mut)]
    pub admin: Signer<'info>,

}

#[account]
pub struct VaultState {
    pub total_stake : u64,
    pub total_reward : u64,
    pub rps : u64,
    pub bump: u8,
}

#[error_code]
pub enum CustomError {
    #[msg("You don't stake any token")]
    NoStake,
    #[msg("Vault does not have enough lamports")]
    NotEnoughLamports,
    #[msg("Authorization failed")]
    AuthorizedFail,
}
