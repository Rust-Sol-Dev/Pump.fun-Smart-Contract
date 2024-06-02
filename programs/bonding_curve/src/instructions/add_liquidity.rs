use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::state::{LiquidityPool, LiquidityPoolAccount};

pub fn add_liquidity(ctx: Context<AddLiquidity>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    let token_accounts = (
        &mut *ctx.accounts.token_mint,
        &mut *ctx.accounts.pool_token_account,
        &mut *ctx.accounts.user_token_account,
    );

    pool.add_liquidity(
        token_accounts,
        &mut ctx.accounts.pool_sol_vault,
        &ctx.accounts.user,
        &ctx.accounts.token_program,
        &ctx.accounts.system_program,
    )?;
    Ok(())
}

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(
        mut,
        seeds = [LiquidityPool::POOL_SEED_PREFIX.as_bytes(), token_mint.key().as_ref()],
        bump
    )]
    pub pool: Account<'info, LiquidityPool>,

    #[account(mut)]
    pub token_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = pool
    )]
    pub pool_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = user,
    )]
    pub user_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK:
    #[account(
        mut,
        seeds = [LiquidityPool::SOL_VAULT_PREFIX.as_bytes(), token_mint.key().as_ref()],
        bump
    )]
    pub pool_sol_vault: AccountInfo<'info>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
