use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;
pub mod error;

use instructions::*;
use error::StakingError;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod solana_staking {    
    use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;
    use anchor_spl::token::{Transfer, self, MintTo, Burn};

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

    pub fn fund(ctx: Context<Fund>, amount: u64) -> Result<()> {
        let staking = &mut ctx.accounts.staking;

        require!(ctx.accounts.staking_fctr_account.mint == staking.fctr_mint, StakingError::InvalidTokenAccount);

        let staking_fctr_account = &mut ctx.accounts.staking_fctr_account;
        let owner_fctr_account = &mut ctx.accounts.owner_fctr_account;

        require!(ctx.accounts.owner.key() == staking.owner, StakingError::NotTheOwner);

        let staking_bump = staking.bump.to_le_bytes();
        let seeds = &[b"staking".as_ref(), staking_bump.as_ref()];
        let outer = [&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(), 
            Transfer {
                from: owner_fctr_account.to_account_info(),
                to: staking_fctr_account.to_account_info(), 
                authority: ctx.accounts.owner.to_account_info()
            }, 
            &outer);
        
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }

    pub fn register(ctx: Context<Register>) -> Result<()> {
        let staker_info = &mut ctx.accounts.staker_info;
        staker_info.staker = ctx.accounts.staker.key();
        staker_info.bump = *ctx.bumps.get("staker_info").unwrap();
        Ok(())
    }

    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        let staking = &mut ctx.accounts.staking;
        let staker_info = &mut ctx.accounts.staker_info;

        require!(ctx.accounts.staking_fctr_account.mint == staking.fctr_mint, StakingError::InvalidTokenAccount);
        require!(ctx.accounts.staker_fctr_account.mint == staking.fctr_mint, StakingError::InvalidTokenAccount);

        let staking_bump = staking.bump.to_le_bytes();
        let seeds = &[b"staking".as_ref(), staking_bump.as_ref()];
        let outer = [&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(), 
            Transfer {
                from: ctx.accounts.staker_fctr_account.to_account_info(),
                to: ctx.accounts.staking_fctr_account.to_account_info(), 
                authority: ctx.accounts.staker.to_account_info()
            }, 
            &outer);
        
        let amount = ctx.accounts.staker_fctr_account.amount;

        token::transfer(cpi_ctx, amount)?;

        let current_time = Clock::get().unwrap().unix_timestamp as u64;
        
        staker_info.stake_size += amount;

        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()>{
        let staker_info = &mut ctx.accounts.staker_info;
        let staking = &mut ctx.accounts.staking;
        let stake_round_id = staking.round_start_times.binary_search(&staker_info.stake_time).unwrap_or_else(|x| x-1);
        let round_start_time = staking.round_start_times.last().unwrap();
        let current_time = Clock::get().unwrap().unix_timestamp as u64;

        let start_round_offset = staker_info.stake_time - staking.round_start_times[stake_round_id];
        let end_round_offset = current_time - round_start_time;

        let bcdev_to_give = staker_info.stake_size ;
        Ok(())
    }

    pub fn start_round(ctx: Context<StartRound>, is_final: bool) -> Result<()> {
        let staking = &mut ctx.accounts.staking;
        let round = &mut ctx.accounts.round;

        let current_time = Clock::get().unwrap().unix_timestamp as u64;

        require!(current_time >= staking.last_round_deadline, StakingError::PrevRoundIsNotFinished);
        
        round.start_time = current_time;
        round.is_final = is_final;
        staking.rounds_num += 1;
        staking.last_round_deadline = current_time + staking.round_time;

        staking.round_start_times.push(current_time);

        Ok(())
    }

    pub fn buy_fctr(ctx: Context<BuyFctr>, amount: u64) -> Result<()> {
        let staking = &mut ctx.accounts.staking;
        let staker_info = &mut ctx.accounts.staker_info;

        require!(amount > 10, StakingError::TooFewAmount);
        require!(ctx.accounts.fctr_mint.key() == staking.fctr_mint, StakingError::InvalidMint);
        require!(!staker_info.is_in_trust_program, StakingError::CantBuyInTrustProgram);

        staker_info.bought_fctr += amount;
        staker_info.ftcr_amount += amount;

        let sol_to_withdraw = amount * LAMPORTS_PER_SOL / 109;

        **ctx.accounts.user.try_borrow_mut_lamports()? -= sol_to_withdraw;

        let staking_bump = staking.bump.to_le_bytes();
        let seeds = &[b"staking".as_ref(), staking_bump.as_ref()];
        let signer_seeds = [&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(), 
            MintTo { mint: ctx.accounts.fctr_mint.to_account_info(), to: ctx.accounts.user.to_account_info(), authority: staking.to_account_info() }, 
            &signer_seeds
        );

        token::mint_to(cpi_ctx, amount)?;

        staking.total_fctr_bought_by_users += amount;

        Ok(())
    }

    pub fn sell_fctr(ctx: Context<SellFctr>, amount: u64) -> Result<()> {
        let staking = &mut ctx.accounts.staking;
        let staker_info = &mut ctx.accounts.staker_info;

        require!(ctx.accounts.fctr_mint.key() == staking.fctr_mint, StakingError::InvalidMint);
        require!(staker_info.ftcr_amount >= amount && ctx.accounts.user_fctr_account.amount >= amount, StakingError::NotEnoughTokens);

        let sol_to_give = amount * LAMPORTS_PER_SOL / 101;

        **ctx.accounts.user.try_borrow_mut_lamports()? += sol_to_give;

        let staking_bump = staking.bump.to_le_bytes();
        let seeds = &[b"staking".as_ref(), staking_bump.as_ref()];
        let signer_seeds = [&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(), 
            Burn { mint: ctx.accounts.fctr_mint.to_account_info(), from: ctx.accounts.user_fctr_account.to_account_info(), authority: staking.to_account_info() }, 
            &signer_seeds
        );

        token::burn(cpi_ctx, amount)?;

        staking.total_fctr_sold_by_users += amount;

        Ok(())
    }

    pub fn sell_bcdev(ctx: Context<SellBcdev>, amount: u64) -> Result<()> {
        let staking = &mut ctx.accounts.staking;
        let staker_info = &mut ctx.accounts.staker_info;

        require!(ctx.accounts.bcdev_mint.key() == staking.bcdev_mint, StakingError::InvalidMint);
        require!(staker_info.bcdev_amount >= amount && ctx.accounts.user_bcdev_account.amount >= amount, StakingError::NotEnoughTokens);

        let sol_to_give = amount * LAMPORTS_PER_SOL / 11;

        **ctx.accounts.user.try_borrow_mut_lamports()? += sol_to_give;

        let staking_bump = staking.bump.to_le_bytes();
        let seeds = &[b"staking".as_ref(), staking_bump.as_ref()];
        let signer_seeds = [&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(), 
            Burn { mint: ctx.accounts.bcdev_mint.to_account_info(), from: ctx.accounts.user.to_account_info(), authority: staking.to_account_info() }, 
            &signer_seeds
        );

        token::burn(cpi_ctx, amount)?;

        staking.total_bcdev_sold_by_users += amount;

        Ok(())
    }

    pub fn entrust(ctx: Context<Entrust>, confidant: Pubkey) -> Result<()> {
        let principal_fctr_account = &mut ctx.accounts.principal_fctr_account;
        let confidant_fctr_account = &mut ctx.accounts.confidant_fctr_account;
        let principal_info = &mut ctx.accounts.principal_info;
        let confidant_info = &mut ctx.accounts.confidant_info;

        let amount = principal_fctr_account.amount / 2;

        require!(confidant_fctr_account.owner == confidant, StakingError::InvalidTokenAccount);
        require!(principal_fctr_account.amount >= amount && principal_info.ftcr_amount >= amount, StakingError::InvalidTokenAccount);
        require!(principal_fctr_account.amount >= principal_info.bought_fctr / 4 && principal_info.ftcr_amount >= principal_info.bought_fctr / 4, StakingError::InvalidAmountEntrusted);

        let staking = &mut ctx.accounts.staking;

        principal_info.is_in_trust_program = true;
        confidant_info.is_in_trust_program = true;

        let staking_bump = staking.bump.to_le_bytes();
        let seeds = &[b"staking".as_ref(), staking_bump.as_ref()];
        let outer = [&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(), 
            Transfer {
                from: principal_fctr_account.to_account_info(),
                to: confidant_fctr_account.to_account_info(), 
                authority: ctx.accounts.principal.to_account_info()
            }, 
            &outer);
        
        token::transfer(cpi_ctx, amount)?;

        principal_info.ftcr_amount -= amount;
        confidant_info.ftcr_amount += amount;

        Ok(())
    }

}

