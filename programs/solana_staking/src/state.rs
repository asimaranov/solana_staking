use anchor_lang::prelude::*;

#[account]
pub struct Staking {
    pub owner: Pubkey,
    pub round_time: u64,
    pub fctr_mint: Pubkey,
    pub bcdev_mint: Pubkey,
    pub bump: u8
}

impl Staking {
    pub const MAX_SIZE: usize = 32 + 8 + 1;
}

#[account]
pub struct StakerInfo {
    pub staker: Pubkey,
    pub stake_size: u64,
    pub bump: u8
}

impl StakerInfo {
    pub const MAX_SIZE: usize = 32 + 1;
}

#[account]
pub struct Round {
    pub is_final: bool,
    pub treasury: u64,
    pub bump: u8
}
