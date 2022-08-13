use anchor_lang::prelude::*;

#[error_code]
pub enum StakingError {
    #[msg("Invalid token account")]
    InvalidTokenAccount
}