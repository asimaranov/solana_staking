use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::state::{Staking, StakerInfo, Round};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer=owner, space = 8 + Staking::LEN, seeds=[b"staking"], bump)]
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
    #[account(init, payer=staker, space = 8 + StakerInfo::LEN, seeds = [b"staker-info", staker.key().as_ref()], bump)]
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

#[derive(Accounts)]
pub struct StartRound<'info> {
    #[account(mut, seeds=[b"staking"], bump)]
    pub staking: Account<'info, Staking>,
    #[account(init, seeds=[b"round", staking.rounds_num.to_le_bytes().as_ref()], space=8+Round::LEN, payer=owner, bump)]
    pub round: Account<'info, Round>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>
}