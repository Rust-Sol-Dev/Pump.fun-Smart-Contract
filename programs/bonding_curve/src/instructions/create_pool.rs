use crate::{errors::CustomError, state::*};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

pub fn create_pool(ctx: Context<CreateLiquidityPool>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    pool.set_inner(LiquidityPool::new(
        ctx.accounts.mint_token_one.key(),
        ctx.accounts.mint_token_two.key(),
        ctx.bumps.pool,
    ));

    Ok(())
}

#[derive(Accounts)]
pub struct CreateLiquidityPool<'info> {
    #[account(
        init,
        space = LiquidityPool::ACCOUNT_SIZE,
        payer = payer,
        seeds = [LiquidityPool::POOL_SEED_PREFIX.as_bytes(), mint_token_one.key().as_ref(), mint_token_two.key().as_ref()],
        bump
    )]
    pub pool: Box<Account<'info, LiquidityPool>>,

    #[account(
        constraint = !mint_token_one.key().eq(&mint_token_two.key()) @ CustomError::DuplicateTokenNotAllowed
    )]
    pub mint_token_one: Box<Account<'info, Mint>>,

    #[account(
        constraint = !mint_token_one.key().eq(&mint_token_two.key()) @ CustomError::DuplicateTokenNotAllowed
    )]
    pub mint_token_two: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = mint_token_one,
        associated_token::authority = pool
    )]
    pub pool_token_account_one: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = mint_token_two,
        associated_token::authority = pool
    )]
    pub pool_token_account_two: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}
