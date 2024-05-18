use crate::{errors::CustomError, state::*};
use anchor_lang::prelude::*;

pub fn initialize(
    ctx: Context<InitializeDexConfiguration>,
    fees: f64,
) -> Result<()> {
    let dex_config = &mut ctx.accounts.dex_configuration_account;

    if fees < 0_f64 || fees > 100_f64 {
        return err!(CustomError::InvalidFee);
    }

    dex_config.set_inner(DexConfiguration::new(fees));

    Ok(())
}

#[derive(Accounts)]
pub struct InitializeDexConfiguration<'info> {
    #[account(
        init,
        space = DexConfiguration::ACCOUNT_SIZE,
        payer = admin,
        seeds = [DexConfiguration::SEED.as_bytes()],
        bump,
    )]
    pub dex_configuration_account: Box<Account<'info, DexConfiguration>>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}
