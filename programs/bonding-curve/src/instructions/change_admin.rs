
use anchor_lang::prelude::*;
use crate::{constant::*, state::*, errors::CustomErrors::*};

pub fn handle(ctx: Context<ChangeAdmin>, new_admin: Pubkey) ->Result<()> {
  if ctx.accounts.admin.to_account_info().key() != ctx.accounts.global_state.admin {
    return Err(InvalidAdmin.into());
  }

  ctx.accounts.global_state.admin = new_admin;

  Ok(())
}

#[derive(Accounts)]
#[instruction()]
pub struct ChangeAdmin<'info> {
  #[account(
    mut,
    seeds = [GLOBAL_SEED],
    bump,
  )]
  pub global_state: Box<Account<'info, GlobalState>>,

  #[account(mut)]
  pub admin: Signer<'info>,

  pub system_program: Program<'info, System>,
}

