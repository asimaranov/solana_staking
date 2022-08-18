use anchor_lang::prelude::*;

#[error_code]
pub enum StakingError {
    #[msg("Invalid token account")]
    InvalidTokenAccount,
    #[msg("Previous round is not finished")]
    PrevRoundIsNotFinished,
    #[msg("Too few token amount to buy")]
    TooFewAmount,
    #[msg("Invalid token mint")]
    InvalidMint,
    #[msg("Not enough tokens")]
    NotEnoughTokens,
    #{msg("Can't buy if in trust program")}
    CantBuyInTrustProgram




}