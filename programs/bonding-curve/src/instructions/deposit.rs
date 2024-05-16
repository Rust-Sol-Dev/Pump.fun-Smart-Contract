use crate::{constant::*, errors::CustomErrors::*, state::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer as SplTransfer};
use std::mem::size_of;

pub fn handle(ctx: Context<Lock>, lock_amount: u64, lock_duration: u64) -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
#[instruction()]
pub struct Lock<'info> {
    #[account(
    init,
    payer = owner,
    space = 8 + size_of::<LockState>(),
    seeds = [owner.key().as_ref(), lp_mint.key().as_ref(), LOCK_SEED],
    bump,
  )]
    pub lock_state: Box<Account<'info, LockState>>,

    #[account(
      mut,
      seeds = [GLOBAL_SEED],
      bump
    )]
    pub global_state: Box<Account<'info, GlobalState>>,

    #[account(mut)]
    pub lp_mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub vault_ata: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub user_ata: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    /// CHECK: Frontend will check for now
    pub pool: AccountInfo<'info>,

    #[account(mut)]
    pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
