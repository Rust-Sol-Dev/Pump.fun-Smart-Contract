use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::{
    errors::CustomError,
    state::{LiquidityPool, LiquidityPoolAccount, LiquidityProvider},
};

pub fn add_liquidity(ctx: Context<AddLiquidity>, amount_one: u64, amount_two: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    let token_one_accounts = (
        &mut *ctx.accounts.mint_token_one,
        &mut *ctx.accounts.pool_token_account_one,
        &mut *ctx.accounts.user_token_account_one,
    );

    let token_two_accounts = (
        &mut *ctx.accounts.mint_token_two,
        &mut *ctx.accounts.pool_token_account_two,
        &mut *ctx.accounts.user_token_account_two,
    );

    pool.add_liquidity(
        token_one_accounts,
        token_two_accounts,
        amount_one,
        amount_two,
        &mut *ctx.accounts.liquidity_provider_account,
        &ctx.accounts.user,
        &ctx.accounts.token_program,
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(
        mut,
        seeds = [LiquidityPool::POOL_SEED_PREFIX.as_bytes(), LiquidityPool::generate_seed(mint_token_one.key(), mint_token_two.key()).as_bytes()],
        bump = pool.bump
    )]
    pub pool: Box<Account<'info, LiquidityPool>>,

    #[account(
        init_if_needed,
        payer = user,
        space = LiquidityProvider::ACCOUNT_SIZE,
        seeds = [LiquidityProvider::SEED_PREFIX.as_bytes(), pool.key().as_ref(), user.key().as_ref()],
        bump,
    )]
    pub liquidity_provider_account: Box<Account<'info, LiquidityProvider>>,

    #[account(
        constraint = !mint_token_one.key().eq(&mint_token_two.key()) @ CustomError::DuplicateTokenNotAllowed
    )]
    pub mint_token_one: Box<Account<'info, Mint>>,

    #[account(
        constraint = !mint_token_one.key().eq(&mint_token_two.key()) @ CustomError::DuplicateTokenNotAllowed
    )]
    pub mint_token_two: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = mint_token_one,
        associated_token::authority = pool
    )]
    pub pool_token_account_one: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_token_two,
        associated_token::authority = pool
    )]
    pub pool_token_account_two: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_token_one,
        associated_token::authority = user,
    )]
    pub user_token_account_one: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_token_two,
        associated_token::authority = user,
    )]
    pub user_token_account_two: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
