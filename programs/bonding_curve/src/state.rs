use crate::consts::INITIAL_LAMPORTS_FOR_POOL;
use crate::consts::INITIAL_PRICE_DIVIDER;
use crate::consts::PROPORTION;
use crate::errors::CustomError;
use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

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
    pub shares: u64, // The number of shares this provider holds in the liquidity pool ( didnt add to contract now )
}

impl LiquidityProvider {
    pub const SEED_PREFIX: &'static str = "LiqudityProvider"; // Prefix for generating PDAs

    // Discriminator (8) + f64 (8)
    pub const ACCOUNT_SIZE: usize = 8 + 8;
}

#[account]
pub struct LiquidityPool {
    pub creator: Pubkey,    // Public key of the pool creator
    pub token: Pubkey,      // Public key of the token in the liquidity pool
    pub total_supply: u64,  // Total supply of liquidity tokens
    pub reserve_token: u64, // Reserve amount of token in the pool
    pub reserve_sol: u64,   // Reserve amount of sol_token in the pool
    pub bump: u8,           // Nonce for the program-derived address
}

impl LiquidityPool {
    pub const POOL_SEED_PREFIX: &'static str = "liquidity_pool";
    pub const SOL_VAULT_PREFIX: &'static str = "liquidity_sol_vault";

    // Discriminator (8) + Pubkey (32) + Pubkey (32) + totalsupply (8)
    // + reserve one (8) + reserve two (8) + Bump (1)
    pub const ACCOUNT_SIZE: usize = 8 + 32 + 32 + 8 + 8 + 8 + 1;

    // Constructor to initialize a LiquidityPool with two tokens and a bump for the PDA
    pub fn new(creator: Pubkey, token: Pubkey, bump: u8) -> Self {
        Self {
            creator,
            token,
            total_supply: 0_u64,
            reserve_token: 0_u64,
            reserve_sol: 0_u64,
            bump,
        }
    }
}

pub trait LiquidityPoolAccount<'info> {
    // Updates the token reserves in the liquidity pool
    fn update_reserves(&mut self, reserve_token: u64, reserve_sol: u64) -> Result<()>;

    // Allows adding liquidity by depositing an amount of two tokens and getting back pool shares
    fn add_liquidity(
        &mut self,
        token_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        pool_sol_vault: &mut AccountInfo<'info>,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
        system_program: &Program<'info, System>,
    ) -> Result<()>;

    // Allows removing liquidity by burning pool shares and receiving back a proportionate amount of tokens
    fn remove_liquidity(
        &mut self,
        token_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        pool_sol_account: &mut AccountInfo<'info>,
        authority: &Signer<'info>,
        bump: u8,
        token_program: &Program<'info, Token>,
        system_program: &Program<'info, System>,
    ) -> Result<()>;

    fn buy(
        &mut self,
        // bonding_configuration_account: &Account<'info, CurveConfiguration>,
        token_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        pool_sol_vault: &mut AccountInfo<'info>,
        amount: u64,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
        system_program: &Program<'info, System>,
    ) -> Result<()>;

    fn sell(
        &mut self,
        // bonding_configuration_account: &Account<'info, CurveConfiguration>,
        token_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        pool_sol_vault: &mut AccountInfo<'info>,
        amount: u64,
        bump: u8,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
        system_program: &Program<'info, System>,
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
        to: &mut AccountInfo<'info>,
        amount: u64,
        system_program: &Program<'info, System>,
    ) -> Result<()>;

    fn transfer_sol_from_pool(
        &self,
        from: &mut AccountInfo<'info>,
        to: &Signer<'info>,
        amount: u64,
        bump: u8,
        system_program: &Program<'info, System>,
    ) -> Result<()>;
}

impl<'info> LiquidityPoolAccount<'info> for Account<'info, LiquidityPool> {
    fn update_reserves(&mut self, reserve_token: u64, reserve_sol: u64) -> Result<()> {
        self.reserve_token = reserve_token;
        self.reserve_sol = reserve_sol;
        Ok(())
    }

    fn add_liquidity(
        &mut self,
        token_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        pool_sol_vault: &mut AccountInfo<'info>,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        self.transfer_token_to_pool(
            token_accounts.2,
            token_accounts.1,
            token_accounts.0.supply,
            authority,
            token_program,
        )?;

        self.transfer_sol_to_pool(
            authority,
            pool_sol_vault,
            INITIAL_LAMPORTS_FOR_POOL,
            system_program,
        )?;
        self.total_supply = 1_000_000_000_000_000_000;
        self.update_reserves(token_accounts.0.supply, INITIAL_LAMPORTS_FOR_POOL)?;

        Ok(())
    }

    fn remove_liquidity(
        &mut self,
        token_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        pool_sol_vault: &mut AccountInfo<'info>,
        authority: &Signer<'info>,
        bump: u8,
        token_program: &Program<'info, Token>,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        self.transfer_token_from_pool(
            token_accounts.1,
            token_accounts.2,
            token_accounts.1.amount as u64,
            token_program,
        )?;
        // let amount = self.to_account_info().lamports() - self.get_lamports();
        let amount = pool_sol_vault.to_account_info().lamports() as u64;
        self.transfer_sol_from_pool(pool_sol_vault, authority, amount, bump, system_program)?;

        Ok(())
    }

    ///////////////////////////////////////////////////////////////
    ///////////////////////////////////////////////////////////////
    //
    //              Linear bonding curve swap
    //
    /////////////////////////////////////////////////////////////
    /////////////////////////////////////////////////////////////
    //
    //  Linear bonding curve : S = T * P ( here, p is constant that show initial price )
    //  SOL amount => S
    //  Token amount => T
    //  Initial Price => P
    //
    //  SOL amount to buy Token a => S_a = ((T_a  + 1) * T_a / 2) * P
    //  SOL amount to buy Token b => S_b = ((T_b + 1) * T_b / 2) * P

    fn buy(
        &mut self,
        // _bonding_configuration_account: &Account<'info, CurveConfiguration>,
        token_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        pool_sol_vault: &mut AccountInfo<'info>,
        amount: u64,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        if amount == 0 {
            return err!(CustomError::InvalidAmount);
        }

        msg!("Trying to buy from the pool");

        let bought_amount =
            (self.total_supply as f64 - self.reserve_token as f64) / 1_000_000.0 / 1_000_000_000.0;
        msg!("bought_amount {}", bought_amount);

        let root_val = (PROPORTION as f64 * amount as f64 / 1_000_000_000.0
            + bought_amount * bought_amount)
            .sqrt();
        msg!("root_val {}", root_val);

        let amount_out_f64 = (root_val - bought_amount as f64) * 1_000_000.0 * 1_000_000_000.0;
        msg!("amount_out_f64 {}", amount_out_f64);

        let amount_out = amount_out_f64.round() as u64;
        msg!("amount_out {}", amount_out);

        if amount_out > self.reserve_token {
            return err!(CustomError::NotEnoughTokenInVault);
        }

        self.reserve_sol += amount;
        self.reserve_token -= amount_out;

        self.transfer_sol_to_pool(authority, pool_sol_vault, amount, system_program)?;

        self.transfer_token_from_pool(
            token_accounts.1,
            token_accounts.2,
            amount_out,
            token_program,
        )?;
        Ok(())
    }

    fn sell(
        &mut self,
        token_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        pool_sol_vault: &mut AccountInfo<'info>,
        amount: u64,
        bump: u8,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        if amount == 0 {
            return err!(CustomError::InvalidAmount);
        }

        if self.reserve_token < amount {
            return err!(CustomError::TokenAmountToSellTooBig);
        }

        let bought_amount =
            (self.total_supply as f64 - self.reserve_token as f64) / 1_000_000.0 / 1_000_000_000.0;
        msg!("bought_amount: {}", bought_amount);

        let result_amount = (self.total_supply as f64 - self.reserve_token as f64 - amount as f64)
            / 1_000_000.0
            / 1_000_000_000.0;
        msg!("result_amount: {}", result_amount);

        let amount_out_f64 = (bought_amount * bought_amount - result_amount * result_amount)
            / PROPORTION as f64
            * 1_000_000_000.0;
        msg!("amount_out_f64: {}", amount_out_f64);

        let amount_out = amount_out_f64.round() as u64;
        msg!("amount_out: {}", amount_out);

        if self.reserve_sol < amount_out {
            return err!(CustomError::NotEnoughSolInVault);
        }

        self.transfer_token_to_pool(
            token_accounts.2,
            token_accounts.1,
            amount as u64,
            authority,
            token_program,
        )?;

        self.reserve_token += amount;
        self.reserve_sol -= amount_out;

        self.transfer_sol_from_pool(pool_sol_vault, authority, amount_out, bump, system_program)?;

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
                    self.token.key().as_ref(),
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
        from: &mut AccountInfo<'info>,
        to: &Signer<'info>,
        amount: u64,
        bump: u8,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        // let pool_account_info = self.to_account_info();

        system_program::transfer(
            CpiContext::new_with_signer(
                system_program.to_account_info(),
                system_program::Transfer {
                    from: from.clone(),
                    to: to.to_account_info().clone(),
                },
                &[&[
                    LiquidityPool::SOL_VAULT_PREFIX.as_bytes(),
                    self.token.key().as_ref(),
                    // LiquidityPool::POOL_SEED_PREFIX.as_bytes(),
                    // self.token.key().as_ref(),
                    &[bump],
                ]],
            ),
            amount,
        )?;
        Ok(())
    }

    fn transfer_sol_to_pool(
        &self,
        from: &Signer<'info>,
        to: &mut AccountInfo<'info>,
        amount: u64,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        // let pool_account_info = self.to_account_info();

        system_program::transfer(
            CpiContext::new(
                system_program.to_account_info(),
                system_program::Transfer {
                    from: from.to_account_info(),
                    to: to.to_account_info(),
                },
            ),
            amount,
        )?;
        Ok(())
    }
}

fn calculate_amount_out(reserve_token_with_decimal: u64, amount_with_decimal: u64) -> Result<u64> {
    // Convert to f64 for decimal calculations
    let reserve_token = (reserve_token_with_decimal as f64) / 1_000_000_000.0;
    let amount = (amount_with_decimal as f64) / 1_000_000_000.0;

    msg!(
        "Starting calculation with reserve_token: {}, amount: {}",
        reserve_token,
        amount
    );

    let two_reserve_token = reserve_token * 2.0;
    msg!("two_reserve_token: {}", two_reserve_token);

    let one_added = two_reserve_token + 1.0;
    msg!("one_added: {}", one_added);

    let squared = one_added * one_added;
    msg!("squared: {}", squared);

    // Use `amount` directly as it's already a decimal in f64
    let amount_added = squared + amount * 8.0;
    msg!("amount_added: {}", amount_added);

    // Square root calculation
    let sqrt_result = amount_added.sqrt();
    msg!("sqrt_result: {}", sqrt_result);

    // Check if sqrt_result is valid
    if sqrt_result < 0.0 {
        msg!("Error: Negative sqrt_result");
        return err!(CustomError::NegativeNumber);
    }

    let subtract_one = sqrt_result - one_added;
    msg!("subtract_one: {}", subtract_one);

    let amount_out = subtract_one / 2.0;
    msg!("amount_out: {}", amount_out);

    // Convert the final result back to u64 with appropriate scaling
    let amount_out_decimal =
        (amount_out * 1_000_000_000.0 * INITIAL_PRICE_DIVIDER as f64).round() as u64;
    msg!("amount_out_decimal: {}", amount_out_decimal);

    Ok(amount_out_decimal)
}


fn calculate_amount_out(reserve_token_decimal: u64, amount_decimal: u64) -> Result<u64> {
    let reserve_token = reserve_token_decimal.checked_div(1000000000).ok_or(CustomError::OverflowOrUnderflowOccurred)?;
    let amount = amount_decimal.checked_div(1000000000).ok_or(CustomError::OverflowOrUnderflowOccurred)?;
    msg!("Starting calculation with reserve_token: {}, amount: {}", reserve_token, amount);
    let two_reserve_token = reserve_token.checked_mul(2).ok_or(CustomError::OverflowOrUnderflowOccurred)?;
    msg!("two_reserve_token: {}", two_reserve_token);

    let one_added = two_reserve_token.checked_add(1).ok_or(CustomError::OverflowOrUnderflowOccurred)?;
    msg!("one_added: {}", one_added);

    let squared = one_added.checked_mul(one_added).ok_or(CustomError::OverflowOrUnderflowOccurred)?;
    msg!("squared: {}", squared);

    let amount_divided = amount.checked_mul(INITIAL_PRICE_DIVIDER).ok_or(CustomError::OverflowOrUnderflowOccurred)?;
    msg!("amount_divided: {}", amount_divided);

    let amount_added = squared.checked_add(amount_divided).ok_or(CustomError::OverflowOrUnderflowOccurred)?;
    msg!("amount_added: {}", amount_added);

    // Convert to f64 for square root calculation
    let sqrt_result = (amount_added as f64).sqrt();
    msg!("sqrt_result: {}", sqrt_result);

    // Check if sqrt_result can be converted back to u64 safely
    if sqrt_result < 0.0 {
        msg!("Error: Negative sqrt_result");
        return err!(CustomError::NegativeNumber);
    }

    let sqrt_u64 = sqrt_result as u64;
    msg!("sqrt_u64: {}", sqrt_u64);

    let subtract_one = sqrt_u64.checked_sub(one_added).ok_or(CustomError::OverflowOrUnderflowOccurred)?;
    msg!("subtract_one: {}", subtract_one);

    let amount_out = subtract_one.checked_div(2).ok_or(CustomError::OverflowOrUnderflowOccurred)?;
    msg!("amount_out: {}", amount_out);
    let amount_out_decimal = amount_out.checked_mul(1000000000);
    Ok(amount_out)
}
