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
    #[msg("Can't buy if in trust program")]
    CantBuyInTrustProgram,
    #[msg("Not the owner")]
    NotTheOwner,
    #[msg("Invalid amont entrusted")]
    InvalidAmountEntrusted,
    #[msg("Staking finished")]
    StakingFinished,
    #[msg("Too much principals")]
    TooMuchPrincipals,
    #[msg("Can't withdraw")]
    CantWithdraw,
    #[msg("Deposit diff too high")]
    InvalidDepositDiff,
    #[msg("Not enough funds in staking")]
    NotEnoughFunds,
    #[msg("User can't unstake in round he staked")]
    CantUnstakeInThisVeryRound,
    #[msg("Invalid principal for the confidant")]
    NoSuchPrincipal,
    #[msg("The principal has to trusted money to the condidant")]
    NothingToDemandBack,
    #[msg("Invalid token account owner")]
    InvalidTokenAccountOwner,
    #[msg("Invalid principal info account")]
    InvalidPrincipalInfoAccount,



}