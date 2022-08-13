use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::state::{Staking, StakerInfo};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer=owner, space = 8 + Staking::MAX_SIZE, seeds=[b"staking"], bump)]
    pub staking: Account<'info, Staking>,

    #[account(mut)]
    pub staking_fctr_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct Register<'info> {
    #[account(mut)]
    pub staker: Signer<'info>,
    #[account(init, payer=staker, space = 8 + StakerInfo::MAX_SIZE, seeds = [b"staker-info", staker.key().as_ref()], bump)]
    pub staker_info: Account<'info, StakerInfo>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut, seeds=[b"staking"], bump)]
    pub staking: Account<'info, Staking>,
    #[account(mut, seeds=[b"staker-info", staker.key().as_ref()], bump)]
    pub staker_info: Account<'info, StakerInfo>,
    #[account(mut, token::authority=staker)]
    pub staker_fctr_account: Account<'info, TokenAccount>,
    #[account(mut, token::authority=staking)]
    pub staking_fctr_account: Account<'info, TokenAccount>,
    pub staker: Signer<'info>,
    pub token_program: Program<'info, Token>
}