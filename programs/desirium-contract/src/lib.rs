use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};
use anchor_spl::associated_token::AssociatedToken;

declare_id!("5zgUq9GxN1xAP32i52eQpexGuKCcGoqWKvfxbZGDEvMb");

#[program]
pub mod wishlist_program {
    use super::*;

    /// Creates a new wishlist entry with a target amount and identifier.
    pub fn create_wishlist(
        ctx: Context<CreateWishlist>,
        identifier: String,
        target_amount: u64,
    ) -> Result<()> {
        let wishlist_account = &mut ctx.accounts.wishlist_account;
        wishlist_account.creator = *ctx.accounts.creator.key;
        wishlist_account.identifier = identifier;
        wishlist_account.target_amount = target_amount;
        wishlist_account.collected_amount = 0;
        wishlist_account.bump = ctx.bumps.wishlist_account;
        Ok(())
    }

    /// Allows a contributor to donate USDT to a specific wishlist.
    pub fn contribute(ctx: Context<Contribute>, amount: u64) -> Result<()> {
        // Transfer USDT from contributor to the wishlist's PDA token account.
        let cpi_accounts = Transfer {
            from: ctx.accounts.contributor_token_account.to_account_info(),
            to: ctx.accounts.wishlist_token_account.to_account_info(),
            authority: ctx.accounts.contributor.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        token::transfer(
            CpiContext::new(cpi_program, cpi_accounts),
            amount,
        )?;

        // Update the collected amount in the wishlist account.
        let wishlist_account = &mut ctx.accounts.wishlist_account;
        wishlist_account.collected_amount += amount;

        Ok(())
    }

    /// Allows the creator to withdraw funds once the target amount is reached.
    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let wishlist_account = &ctx.accounts.wishlist_account;

        // Ensure the target amount has been collected.
        if wishlist_account.collected_amount < wishlist_account.target_amount {
            return Err(ErrorCode::TargetNotReached.into());
        }

        // Transfer USDT from the wishlist's PDA token account to the creator's token account.
        let seeds = &[
            b"wishlist",
            wishlist_account.creator.as_ref(),
            wishlist_account.identifier.as_bytes(),
            &[wishlist_account.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.wishlist_token_account.to_account_info(),
            to: ctx.accounts.creator_token_account.to_account_info(),
            authority: ctx.accounts.wishlist_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        token::transfer(
            CpiContext::new_with_signer(cpi_program, cpi_accounts, signer),
            wishlist_account.collected_amount,
        )?;

        // // Optionally, reset collected amount or mark the wishlist as fulfilled.
        // wishlist_account.collected_amount = 0;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(identifier: String)]
pub struct CreateWishlist<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        init,
        seeds = [b"wishlist", creator.key().as_ref(), identifier.as_bytes()],
        bump,
        payer = creator,
        space = 8 + 32 + 4 + identifier.len() + 8 + 8 + 1,
    )]
    pub wishlist_account: Account<'info, WishlistAccount>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Contribute<'info> {
    #[account(mut)]
    pub contributor: Signer<'info>,

    #[account(
        mut,
        seeds = [b"wishlist", wishlist_account.creator.as_ref(), wishlist_account.identifier.as_bytes()],
        bump = wishlist_account.bump,
    )]
    pub wishlist_account: Account<'info, WishlistAccount>,

    #[account(
        mut,
        constraint = contributor_token_account.owner == contributor.key(),
        constraint = contributor_token_account.mint == usdt_mint.key(),
    )]
    pub contributor_token_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = contributor,
        associated_token::mint = usdt_mint,
        associated_token::authority = wishlist_account,
    )]
    pub wishlist_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub usdt_mint: Account<'info, Mint>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        mut,
        has_one = creator,
        seeds = [b"wishlist", creator.key().as_ref(), wishlist_account.identifier.as_bytes()],
        bump = wishlist_account.bump,
    )]
    pub wishlist_account: Account<'info, WishlistAccount>,

    #[account(
        mut,
        associated_token::mint = usdt_mint,
        associated_token::authority = wishlist_account,
    )]
    pub wishlist_token_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = creator,
        associated_token::mint = usdt_mint,
        associated_token::authority = creator,
    )]
    pub creator_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub usdt_mint: Account<'info, Mint>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[account]
pub struct WishlistAccount {
    pub creator: Pubkey,
    pub identifier: String,
    pub target_amount: u64,
    pub collected_amount: u64,
    pub bump: u8,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Target amount has not been reached yet.")]
    TargetNotReached,
}
