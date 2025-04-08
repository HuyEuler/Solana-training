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
        let vault_ata = &ctx.accounts.vault_ata;
        let user_ata = &ctx.accounts.user_ata;
        let token_program = &ctx.accounts.token_program;
        let signer = &ctx.accounts.user;

        let _stake_account = &ctx.accounts.stake_account;
        let _vault_state = &ctx.accounts.vault_state;

        let cpi_accounts = Transfer {
            from: user_ata.to_account_info().clone(),
            to: vault_ata.to_account_info().clone(),
            authority: signer.to_account_info().clone(),
        };
        let cpi_program = token_program.to_account_info();

        token::transfer(
            CpiContext::new(cpi_program, cpi_accounts),
            amount,
        )?;

        // Update stake account
        _stake_account.amount_token += amount;
        _stake_account.owner = user.key();
        _stake_account.bump = ctx.bumps.stake_account;

        // Update total token
        _vault_state.total_stake += amount;
        
        // Update reward user 
        let now = Clock::get()?.unix_timestamp as u64;
        let passtime = now - _vault_state.last_time_updated; // Thời gian trôi qua từ lần cuối 1 user thay đổi total stake
        // Tính lại globalIndex
        _vault_state.global_index += passtime * rps / _vault_state.total_stake;
        // Cập nhật reward 
        _stake_account.pending_reward += (_vault_state.global_index - _stake_account.index) * _stake_account.amount_token;
        _stake_account.index = _vault_state.global_index;
        

        
        // update_reward();

        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()>{
        let vault_ata = &ctx.accounts.vault_ata;
        let user_ata = &ctx.accounts.user_ata;
        let token_program = &ctx.accounts.token_program;
        let signer = &ctx.accounts.user;

        let _stake_account = &ctx.accounts.stake_account;
        let _vault_state = &ctx.accounts.vault_state;

        let cpi_accounts = Transfer {
            from: vault_ata.to_account_info().clone(),
            to: user_ata.to_account_info().clone(),
            authority: signer.to_account_info().clone(),
        };
        let cpi_program = token_program.to_account_info();

        token::transfer(
            CpiContext::new(cpi_program, cpi_accounts),
            amount,
        )?;

        _stake_account.amount_token -= amount;
        _stake_account.pending_reward = 0;

        // Update reward user 
        let now = Clock::get()?.unix_timestamp as u64;
        let passtime = now - _stake_account.last_time_update;

        

        _vault_state.total_stake -= amount;

        Ok(())
    }

    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        let stake_account = &ctx.accounts.stake_account;
        let vault_addr = &ctx.accounts.vault_addr;
        let user = &ctx.accounts.user;

        let reward = stake_account.pending_reward;

        // Update reward
        
        
        // Chuyển tiền từ vault về tài khoản 
        **vault_addr.to_account_info().lamports.borrow_mut() -= reward;
        **user.to_account_info().lamports.borrow_mut() += reward;
        Ok(())
    }

    
    // fn update_reward(vault: &mut Account<'info, VaultState>, stake: &mut Account<'info,StakeAccount>) -> Result<()> {
    //     let now = Clock::get()?.unix_timestamp as u64;
    //     let passtime = now.saturating_sub(vault.last_time_updated);

    //     if vault.total_stake > 0 {
    //         vault.global_index += passtime * vault.rps / vault.total_stake;
    //     }

    //     stake.pending_reward += (vault.global_index.saturating_sub(stake.index)) * stake.amount_token;
    //     stake.index = vault.global_index;

    //     vault.total_stake += amount;
    //     vault.last_time_updated = now;

    //     Ok(())
    // }


}

// DERIVE ACCOUNT FOR VAULT
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
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(
        mut, 
        seeds = [],
        bump,
    )]
    pub vault_state : Account<'info, VaultState>,
    #[account(
        init_if_needed,
        payer = user,
        seeds = [b"data", user.key().as_ref()],
        bump,
        space = 8 + std::mem::size_of::<StakeAccount>() + 50,
    )]
    pub stake_account: Account<'info, StakeAccount>, 
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
pub struct Unstake<'info> {
    #[account(mut)]
    pub vault_state : Account<'info, VaultState>,
    #[account(
        seeds = [b"data", user.key().as_ref()],
        bump,
    )]
    pub stake_account: Account<'info, StakeAccount>, 
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
        bump = stake_account.bump,
    )]
    pub stake_account: Account<'info, StakeAccount>, 
    pub vault_addr : AccountInfo<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// ERROR

#[error_code]
pub enum CustomError {
    #[msg("You don't stake any token")]
    NoStake,
    #[msg("Vault does not have enough lamports")]
    NotEnoughLamports,
    #[msg("Authorization failed")]
    AuthorizedFail,
}

// STRUCT DATA 
// Thông tin các biến toàn cục 
#[account]
pub struct VaultState {
    pub admin: Pubkey,
    pub total_stake : u64,
    pub total_reward : u64,
    pub global_index : u64, // Thêm biến global_index hỗ trợ tính reward 
    pub last_time_updated: u64, // Lần cuối 1 user nào đó làm thay đổi total stakestake
    pub rps : u64,
    pub bump: u8,
}

// Thông tin stake account của user 
#[account]
pub struct StakeAccount {
    pub owner: Pubkey,
    pub amount_token: u64,
    pub index: u64, // Thêm biến index hỗ trợ tính reward 
    pub pending_reward: u64,
    pub bump: u8,
}


