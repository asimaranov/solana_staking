use anchor_lang::prelude::*;

#[account]
pub struct Staking {
    pub owner: Pubkey,
    pub round_time: u64,
    pub rounds_num: u64,
    pub total_fctr_bought_by_users: u64,
    pub total_fctr_sold_by_users: u64,
    pub total_bcdev_sold_by_users: u64,
    pub finished: bool,
    pub finish_time: u64,
    pub fctr_mint: Pubkey,
    pub bcdev_mint: Pubkey,
    pub proof_signer: Pubkey,
    pub bump: u8
}

impl Staking {
    pub const LEN: usize = 8*7 + 32*4 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct EntrustInfo {
    pub principal: Pubkey,
    pub amount: u64
}

#[account()]
pub struct StakerInfo {
    pub staker: Pubkey,
    pub stake_size: u64,
    pub stake_time: u64,
    pub ftcr_amount: u64,
    pub bcdev_amount: u64,
    pub pending_bcdev_reward: u64,
    pub last_update_timestamp: u64,
    pub user_rpr: u64,
    pub bought_fctr: u64,
    pub entrusted_tokens: bool,
    pub is_staked: bool,
    pub is_in_trust_program: bool,
    pub principals: Vec<EntrustInfo>,
    pub bump: u8
}

impl StakerInfo {
    pub const LEN: usize = 32 + 8*8 + 1*5 + (4 + (32 + 8) * 4);
}

