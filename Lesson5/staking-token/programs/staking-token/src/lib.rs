use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use anchor_lang::solana_program::{system_instruction};

// Program ID
declare_id!("86KPGwgV3d5KJ4VXkgeiN2FpeQZErRDZqFj946H7LifD");

const STAKING_TOKEN_MINT : Pubkey = pubkey!("8dh63X7teT3h1SLyTSwchfYRRWncer18mFe1X23RsbSr");

#[program]
pub mod staking_token {
    use anchor_lang::{accounts::signer, solana_program::clock};

    use super::*;

    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        let vault_state = &mut ctx.accounts.vault_state;
        vault_state.admin = ctx.accounts.admin.key();
        vault_state.total_stake = 0;
        vault_state.total_reward = 0;
        vault_state.rps = 0.0;
        vault_state.global_index = 0.0;
        vault_state.last_time_updated = Clock::get()?.unix_timestamp as u64;
        vault_state.bump = ctx.bumps.vault_state;
        Ok(())
    }

    pub fn update_rps(ctx: Context<UpdateRPS>) -> Result<()> {
        let vault_state = &mut ctx.accounts.vault_state;
        let vault = &ctx.accounts.vault;
        let signer = &ctx.accounts.admin;
    
        require_keys_eq!(vault_state.admin, signer.key(), CustomError::AuthorizedFail);
    
        // Update global_index < ...  
        let clock = Clock::get()?;
        let _current = clock.unix_timestamp as u64;
        let passtime = _current.saturating_sub(vault_state.last_time_updated);
    
        if vault_state.total_stake > 0 {
            vault_state.global_index += passtime as f64 * vault_state.rps / vault_state.total_stake as f64;
        }
        vault_state.last_time_updated = _current;
        // ... >
    
        // Update total_reward of vault state
        vault_state.total_reward = vault.lamports();
    
        // Phân phối đều phần thưởng trong 1 ngày
        let seconds_per_day: f64 = (24 * 3600) as f64;
        vault_state.rps = (vault_state.total_reward as f64) / (seconds_per_day as f64);
    
        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        let vault_ata = &ctx.accounts.vault_ata;
        let user_ata = &ctx.accounts.user_ata;
        let token_program = &ctx.accounts.token_program;
        let signer = &ctx.accounts.user;

        let stake_account = &mut ctx.accounts.stake_account;
        let vault_state = &mut ctx.accounts.vault_state;

        let cpi_accounts = Transfer {
            from: user_ata.to_account_info(),
            to: vault_ata.to_account_info(),
            authority: signer.to_account_info(),
        };
        let cpi_program = token_program.to_account_info();
        token::transfer(CpiContext::new(cpi_program, cpi_accounts), amount)?;

        // Update reward < ...  
        let clock = Clock::get()?;
        update_reward(vault_state, stake_account, &clock);
        // ... >
        
        stake_account.amount_token += amount;
        stake_account.owner = signer.key();
        stake_account.bump = ctx.bumps.stake_account;
        
        vault_state.total_stake += amount;

        msg!("Pending reward : {}", stake_account.pending_reward);
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
        require!(amount <= stake_account.amount_token, CustomError::InsufficientToken);
        
        // Update reward < ...  
        let clock = Clock::get()?;
        update_reward(vault_state, stake_account, &clock);
        // ... >

        stake_account.amount_token -= amount;
        vault_state.total_stake -= amount;
        
        let cpi_accounts = Transfer {
            from: vault_ata.to_account_info(),
            to: user_ata.to_account_info(),
            authority: vault_state.to_account_info(),
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
        let vault_state = &mut ctx.accounts.vault_state;
        let vault = &ctx.accounts.vault;
        let user = &ctx.accounts.user;
        let system_program = &ctx.accounts.system_program;

        require_keys_eq!(stake_account.owner, user.key(), CustomError::AuthorizedFail);

        // Update reward < ...  
        let clock = Clock::get()?;
        update_reward(vault_state, stake_account, &clock);
        // ... >

        let reward = stake_account.pending_reward as u64;
        stake_account.pending_reward = 0.0;

        **vault.to_account_info().lamports.borrow_mut() -= reward;
        **user.to_account_info().lamports.borrow_mut() += reward;

        // Dùng invoke_signed
        // let seeds = &[b"vault", vault_state.admin.as_ref(), &[vault_state.bump]];

        // anchor_lang::solana_program::program::invoke_signed(
        //     &system_instruction::transfer(
        //         &vault.key(),
        //         &user.key(),
        //         reward,
        //     ),
        //     &[
        //         vault.to_account_info(),
        //         user.to_account_info(),
        //         system_program.to_account_info(),
        //     ],
        //     &[seeds],
        // )?;

        Ok(())
    }
    
}

fn update_reward(
    vault_state: &mut anchor_lang::prelude::Account<'_, VaultState>,
    stake_account: &mut anchor_lang::prelude::Account<'_, StakeAccount>,
    clock: &Clock,
)  {
    let _current = clock.unix_timestamp as u64;
    let passtime = _current.saturating_sub(vault_state.last_time_updated);

    if vault_state.total_stake > 0 {
        vault_state.global_index += passtime as f64 * vault_state.rps / vault_state.total_stake as f64;
    }

    if stake_account.index == 0.0 {
        stake_account.index = vault_state.global_index;
    }

    stake_account.pending_reward += (vault_state.global_index - stake_account.index) * (stake_account.amount_token as f64);
    stake_account.index = vault_state.global_index;

    vault_state.last_time_updated = _current;
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
    #[account(
        mut,
        associated_token::mint = STAKING_TOKEN_MINT,
        associated_token::authority = user,
    )]
    pub user_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = STAKING_TOKEN_MINT,
        associated_token::authority = vault_state,
    )]
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
    // Check Token and owner 
    #[account(
        mut,
        associated_token::mint = STAKING_TOKEN_MINT,
        associated_token::authority = user,
    )]
    pub user_ata: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = STAKING_TOKEN_MINT,
        associated_token::authority = vault_state,
    )]
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

#[derive(Accounts)]
pub struct UpdateRPS<'info> {
    #[account(
        mut,
        seeds = [b"vault", admin.key().as_ref()],
        bump = vault_state.bump,
    )]
    pub vault_state: Account<'info, VaultState>,
    #[account(
        seeds = [b"vault", vault_state.admin.as_ref()],
        bump = vault_state.bump
    )]
    /// CHECK
    pub vault: AccountInfo<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct VaultState {
    pub admin: Pubkey,
    pub total_stake: u64,
    pub total_reward: u64,
    pub rps: f64,
    pub global_index: f64,
    pub last_time_updated: u64,
    pub bump: u8,
}

#[account]
pub struct StakeAccount {
    pub owner: Pubkey,
    pub amount_token: u64,
    pub pending_reward: f64,
    pub index: f64, // last_reward_index 
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
    #[msg("Insufficient token to unstake")]
    InsufficientToken,
}
