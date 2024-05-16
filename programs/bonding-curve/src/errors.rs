use anchor_lang::prelude::*;

#[error_code]
pub enum CustomErrors {
    #[msg("Math operation overflow")]
    MathOverflow,

    #[msg("Lp token lock amount is smaller than minimum")]
    SmallLpLockAmount,

    #[msg("Lp token lock period is shorter than minimum")]
    ShorterLockDuration,

    #[msg("Insufficient token balance in vault")]
    InsufficientToken,

    #[msg("Token vault has no token")]
    NoTokenInVault,

    #[msg("Pool ID doesn't match with stored user pool ID")]
    InvalidPool,

    #[msg("Incorrect Admin")]
    InvalidAdmin,
}
