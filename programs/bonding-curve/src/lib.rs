use anchor_lang::prelude::*;

pub mod state;

pub mod constant;

pub mod errors;

pub mod instructions;

use crate::instructions::*;

declare_id!("8mpK8svcXKb4PuY8QtE3xd8k3AcBgonUJBSgMazeo3tz");

#[program]
pub mod bonding_curve {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
			initialize::handle(ctx)
		}

		pub fn lock(ctx: Context<Lock>, lock_amount: u64, lock_duration: u64) -> Result<()> {
			deposit::handle(ctx, lock_amount, lock_duration)
		}

		pub fn withdraw(ctx: Context<Withdraw>, bump: u8) -> Result<()> {
			withdraw::handle(ctx, bump)
		}

		pub fn change_admin(ctx: Context<ChangeAdmin>, new_admin: Pubkey) -> Result<()> {
			change_admin::handle(ctx, new_admin)
		}

}
