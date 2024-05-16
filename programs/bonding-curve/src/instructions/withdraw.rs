use crate::{constant::*, errors::CustomErrors::*, state::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer as SplTransfer};

pub fn handle(ctx: Context<Withdraw>, bump: u8) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp as u64;
    if ctx.accounts.lock_state.end_date > current_time {
        return Err(LockPeriodNotEnded.into());
    }

    let vault_token_balance = ctx.accounts.vault_ata.amount;
    if vault_token_balance == 0 {
        return Err(NoTokenInVault.into());
    }

    if ctx.accounts.lock_state.locked == false {
        return Err(AlreadyWithdrawn.into());
    }

    // if ctx.accounts.pool.key() == ctx.accounts.lock_state.pool_id {
    //     return Err(InvalidPool.into());
    // }

    let destination = &ctx.accounts.user_ata;
    let source = &ctx.accounts.vault_ata;
    let global_state = &mut ctx.accounts.global_state;
    let lock_state = &mut ctx.accounts.lock_state;

    let owner_binding = ctx.accounts.owner.key();
    let mint_binding = ctx.accounts.lp_mint.key();
    let seeds: &[&[&[u8]]] = &[&[
        owner_binding.as_ref(),
        mint_binding.as_ref(),
        LOCK_SEED,
        &[bump],
    ]];

    let cpi_context = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        SplTransfer {
            from: source.to_account_info().clone(),
            to: destination.to_account_info().clone(),
            authority: lock_state.to_account_info().clone(),
        },
        seeds,
    );
    transfer(cpi_context, vault_token_balance).ok();

    lock_state.amount = 0;
    lock_state.locked = false;
    global_state.withdraw_times += 1;

    Ok(())
}

#[derive(Accounts)]
#[instruction()]
pub struct Withdraw<'info> {
    #[account(
      mut,
      seeds = [GLOBAL_SEED],
      bump
    )]
    pub global_state: Box<Account<'info, GlobalState>>,

    #[account(
      mut,
      seeds = [owner.key().as_ref(), lp_mint.key().as_ref(), LOCK_SEED],
      bump,
    )]
    pub lock_state: Box<Account<'info, LockState>>,

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
