use crate::*;
use anchor_spl::token::{self, Token};
use anchor_lang::solana_program::{program::invoke, program::invoke_signed,system_instruction};
// use solana_program::program::invoke;

//  transfer usdc from user
pub fn sol_transfer_from_user<'info>(
    signer: &Signer<'info>,
    destination: AccountInfo<'info>,
    system_program: &Program<'info, System>,
    amount: u64,
) -> Result<()> {
    let ix = system_instruction::transfer(
        &signer.key(),
        &destination.key(),
        amount,
    );

    invoke(
        &ix,
        &[
            signer.to_account_info(),
            destination,
            system_program.to_account_info(),
        ],
    )?;
    Ok(())
}

// // transfer usdc from PDA
pub fn sol_transfer_from_pda<'info>(
    from: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    to: AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    amount: u64,
) -> Result<()> {
    let ix = system_instruction::transfer(
        &from.key(), 
        &to.key(),       
        amount,
    );

    // Gọi CPI với chữ ký của PDA
    invoke_signed(
        &ix,
        &[
            authority.clone(), // account của PDA
            to.clone(),        // account nhận
            from.clone(),      // account chứa lamports, thường là cùng với authority
        ],
        signer_seeds,
    )?;

    Ok(())
}

//  transfer token from PDA
pub fn token_transfer_with_signer<'info>(
    from: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    to: AccountInfo<'info>,
    token_program: &Program<'info, Token>,
    signer_seeds: &[&[&[u8]]],
    amount: u64,
) -> Result<()> {
    let cpi_ctx: CpiContext<_> = CpiContext::new_with_signer(
        token_program.to_account_info(),
        token::Transfer {
            from,
            to,
            authority,
        },
        signer_seeds,
    );
    token::transfer(cpi_ctx, amount)?;

    Ok(())
}

//  transfer token from user
pub fn token_transfer_user<'info>(
    from: AccountInfo<'info>,
    authority: &Signer<'info>,
    to: AccountInfo<'info>,
    token_program: &Program<'info, Token>,
    amount: u64,
) -> Result<()> {
    let cpi_ctx: CpiContext<_> = CpiContext::new(
        token_program.to_account_info(),
        token::Transfer {
            from,
            authority: authority.to_account_info(),
            to,
        },
    );
    token::transfer(cpi_ctx, amount)?;

    Ok(())
}
