use anchor_lang::prelude::*;
pub mod utils;
use crate::utils::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use anchor_lang::solana_program::{program::invoke, system_instruction};
declare_id!("EPEmHGeMkPVUput3iXBcVWuicgrNm3yQnTP4H5j8ioR7");

// const PUBLISH_TOKEN_MINT : Pubkey = pubkey!("8dh63X7teT3h1SLyTSwchfYRRWncer18mFe1X23RsbSr");
const VESTING_DURATION : u64 = 60 * 24 * 3600; // 2 month
#[program]
pub mod presale_contract {
    use std::u64;

    use solana_program::native_token::LAMPORTS_PER_SOL;

    use super::*;

    pub fn setup_presale(ctx: Context<SetupPresale>, start_presale: u64, end_presale: u64, tge_ts: u64, price_sol_per_token: u64) -> Result<()> {
        let presale_event = &mut ctx.accounts.presale_event;
        let authority = &mut ctx.accounts.signer;

        presale_event.authority = authority.key();
        presale_event.token_mint = ctx.accounts.token_mint.key();
        presale_event.tge_ts = tge_ts;
        presale_event.start_presale = start_presale;
        presale_event.end_presale = end_presale;
        presale_event.price_sol_per_token = price_sol_per_token as f64;
        presale_event.bump = ctx.bumps.presale_event;
        presale_event.tge_released_amount = 0.5;
        // presale_event.total_tokens = 0;
        Ok(())
    }

    pub fn provide_token_to_vault(ctx: Context<ProvideTokenToVault>, amount: u64)->Result<()>{
        let presale_event = &mut ctx.accounts.presale_event;
        let authority = &mut ctx.accounts.authority;
        let vault_ata = &mut ctx.accounts.vault_ata;
        let authority_ata = &mut ctx.accounts.authority_ata;
        let token_program = &mut ctx.accounts.token_program;

        presale_event.total_tokens += amount;

        // Transfer token to vault

        // let cpi_accounts = Transfer {
        //     from: authority_ata.to_account_info(),
        //     to: vault_ata.to_account_info(),
        //     authority: authority.to_account_info(),
        // };
        // let cpi_program = token_program.to_account_info();
        // token::transfer(CpiContext::new(cpi_program, cpi_accounts), amount)?;

        token_transfer_user(authority_ata.to_account_info(), authority, vault_ata.to_account_info(), token_program, amount)?;

        Ok(())
    }

    pub fn withdraw_sol_from_vault(ctx: Context<WithdrawSol>) -> Result<()>{
        let presale_event = &mut ctx.accounts.presale_event;
        let authority = &mut ctx.accounts.authority;
        // let amount = presale_event.lamports();
        // let amount = 1

        // Withdraw SOL from vault
        // **vault.to_account_info().lamports.borrow_mut() -= amount;
        // **authority.to_account_info().lamports.borrow_mut() += amount;
        
        let seeds = &[
            b"presale-event",
            presale_event.token_mint.as_ref(),
            &[presale_event.bump],
        ];
        sol_transfer_from_pda(presale_event.to_account_info(), authority.to_account_info(), authority.to_account_info(), &[seeds], 1*LAMPORTS_PER_SOL/2)?;

        Ok(())
    }

    pub fn purchase_token_by_sol(ctx: Context<PurchaseTokenBySol>, amount_bought: u64)-> Result<()>{
        let presale_event = &mut ctx.accounts.presale_event;
        let user = &mut ctx.accounts.user;
        let user_purchase = &mut ctx.accounts.user_purchase;
        // let vault = &mut ctx.accounts.presale_event;

        let _current = Clock::get()?.unix_timestamp as u64;
        require!(_current >= presale_event.start_presale && _current <= presale_event.end_presale, CustomError::InvaliTimePurchase);
        require!(amount_bought <= presale_event.total_tokens, CustomError::InsufficientToken);

        user_purchase.user = user.key();
        user_purchase.total_purchased_token += amount_bought;

        presale_event.total_tokens -= amount_bought;
        
        let amount_sol = (amount_bought as f64 * presale_event.price_sol_per_token) as u64;

        // Transfer SOL to vault 

        // let ix = system_instruction::transfer(
        //     &user.key(),
        //     &presale_event.to_account_info().key(),
        //     amount_sol,
        // );
    
        // invoke(
        //     &ix,
        //     &[
        //         user.to_account_info(),
        //         presale_event.to_account_info(),
        //         ctx.accounts.system_program.to_account_info(),
        //     ],
        // )?;

        sol_transfer_from_user(user, presale_event.to_account_info(), &ctx.accounts.system_program, amount_sol)?;

        Ok(())
    }

    pub fn claim_token(ctx: Context<ClaimToken>)-> Result<()>{
        let presale_event = &mut ctx.accounts.presale_event;
        let user = &mut ctx.accounts.user;
        let user_purchase = &mut ctx.accounts.user_purchase;
        let vault_ata = &mut ctx.accounts.vault_ata;
        let user_ata = &mut ctx.accounts.user_ata;
        let token_program = &ctx.accounts.token_program;
        
        let _current = Clock::get()?.unix_timestamp as u64;
        
        // Claim token 
        require!(_current >= presale_event.tge_ts, CustomError::InvalidTimeClaimToken);
        let passtime_from_tge = _current - presale_event.tge_ts;

        let total_token = user_purchase.total_purchased_token;

        let half = total_token  * presale_event.tge_released_amount as u64; 
        let mut vested = 0;
        if passtime_from_tge >= VESTING_DURATION {
            vested = half;
        }else{
            vested = half * passtime_from_tge / VESTING_DURATION;
        }
        let amount_to_claim = (half + vested).saturating_sub(user_purchase.claimed_tokens);
        user_purchase.claimed_tokens = half + vested;
        
        // Transfer token mint from vault to user 
        let seeds = &[
            b"presale-event",
            presale_event.token_mint.as_ref(),
            &[presale_event.bump],
        ];

        
        // let cpi_accounts = Transfer {
        //     from: vault_ata.to_account_info(),
        //     to: user_ata.to_account_info(),
        //     authority: presale_event.to_account_info(),
        // };
        // let cpi_program = token_program.to_account_info();

        // token::transfer(
        //     CpiContext::new_with_signer(cpi_program, cpi_accounts, &[seeds]),
        //     amount_to_claim,
        // )?;

        token_transfer_with_signer(vault_ata.to_account_info(), presale_event.to_account_info(), user_ata.to_account_info(), token_program, &[seeds], amount_to_claim)?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(token_publish: Pubkey)]
pub struct SetupPresale<'info> {
    #[account(
        init_if_needed,  // Chỗ này để if_need để tiện test, chuẩn thì phải để init 
        payer = signer,
        seeds = [b"presale-event", token_mint.key().as_ref()],
        bump,
        space = 8 + std::mem::size_of::<PresaleEvent>(),
    )]
    pub presale_event : Account<'info, PresaleEvent>,
    pub token_mint : Account<'info, Mint>, // Nhằm tái sử dụng contract cho nhiều presale token
    #[account(mut)]
    pub signer : Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ProvideTokenToVault<'info> {
    #[account(
        mut, 
        seeds = [b"presale-event", presale_event.token_mint.as_ref()], 
        bump,
    )]
    pub presale_event : Account<'info, PresaleEvent>,
    #[account(
        mut,
        associated_token::mint = presale_event.token_mint,
        associated_token::authority = presale_event,
    )]
    pub vault_ata : Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = presale_event.token_mint,
        associated_token::authority = authority,
    )]
    pub authority_ata : Account<'info, TokenAccount>,
    pub token_program : Program<'info, Token>,
    #[account(mut)]
    pub authority : Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawSol<'info> {
    #[account(
        mut, 
        seeds = [b"presale-event", presale_event.token_mint.as_ref()], 
        bump,
    )]
    pub presale_event : Account<'info, PresaleEvent>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PurchaseTokenBySol<'info> {
    #[account(
        mut, 
        seeds = [b"presale-event", presale_event.token_mint.as_ref()],
        bump,
    )]
    pub presale_event : Account<'info, PresaleEvent>,
    #[account(
        init_if_needed,
        payer = user,
        seeds = [b"user-purchase", user.key().as_ref(), presale_event.token_mint.key().as_ref()],
        bump,
        space = 8 + std::mem::size_of::<UserPurchase>() + 32,
    )]
    pub user_purchase : Account<'info, UserPurchase>,
    #[account(mut)]
    pub user : Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimToken<'info> {
    #[account(
        mut,
        seeds = [b"presale-event", presale_event.token_mint.as_ref()],
        bump,
    )]
    pub presale_event : Account<'info, PresaleEvent>,
    #[account(
        mut, 
        seeds = [b"user-purchase", user.key().as_ref(), presale_event.token_mint.as_ref()],
        bump,
    )]
    pub user_purchase : Account<'info, UserPurchase>,
    #[account(
        mut,
        associated_token::mint = presale_event.token_mint,
        associated_token::authority = presale_event,
    )]
    pub vault_ata : Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = presale_event.token_mint,
        associated_token::authority = user,
    )]
    pub user_ata : Account<'info, TokenAccount>,
    pub token_program : Program<'info, Token>,
    #[account(mut)]
    pub user : Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct PresaleEvent {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub vault_ata: Pubkey,
    pub start_presale: u64, // Thời gian mở presale 
    pub end_presale: u64, // Thời gian kết thúc presale
    pub tge_ts: u64, // Thời gian TGE 
    pub tge_released_amount: f64,
    pub total_tokens: u64,
    pub price_sol_per_token: f64, // in lamports or SPL decimals
    pub bump: u8, 
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum TokenType {
    Sol,
    Spl(Pubkey),
}

#[account]
pub struct UserPurchase {
    pub user: Pubkey,
    pub total_purchased_token: u64, // Tổng số token đã mua 
    pub claimed_tokens: u64, // Số lượng đã claim 
}

#[error_code]
pub enum CustomError {
    #[msg("Its not time yet, please wait until TGE !!")]
    InvalidTimeClaimToken,
    #[msg("Its not time to purchase token")]
    InvaliTimePurchase,
    #[msg("Insufficient token to purchase")]
    InsufficientToken,
    #[msg("Invalid token!!")]
    InvalidTokenMint,
}

