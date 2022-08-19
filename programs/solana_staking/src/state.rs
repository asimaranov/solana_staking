use anchor_lang::prelude::*;

#[account]
pub struct Staking {
    pub owner: Pubkey,
    pub round_time: u64,
    pub rounds_num: u64,
    pub fctr_mint: Pubkey,
    pub bcdev_mint: Pubkey,
    pub last_round_deadline: u64,
    pub round_start_times: Vec<u64>,
    pub total_fctr_bought_by_users: u64,
    pub total_fctr_sold_by_users: u64,
    pub total_bcdev_sold_by_users: u64,
    pub bump: u8
}

impl Staking {
    pub const LEN: usize = 32 + 8 + 1;
}


#[account]
pub struct StakerInfo {
    pub staker: Pubkey,
    pub stake_size: u64,
    pub stake_time: u64,
    pub ftcr_amount: u64,
    pub bcdev_amount: u64,
    pub bought_fctr: u64,
    pub entrusted_tokens: bool,
    pub is_in_trust_program: bool,
    pub bump: u8
}

impl StakerInfo {
    pub const LEN: usize = 32 + 8 + 8 + 1 + 1;
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

