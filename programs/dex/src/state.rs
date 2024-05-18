use crate::errors::CustomError;
use crate::helpers::convert_from_float;
use crate::helpers::convert_to_float;
use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use std::cmp;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;

#[account]
pub struct CurveConfiguration {
    pub fees: f64,
}

impl CurveConfiguration {
    pub const SEED: &'static str = "CurveConfiguration";

    // Discriminator (8) + f64 (8)
    pub const ACCOUNT_SIZE: usize = 8 + 32 + 8;

    pub fn new(fees: f64) -> Self {
        Self { fees }
    }
}

#[account]
pub struct LiquidityProvider {
    pub shares: u64,
}

impl LiquidityProvider {
    pub const SEED_PREFIX: &'static str = "LiqudityProvider";

    // Discriminator (8) + f64 (8)
    pub const ACCOUNT_SIZE: usize = 8 + 8;
}

#[account]
pub struct LiquidityPool {
    pub token_one: Pubkey,
    pub token_two: Pubkey,
    pub total_supply: u64,
    pub reserve_one: u64,
    pub reserve_two: u64,
    pub bump: u8,
}

impl LiquidityPool {
    pub const POOL_SEED_PREFIX: &'static str = "liquidity_pool";

    // Discriminator (8) + Pubkey (32) + Pubkey (32) + totalsupply (8)
    // + reserve one (8) + reserve two (8) + Bump (1)
    pub const ACCOUNT_SIZE: usize = 8 + 32 + 32 + 8 + 8 + 8 + 1;

    pub fn generate_seed(token_one: Pubkey, token_two: Pubkey) -> String {
        if token_one > token_two {
            format!("{}{}", token_one.to_string(), token_two.to_string())
        } else {
            format!("{}{}", token_two.to_string(), token_one.to_string())
        }
    }

    pub fn new(token_one: Pubkey, token_two: Pubkey, bump: u8) -> Self {
        Self {
            token_one: token_one,
            token_two: token_two,
            total_supply: 0_u64,
            reserve_one: 0_u64,
            reserve_two: 0_u64,
            bump: bump,
        }
    }
}

pub trait LiquidityPoolAccount<'info> {
    fn grant_shares(
        &mut self,
        liquidity_provider_account: &mut Account<'info, LiquidityProvider>,
        hares: u64,
    ) -> Result<()>;
    fn remove_shares(
        &mut self,
        liquidity_provider_account: &mut Account<'info, LiquidityProvider>,
        shares: u64,
    ) -> Result<()>;
    fn update_reserves(&mut self, reserve_one: u64, reserve_two: u64) -> Result<()>;

    fn add_liquidity(
        &mut self,
        token_one_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        token_two_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        amount_one: u64,
        amount_two: u64,
        liquidity_provider_account: &mut Account<'info, LiquidityProvider>,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
    ) -> Result<()>;

    fn remove_liquidity(
        &mut self,
        token_one_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        token_two_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        shares: u64,
        liquidity_provider_account: &mut Account<'info, LiquidityProvider>,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
    ) -> Result<()>;

    fn swap(
        &mut self,
        bonding_configuration_account: &Account<'info, CurveConfiguration>,
        token_one_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        token_two_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        amount: u64,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
    ) -> Result<()>;

    fn transfer_token_from_pool(
        &self,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        amount: u64,
        token_program: &Program<'info, Token>,
    ) -> Result<()>;

    fn transfer_token_to_pool(
        &self,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        amount: u64,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
    ) -> Result<()>;

    fn transfer_sol_to_pool(
        &self,
        from: &Signer<'info>,
        amount: u64,
        system_program: &Program<'info, System>,
    ) -> Result<()>;

    fn transfer_sol_from_pool(
        &self,
        to: &AccountInfo<'info>,
        amount: u64,
        system_program: &Program<'info, System>,
    ) -> Result<()>;
}

impl<'info> LiquidityPoolAccount<'info> for Account<'info, LiquidityPool> {
    fn grant_shares(
        &mut self,
        liquidity_provider_account: &mut Account<'info, LiquidityProvider>,
        shares: u64,
    ) -> Result<()> {
        liquidity_provider_account.shares = liquidity_provider_account
            .shares
            .checked_add(shares)
            .ok_or(CustomError::FailedToAllocateShares)?;

        self.total_supply = self
            .total_supply
            .checked_sub(shares)
            .ok_or(CustomError::OverflowOrUnderflowOccurred)?;

        Ok(())
    }

    fn remove_shares(
        &mut self,
        liquidity_provider_account: &mut Account<'info, LiquidityProvider>,
        shares: u64,
    ) -> Result<()> {
        liquidity_provider_account.shares = liquidity_provider_account
            .shares
            .checked_sub(shares)
            .ok_or(CustomError::FailedToDeallocateShares)?;

        self.total_supply = self
            .total_supply
            .checked_sub(shares)
            .ok_or(CustomError::OverflowOrUnderflowOccurred)?;

        Ok(())
    }

    fn update_reserves(&mut self, reserve_one: u64, reserve_two: u64) -> Result<()> {
        self.reserve_one = reserve_one;
        self.reserve_two = reserve_two;

        Ok(())
    }

    fn add_liquidity(
        &mut self,
        token_one_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        token_two_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        amount_one: u64,
        amount_two: u64,
        liquidity_provider_account: &mut Account<'info, LiquidityProvider>,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
    ) -> Result<()> {
        let mut shares_to_allocate = 0_u64;

        if self.total_supply == 0 {
            let sqrt_shares = (convert_to_float(amount_one, token_one_accounts.0.decimals)
                .mul(convert_to_float(amount_two, token_two_accounts.0.decimals)))
            .sqrt();

            shares_to_allocate = sqrt_shares as u64;
        } else {
            let mul_value = amount_one
                .checked_mul(self.total_supply)
                .ok_or(CustomError::OverflowOrUnderflowOccurred)?;
            let shares_one = mul_value
                .checked_div(self.reserve_one)
                .ok_or(CustomError::OverflowOrUnderflowOccurred)?;

            let mul_value = amount_two
                .checked_mul(self.total_supply)
                .ok_or(CustomError::OverflowOrUnderflowOccurred)?;
            let shares_two = mul_value
                .checked_div(self.reserve_two)
                .ok_or(CustomError::OverflowOrUnderflowOccurred)?;

            shares_to_allocate = cmp::min(shares_one, shares_two);
        }

        if shares_to_allocate <= 0 {
            return err!(CustomError::FailedToAddLiquidity);
        }

        self.grant_shares(liquidity_provider_account, shares_to_allocate)?;

        let new_reserves_one = self
            .reserve_one
            .checked_add(amount_one)
            .ok_or(CustomError::OverflowOrUnderflowOccurred)?;
        let new_reserves_two = self
            .reserve_two
            .checked_add(amount_two)
            .ok_or(CustomError::OverflowOrUnderflowOccurred)?;

        self.update_reserves(new_reserves_one, new_reserves_two)?;

        self.transfer_token_to_pool(
            token_one_accounts.2,
            token_one_accounts.1,
            amount_one,
            authority,
            token_program,
        )?;

        self.transfer_token_to_pool(
            token_two_accounts.2,
            token_two_accounts.1,
            amount_two,
            authority,
            token_program,
        )?;

        Ok(())
    }

    fn remove_liquidity(
        &mut self,
        token_one_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        token_two_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        shares: u64,
        liquidity_provider_account: &mut Account<'info, LiquidityProvider>,
        _authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
    ) -> Result<()> {
        if shares <= 0 {
            return err!(CustomError::FailedToRemoveLiquidity);
        }

        if liquidity_provider_account.shares < shares {
            return err!(CustomError::InsufficientShares);
        }

        let mul_value = shares
            .checked_mul(self.reserve_one)
            .ok_or(CustomError::OverflowOrUnderflowOccurred)?;

        let amount_out_one = mul_value
            .checked_div(self.total_supply)
            .ok_or(CustomError::OverflowOrUnderflowOccurred)?;

        let mul_value = shares
            .checked_mul(self.reserve_two)
            .ok_or(CustomError::OverflowOrUnderflowOccurred)?;

        let amount_out_two = mul_value
            .checked_div(self.total_supply)
            .ok_or(CustomError::OverflowOrUnderflowOccurred)?;

        if amount_out_one <= 0 || amount_out_two <= 0 {
            return err!(CustomError::FailedToRemoveLiquidity);
        }

        self.remove_shares(liquidity_provider_account, shares)?;

        let new_reserves_one = self
            .reserve_one
            .checked_sub(amount_out_one)
            .ok_or(CustomError::OverflowOrUnderflowOccurred)?;
        let new_reserves_two = self
            .reserve_two
            .checked_sub(amount_out_two)
            .ok_or(CustomError::OverflowOrUnderflowOccurred)?;

        self.update_reserves(new_reserves_one, new_reserves_two)?;

        self.transfer_token_from_pool(
            token_one_accounts.1,
            token_one_accounts.2,
            amount_out_one,
            token_program,
        )?;

        self.transfer_token_from_pool(
            token_two_accounts.1,
            token_two_accounts.2,
            amount_out_two,
            token_program,
        )?;
        Ok(())
    }

    fn swap(
        &mut self,
        bonding_configuration_account: &Account<'info, CurveConfiguration>,
        token_one_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        token_two_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        amount: u64,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
    ) -> Result<()> {
        if amount <= 0 {
            return err!(CustomError::InvalidAmount);
        }

        // xy = k => Constant product formula
        // (x + dx)(y - dy) = k
        // y - dy = k / (x + dx)
        // y - dy = xy / (x + dx)
        // dy = y - (xy / (x + dx))
        // dy = yx + ydx - xy / (x + dx)
        // formula => dy = ydx / (x + dx)

        let adjusted_amount_in_float = convert_to_float(amount, token_one_accounts.0.decimals)
            .div(100_f64)
            .mul(100_f64.sub(bonding_configuration_account.fees));

        let adjusted_amount =
            convert_from_float(adjusted_amount_in_float, token_one_accounts.0.decimals);

        let denominator_sum = self
            .reserve_one
            .checked_add(adjusted_amount)
            .ok_or(CustomError::OverflowOrUnderflowOccurred)?;

        let numerator_mul = self
            .reserve_two
            .checked_mul(adjusted_amount)
            .ok_or(CustomError::OverflowOrUnderflowOccurred)?;

        let amount_out = numerator_mul
            .checked_div(denominator_sum)
            .ok_or(CustomError::OverflowOrUnderflowOccurred)?;

        let new_reserves_one = self
            .reserve_one
            .checked_add(amount)
            .ok_or(CustomError::OverflowOrUnderflowOccurred)?;
        let new_reserves_two = self
            .reserve_two
            .checked_sub(amount_out)
            .ok_or(CustomError::OverflowOrUnderflowOccurred)?;

        self.update_reserves(new_reserves_one, new_reserves_two)?;

        self.transfer_token_to_pool(
            token_one_accounts.2,
            token_one_accounts.1,
            amount,
            authority,
            token_program,
        )?;

        self.transfer_token_from_pool(
            token_two_accounts.1,
            token_two_accounts.2,
            amount_out,
            token_program,
        )?;
        Ok(())
    }

    fn transfer_token_from_pool(
        &self,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        amount: u64,
        token_program: &Program<'info, Token>,
    ) -> Result<()> {
        token::transfer(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                token::Transfer {
                    from: from.to_account_info(),
                    to: to.to_account_info(),
                    authority: self.to_account_info(),
                },
                &[&[
                    LiquidityPool::POOL_SEED_PREFIX.as_bytes(),
                    LiquidityPool::generate_seed(self.token_one.key(), self.token_two.key())
                        .as_bytes(),
                    &[self.bump],
                ]],
            ),
            amount,
        )?;

        Ok(())
    }

    fn transfer_token_to_pool(
        &self,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        amount: u64,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
    ) -> Result<()> {
        token::transfer(
            CpiContext::new(
                token_program.to_account_info(),
                token::Transfer {
                    from: from.to_account_info(),
                    to: to.to_account_info(),
                    authority: authority.to_account_info(),
                },
            ),
            amount,
        )?;

        Ok(())
    }

    fn transfer_sol_from_pool(
        &self,
        to: &AccountInfo<'info>,
        amount: u64,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        let pool_account_info = self.to_account_info();

        system_program::transfer(
            CpiContext::new_with_signer(
                system_program.to_account_info(),
                system_program::Transfer {
                    from: pool_account_info,
                    to: to.clone(),
                },
                &[&[
                    LiquidityPool::POOL_SEED_PREFIX.as_bytes(),
                    LiquidityPool::generate_seed(self.token_one.key(), self.token_two.key())
                        .as_bytes(),
                    &[self.bump],
                ]],
            ),
            amount,
        )?;

        Ok(())
    }

    fn transfer_sol_to_pool(
        &self,
        from: &Signer<'info>,
        amount: u64,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        let pool_account_info = self.to_account_info();

        system_program::transfer(
            CpiContext::new(
                system_program.to_account_info(),
                system_program::Transfer {
                    from: from.to_account_info(),
                    to: pool_account_info,
                },
            ),
            amount,
        )?;
        Ok(())
    }
}
