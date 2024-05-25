use anchor_lang::prelude::*;

pub mod errors;
pub mod utils;
pub mod instructions;
pub mod state;
pub mod consts;

use crate::instructions::*;

declare_id!("JCmZnunFj922uGE1KpbXey7Ynhsny6H1trjmU2mqqCa3");

#[program]
pub mod bonding_curve {
    use super::*;

    pub fn initialize(ctx: Context<InitializeCurveConfiguration>, fee: f64) -> Result<()> {
        instructions::initialize(ctx, fee)
    }

    pub fn create_pool(ctx: Context<CreateLiquidityPool>) -> Result<()> {
        instructions::create_pool(ctx)
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        amount_one: u64,
        amount_two: u64,
    ) -> Result<()> {
        instructions::add_liquidity(ctx, amount_one, amount_two)
    }

    pub fn remove_liquidity(ctx: Context<RemoveLiquidity>, shares: u64) -> Result<()> {
        instructions::remove_liquidity(ctx, shares)
    }

    pub fn swap(ctx: Context<Swap>, amount: u64) -> Result<()> {
        instructions::swap(ctx, amount)
    }
}
