use anchor_lang::prelude::*;

#[account]
pub struct Staking {
    pub owner: Pubkey,
    pub round_time: u64,
    pub rounds_num: u64,
    pub fctr_mint: Pubkey,
    pub bcdev_mint: Pubkey,
    pub bump: u8
}

impl Staking {
    pub const LEN: usize = 32 + 8 + 1;
}

#[account]
pub struct StakerInfo {
    pub staker: Pubkey,
    pub stake_size: u64,
    pub bump: u8
}

impl StakerInfo {
    pub const LEN: usize = 32 + 1;
}

#[account]
pub struct Round {
    pub is_final: bool,
    pub is_finished: bool,
    pub start_time: u64,
    pub treasury: u64,
    pub bump: u8
}


impl Round {
    pub const LEN: usize = 1 + 1 + 8 + 1;
}

