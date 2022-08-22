use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};

use crate::{state::{Staking, StakerInfo}};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer=owner, space = 8 + Staking::LEN, seeds=[b"staking"], bump)]
    pub staking: Account<'info, Staking>,

    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct Fund<'info> {
    #[account(init, payer=owner, space = 8 + Staking::LEN, seeds=[b"staking"], bump)]
    pub staking: Account<'info, Staking>,

    #[account(mut, token::authority=staking)]
    pub staking_fctr_account: Account<'info, TokenAccount>,

    #[account(mut, token::authority=owner)]
    pub owner_fctr_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>
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
pub struct Unstake<'info> {
    #[account(mut, seeds=[b"staking"], bump)]
    pub staking: Account<'info, Staking>,
    #[account(mut, seeds=[b"staker-info", staker.key().as_ref()], bump)]
    pub staker_info: Account<'info, StakerInfo>,
    #[account(mut, token::authority=staker, token::mint=fctr_mint)]
    pub staker_fctr_account: Account<'info, TokenAccount>,
    #[account(mut, token::authority=staking, token::mint=fctr_mint)]
    pub staking_fctr_account: Account<'info, TokenAccount>,
    #[account(mut, token::authority=staking, token::mint=bcdev_mint)]
    pub staking_bcdev_account: Account<'info, TokenAccount>,
    #[account(mut, token::authority=staker, token::mint=bcdev_mint)]
    pub staker_bcdev_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub bcdev_mint: Account<'info, Mint>,
    #[account(mut)]
    pub fctr_mint: Account<'info, Mint>,
    pub staker: Signer<'info>,
    pub token_program: Program<'info, Token>
}

#[derive(Accounts)]
pub struct BuyFctr<'info> {
    #[account(mut, seeds=[b"staking"], bump)]
    pub staking: Account<'info, Staking>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, seeds=[b"staker-info", user.key().as_ref()], bump)]
    pub staker_info: Account<'info, StakerInfo>,

    #[account(mut)]
    pub fctr_mint: Account<'info, Mint>,

    #[account(mut, token::authority=user, token::mint=fctr_mint)]
    pub user_fctr_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct SellFctr<'info> {
    #[account(mut, seeds=[b"staking"], bump)]
    pub staking: Account<'info, Staking>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, seeds=[b"staker-info", user.key().as_ref()], bump)]
    pub staker_info: Account<'info, StakerInfo>,

    #[account(mut)]
    pub fctr_mint: Account<'info, Mint>,

    #[account(mut, token::authority=staking, token::mint=fctr_mint)]
    pub service_fctr_account: Account<'info, TokenAccount>,

    #[account(mut, token::authority=user, token::mint=fctr_mint)]
    pub user_fctr_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct SellBcdev<'info> {
    #[account(mut, seeds=[b"staking"], bump)]
    pub staking: Account<'info, Staking>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, token::authority=user, token::mint=bcdev_mint)]
    pub user_bcdev_account: Account<'info, TokenAccount>,

    #[account(mut, seeds=[b"staker-info", user.key().as_ref()], bump)]
    pub staker_info: Account<'info, StakerInfo>,

    #[account(mut)]
    pub bcdev_mint: Account<'info, Mint>,

    #[account(mut, token::authority=staking, token::mint=bcdev_mint)]
    pub service_bcdev_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(confidant_address: Pubkey)]
pub struct Entrust<'info> {
    #[account(mut, seeds=[b"staking"], bump)]
    pub staking: Account<'info, Staking>,

    #[account(mut)]
    pub principal: Signer<'info>,

    #[account(mut, seeds=[b"staker-info", principal.key().as_ref()], bump)]
    pub principal_info: Account<'info, StakerInfo>,

    #[account(mut, seeds=[b"staker-info", confidant_address.as_ref()], bump)]
    pub confidant_info: Account<'info, StakerInfo>,

    #[account(mut)]
    pub fctr_mint: Account<'info, Mint>,

    #[account(mut, token::authority=principal, token::mint=fctr_mint)]
    pub principal_fctr_account: Account<'info, TokenAccount>,

    #[account(mut, token::mint=fctr_mint)]
    pub confidant_fctr_account: Account<'info, TokenAccount>,

    #[account(mut, token::authority=staking, token::mint=fctr_mint)]
    pub staking_fctr_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct Stop<'info>{
    #[account(mut, seeds=[b"staking"], bump)]
    pub staking: Account<'info, Staking>
}