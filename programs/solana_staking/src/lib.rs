use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;
pub mod error;

use instructions::*;
use error::StakingError;


declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");


#[program]
pub mod solana_staking {    
    use anchor_spl::token::{Transfer, self};

    use super::*;

    pub fn initialize(ctx: Context<Initialize>, round_time: u64, fctr_mint: Pubkey, bcdev_mint: Pubkey) -> Result<()> {
        let staking = &mut ctx.accounts.staking;
        staking.round_time = round_time;
        staking.owner = ctx.accounts.owner.key();
        staking.bump = *ctx.bumps.get("staking").unwrap();
        staking.fctr_mint = fctr_mint;
        staking.bcdev_mint = bcdev_mint;
        Ok(())
    }

    pub fn register(ctx: Context<Register>) -> Result<()> {
        let staker_info = &mut ctx.accounts.staker_info;
        staker_info.staker = ctx.accounts.staker.key();
        staker_info.bump = *ctx.bumps.get("staker_info").unwrap();
        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, amount: u64 ) -> Result<()>{
        let staking = &mut ctx.accounts.staking;
        let staker_info = &mut ctx.accounts.staker_info;

        let staking_bump = staking.bump.to_le_bytes();
        let seeds = &[b"staking".as_ref(), staking_bump.as_ref()];
        let outer = [&seeds[..]];

        require!(ctx.accounts.staking_fctr_account.mint == staking.fctr_mint, StakingError::InvalidTokenAccount);
        require!(ctx.accounts.staker_fctr_account.mint == staking.fctr_mint, StakingError::InvalidTokenAccount);

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(), 
            Transfer {
                from: ctx.accounts.staker_fctr_account.to_account_info(),
                to: ctx.accounts.staking_fctr_account.to_account_info(), 
                authority: ctx.accounts.staker.to_account_info()
            }, 
            &outer);

        token::transfer(cpi_ctx, amount)?;
        staker_info.stake_size += amount;

        Ok(())
    }
}

