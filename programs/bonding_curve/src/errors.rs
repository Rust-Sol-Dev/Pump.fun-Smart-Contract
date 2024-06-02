use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Duplicate tokens are not allowed")]
    DuplicateTokenNotAllowed,

    #[msg("Failed to allocate shares")]
    FailedToAllocateShares,

    #[msg("Failed to deallocate shares")]
    FailedToDeallocateShares,

    #[msg("Insufficient shares")]
    InsufficientShares,

    #[msg("Insufficient funds to swap")]
    InsufficientFunds,

    #[msg("Invalid amount to swap")]
    InvalidAmount,

    #[msg("Invalid fee")]
    InvalidFee,

    #[msg("Failed to add liquidity")]
    FailedToAddLiquidity,

    #[msg("Failed to remove liquidity")]
    FailedToRemoveLiquidity,
    
    #[msg("Sold token is not enough to remove pool")]
    NotEnoughToRemove,

    #[msg("Not a pool creator")]
    NotCreator,

    #[msg("Overflow or underflow occured")]
    OverflowOrUnderflowOccurred,

    #[msg("Token amount is too big to sell")]
    TokenAmountToSellTooBig,

    #[msg("SOL is not enough in vault")]
    NotEnoughSolInVault,

    #[msg("Token is not enough in vault")]
    NotEnoughTokenInVault,
   
    #[msg("Amount is negative")]
    NegativeNumber,
}
