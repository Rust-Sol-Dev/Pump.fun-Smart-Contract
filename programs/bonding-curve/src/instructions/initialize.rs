
use anchor_lang::prelude::*;
use std::mem::size_of;
use crate::{constant::*, state::*};

pub fn handle(ctx: Context<Initialize>) ->Result<()> {
  let global_state = &mut ctx.accounts.global_state;
  global_state.admin = ctx.accounts.admin.key();
  global_state.locked_lp_num = 0;
  global_state.withdraw_times = 0;
  global_state.minimum_lock_duration = MINIMUM_LOCK_DURATION;
  global_state.minimum_lock_percent = MINIMUM_LOCK_PERCENT;

  Ok(())
}

#[derive(Accounts)]
#[instruction()]
pub struct Initialize<'info> {
  #[account(
    init,
    seeds = [GLOBAL_SEED],
    bump,
    space = 8 + size_of::<GlobalState>(),
    payer = admin
  )]
  pub global_state: Box<Account<'info, GlobalState>>,

  #[account(mut)]
  pub admin: Signer<'info>,

  pub system_program: Program<'info, System>,
}

