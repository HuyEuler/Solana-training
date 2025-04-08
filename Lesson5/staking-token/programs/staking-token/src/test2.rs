use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use anchor_lang::solana_program::{system_instruction};

// Program ID
declare_id!("86KPGwgV3d5KJ4VXkgeiN2FpeQZErRDZqFj946H7LifD");

#[program]
pub mod staking_token {
    use super::*;

    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        let vault_state = &mut ctx.accounts.vault_state;
        vault_state.admin = ctx.accounts.admin.key();
        vault_state.total_stake = 0;
        vault_state.total_reward = 0;
        vault_state.rps = 0;
        vault_state.global_index = 0;
        vault_state.last_time_updated = Clock::get()?.unix_timestamp as u64;
        vault_state.bump = ctx.bumps.vault_state;
        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        let vault_ata = &ctx.accounts.vault_ata;
        let user_ata = &ctx.accounts.user_ata;
        let token_program = &ctx.accounts.token_program;
        let signer = &ctx.accounts.user;

        let stake_account = &mut ctx.accounts.stake_account;
        let vault_state = &mut ctx.accounts.vault_state;

        let now = Clock::get()?.unix_timestamp as u64;

        let cpi_accounts = Transfer {
            from: user_ata.to_account_info(),
            to: vault_ata.to_account_info(),
            authority: signer.to_account_info(),
        };
        let cpi_program = token_program.to_account_info();
        token::transfer(CpiContext::new(cpi_program, cpi_accounts), amount)?;

        update_reward(vault_state, stake_account, now, amount)?;

        stake_account.amount_token += amount;
        stake_account.owner = signer.key();
        stake_account.bump = ctx.bumps.stake_account;

        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        let vault_ata = &ctx.accounts.vault_ata;
        let user_ata = &ctx.accounts.user_ata;
        let token_program = &ctx.accounts.token_program;
        let signer = &ctx.accounts.user;

        let stake_account = &mut ctx.accounts.stake_account;
        let vault_state = &mut ctx.accounts.vault_state;

        require_keys_eq!(stake_account.owner, signer.key(), CustomError::AuthorizedFail);

        let now = Clock::get()?.unix_timestamp as u64;

        update_reward(vault_state, stake_account, now, 0)?;

        stake_account.amount_token = stake_account.amount_token.saturating_sub(amount);
        vault_state.total_stake = vault_state.total_stake.saturating_sub(amount);

        let cpi_accounts = Transfer {
            from: vault_ata.to_account_info(),
            to: user_ata.to_account_info(),
            authority: ctx.accounts.vault_state.to_account_info(),
        };
        let cpi_program = token_program.to_account_info();

        let seeds = &[b"vault", vault_state.admin.as_ref(), &[vault_state.bump]];
        token::transfer(
            CpiContext::new_with_signer(cpi_program, cpi_accounts, &[seeds]),
            amount,
        )?;

        Ok(())
    }

    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        let stake_account = &mut ctx.accounts.stake_account;
        let vault_state = &ctx.accounts.vault_state;
        let vault = &ctx.accounts.vault;
        let user = &ctx.accounts.user;
        let system_program = &ctx.accounts.system_program;

        require_keys_eq!(stake_account.owner, user.key(), CustomError::AuthorizedFail);

        let now = Clock::get()?.unix_timestamp as u64;
        update_reward(&mut ctx.accounts.vault_state, stake_account, now, 0)?;

        let reward = stake_account.pending_reward;
        stake_account.pending_reward = 0;

        let seeds = &[b"vault", vault_state.admin.as_ref(), &[vault_state.bump]];

        invoke_signed(
            &system_instruction::transfer(
                &vault.key(),
                &user.key(),
                reward,
            ),
            &[
                vault.to_account_info(),
                user.to_account_info(),
                system_program.to_account_info(),
            ],
            &[seeds],
        )?;

        Ok(())
    }

    fn update_reward(vault: &mut Account<VaultState>, stake: &mut Account<StakeAccount>, now: u64, amount: u64) -> Result<()> {
        let passtime = now.saturating_sub(vault.last_time_updated);

        if vault.total_stake > 0 {
            vault.global_index += passtime * vault.rps / vault.total_stake;
        }

        stake.pending_reward += (vault.global_index.saturating_sub(stake.index)) * stake.amount_token;
        stake.index = vault.global_index;

        vault.total_stake += amount;
        vault.last_time_updated = now;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(
        init_if_needed,
        payer = admin,
        seeds = [b"vault", admin.key().as_ref()],
        bump,
        space = 8 + std::mem::size_of::<VaultState>(),
    )]
    pub vault_state: Account<'info, VaultState>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault_state.admin.as_ref()],
        bump = vault_state.bump,
    )]
    pub vault_state: Account<'info, VaultState>,
    #[account(
        init_if_needed,
        payer = user,
        seeds = [b"data", user.key().as_ref()],
        bump,
        space = 8 + std::mem::size_of::<StakeAccount>(),
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
    #[account(
        mut,
        seeds = [b"vault", vault_state.admin.as_ref()],
        bump = vault_state.bump,
    )]
    pub vault_state: Account<'info, VaultState>,
    #[account(
        mut,
        seeds = [b"data", user.key().as_ref()],
        bump = stake_account.bump,
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
    #[account(
        mut,
        seeds = [b"vault", vault_state.admin.as_ref()],
        bump = vault_state.bump,
    )]
    pub vault_state: Account<'info, VaultState>,
    /// CHECK: vault PDA, signer
    #[account(
        mut,
        seeds = [b"vault", vault_state.admin.as_ref()],
        bump = vault_state.bump,
    )]
    pub vault: AccountInfo<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct VaultState {
    pub admin: Pubkey,
    pub total_stake: u64,
    pub total_reward: u64,
    pub rps: u64,
    pub global_index: u64,
    pub last_time_updated: u64,
    pub bump: u8,
}

#[account]
pub struct StakeAccount {
    pub owner: Pubkey,
    pub amount_token: u64,
    pub pending_reward: u64,
    pub last_time_update: u64,
    pub index: u64,
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
