use crate::{constant::*, errors::CustomErrors::*, state::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer as SplTransfer};

pub fn handle(ctx: Context<IncreaseLockAmount>, lock_amount: u64) -> Result<()> {
    let lock_state = &mut ctx.accounts.lock_state;
    if !lock_state.locked {
        return Err(LpAlreadyWithdrawn.into());
    }
    if ctx.accounts.lp_mint.supply < lock_state.amount + lock_amount {
        return Err(LockMoreThanSupply.into());
    }

    let destination = &ctx.accounts.vault_ata;
    let source = &ctx.accounts.user_ata;
    let token_program = &ctx.accounts.token_program;
    let authority = &ctx.accounts.owner;

    // Transfer tokens from taker to initializer
    let cpi_accounts = SplTransfer {
        from: source.to_account_info().clone(),
        to: destination.to_account_info().clone(),
        authority: authority.to_account_info().clone(),
    };
    let cpi_program = token_program.to_account_info();

    token::transfer(CpiContext::new(cpi_program, cpi_accounts), lock_amount)?;

    lock_state.amount += lock_amount;
    lock_state.lock_token_times += 1;

    Ok(())
}

#[derive(Accounts)]
#[instruction()]
pub struct IncreaseLockAmount<'info> {
    #[account(
      mut,
      seeds = [owner.key().as_ref(), lp_mint.key().as_ref(), LOCK_SEED],
      bump
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
