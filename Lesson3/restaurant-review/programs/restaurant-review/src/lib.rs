use anchor_lang::prelude::*;

declare_id!("43JrGEWvW7h6A6Chxbv9XVVSDCqJAxYVGgiRF9RF91jA");

#[program]
pub mod review_program {
    use super::*;

    pub fn submit_review(
        ctx: Context<SubmitReview>,
        restaurant: String,
        review: String,
        rating: u8,
    ) -> Result<()> {
        require!(rating >= 1 && rating <= 5, ReviewError::InvalidRating);
        
        let review_account = &mut ctx.accounts.review;
        review_account.reviewer = *ctx.accounts.reviewer.key;
        review_account.restaurant = restaurant;
        review_account.review = review;
        review_account.rating = rating;
        review_account.bump = ctx.bumps.review;
        Ok(())
    }

    pub fn edit_review(
        ctx: Context<EditReview>,
        new_review: String,
        new_rating: u8,
        new_loc: String
    ) -> Result<()> {
        let review_account = &mut ctx.accounts.review;
        review_account.review = new_review;
        review_account.rating = new_rating;
        review_account.location = new_loc;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct SubmitReview<'info> {
    #[account(
        init_if_needed, 
        payer = reviewer, 
        seeds = [b"review", reviewer.key().as_ref()],
        bump,
        space = 8 + std::mem::size_of::<Review>() + 200 
    )]
    pub review: Account<'info, Review>,
    #[account(mut)]
    pub reviewer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct EditReview<'info> {
    #[account(
        mut,
        seeds = [b"review", reviewer.key().as_ref()],
        bump,
    )]
    pub review: Account<'info, Review>,
    pub reviewer: Signer<'info>, 
}

#[account]
pub struct Review {
    pub reviewer: Pubkey,
    pub restaurant: String,
    pub review: String,
    pub rating: u8,
    pub location: String,
    pub bump: u8
}

#[error_code]
pub enum ReviewError {
    #[msg("Rating must be between 1 and 5.")]
    InvalidRating,
}

