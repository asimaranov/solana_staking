use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;
pub mod error;

use instructions::*;
use error::StakingError;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod solana_staking {    
    use std::cmp::min;

    use anchor_lang::solana_program::{native_token::LAMPORTS_PER_SOL, system_instruction, program::invoke};
    use anchor_spl::token::{Transfer, self, MintTo, Burn, TokenAccount};

    use crate::state::StakerInfo;

    use super::*;

    const ONE_FCTR: u64 = 10_u64.pow(12);
    const ONE_BCDEV: u64 = 10_u64.pow(18);

    pub fn initialize(ctx: Context<Initialize>, round_time: u64, fctr_mint: Pubkey, bcdev_mint: Pubkey, proof_signer: Pubkey) -> Result<()> {
        let staking = &mut ctx.accounts.staking;
        staking.round_time = round_time;
        staking.owner = ctx.accounts.owner.key();
        staking.bump = *ctx.bumps.get("staking").unwrap();
        staking.fctr_mint = fctr_mint;
        staking.bcdev_mint = bcdev_mint;
        staking.proof_signer = proof_signer;
        Ok(())
    }

    pub fn fund(ctx: Context<Fund>, amount: u64) -> Result<()> {
        let staking = &mut ctx.accounts.staking;
        let donation_transfer_instruction = system_instruction::transfer(&ctx.accounts.owner.key(), &staking.key(), amount);

        invoke(&donation_transfer_instruction, &[
            ctx.accounts.owner.to_account_info(),
            staking.to_account_info()
        ])?;

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let staking = &mut ctx.accounts.staking;
        let current_time = Clock::get().unwrap().unix_timestamp as u64;

        require!(staking.finished && current_time - staking.finish_time >= staking.round_time * 2, StakingError::CantWithdraw);

        **staking.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.owner.to_account_info().try_borrow_mut_lamports()? += amount;

        Ok(())
    }

    pub fn register(ctx: Context<Register>) -> Result<()> {
        let staking = &mut ctx.accounts.staking;
        let staker_info = &mut ctx.accounts.staker_info;
        require!(!staking.finished, StakingError::StakingFinished);
        require!(ctx.accounts.proof_signer.key() == staking.proof_signer, StakingError::StakingFinished);

        staker_info.staker = ctx.accounts.staker.key();
        staker_info.user_rpr = 1;
        staker_info.bump = *ctx.bumps.get("staker_info").unwrap();
        Ok(())
    }

    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        let staking = &mut ctx.accounts.staking;
        let staker_info = &mut ctx.accounts.staker_info;
        staker_info.is_staked = true;

        require!(!staking.finished, StakingError::StakingFinished);
        require!(ctx.accounts.fctr_mint.key() == staking.fctr_mint, StakingError::InvalidTokenAccount);
        require!(ctx.accounts.staker_fctr_account.mint == staking.fctr_mint, StakingError::InvalidTokenAccount);

        let staking_bump = staking.bump.to_le_bytes();
        let seeds = &[b"staking".as_ref(), staking_bump.as_ref()];
        let signer_seeds = [&seeds[..]];
        let amount = min(ctx.accounts.staker_fctr_account.amount, staker_info.ftcr_amount);

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(), 
            Burn { mint: ctx.accounts.fctr_mint.to_account_info(), from: ctx.accounts.staker_fctr_account.to_account_info(), authority: ctx.accounts.staker.to_account_info() }, 
            &signer_seeds
        );
        token::burn(cpi_ctx, amount)?;
        
        let current_time = Clock::get().unwrap().unix_timestamp as u64;
        if staker_info.last_update_timestamp > 0 {  // Already staked
            let period = current_time - staker_info.last_update_timestamp;
            staker_info.pending_bcdev_reward += period * staker_info.stake_size * staker_info.user_rpr;
        }

        staker_info.last_update_timestamp = current_time;
        staker_info.stake_size += amount;
        staker_info.ftcr_amount = 0;

        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()>{
        let staker_info = &mut ctx.accounts.staker_info;
        let staking = &mut ctx.accounts.staking;
        let current_time = Clock::get().unwrap().unix_timestamp as u64;

        require!(!staking.finished, StakingError::StakingFinished);
        require!(ctx.accounts.bcdev_mint.key() == staking.bcdev_mint, StakingError::InvalidMint);
        require!(ctx.accounts.fctr_mint.key() == staking.fctr_mint, StakingError::InvalidMint);
        require!(current_time - staker_info.stake_time >= staking.round_time, StakingError::CantUnstakeInThisVeryRound);

        let current_time = Clock::get().unwrap().unix_timestamp as u64;

        let period = current_time - staker_info.last_update_timestamp;        
        staker_info.pending_bcdev_reward += period * staker_info.stake_size * staker_info.user_rpr;

        let staking_bump = staking.bump.to_le_bytes();
        let seeds = &[b"staking".as_ref(), staking_bump.as_ref()];
        let signer_seeds = [&seeds[..]];

        let mut amount_to_give_to_user = staker_info.stake_size;

        for (i, principal_accounts) in ctx.remaining_accounts.chunks_exact(3).enumerate() {
            let pricipal_fctr_account = &mut Account::<TokenAccount>::try_from(&principal_accounts[0])?;
            let pricipal_bcdev_account = &mut Account::<TokenAccount>::try_from(&principal_accounts[1])?;
            let pricipal_info = &mut Account::<StakerInfo>::try_from(&principal_accounts[2])?;

            require!(pricipal_fctr_account.mint.key() == staking.fctr_mint, StakingError::InvalidMint);
            require!(pricipal_bcdev_account.mint.key() == staking.bcdev_mint, StakingError::InvalidMint);
            require!(pricipal_fctr_account.owner == staker_info.principals[i].principal, StakingError::InvalidTokenAccountOwner);
            require!(pricipal_bcdev_account.owner == staker_info.principals[i].principal, StakingError::InvalidTokenAccountOwner);
            require!(pricipal_info.staker == staker_info.principals[i].principal, StakingError::InvalidTokenAccountOwner);

            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(), 
                MintTo { mint: ctx.accounts.fctr_mint.to_account_info(), to: ctx.accounts.staker_fctr_account.to_account_info(), authority: staking.to_account_info() }, 
                &signer_seeds
            );
            token::mint_to(cpi_ctx, staker_info.principals[i].amount)?;
            pricipal_info.ftcr_amount += staker_info.principals[i].amount;
            pricipal_info.ftcr_amount += staker_info.principals[i].amount;
            amount_to_give_to_user -= staker_info.principals[i].amount;

            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(), 
                MintTo { mint: ctx.accounts.fctr_mint.to_account_info(), to: ctx.accounts.staker_fctr_account.to_account_info(), authority: staking.to_account_info() }, 
                &signer_seeds
            );
            token::mint_to(cpi_ctx, staker_info.pending_bcdev_reward * staker_info.principals[i].amount / staker_info.stake_size)?;
        }

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(), 
            MintTo { mint: ctx.accounts.bcdev_mint.to_account_info(), to: ctx.accounts.staker_bcdev_account.to_account_info(), authority: staking.to_account_info() }, 
            &signer_seeds
        );

        let reward_to_give_to_user = (staker_info.pending_bcdev_reward as u128 * amount_to_give_to_user as u128 / staker_info.stake_size as u128) as u64;
        token::mint_to(cpi_ctx, reward_to_give_to_user)?;

        staker_info.bcdev_amount += reward_to_give_to_user;

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(), 
            MintTo { mint: ctx.accounts.fctr_mint.to_account_info(), to: ctx.accounts.staker_fctr_account.to_account_info(), authority: staking.to_account_info() }, 
            &signer_seeds
        );
        token::mint_to(cpi_ctx, amount_to_give_to_user)?;

        staker_info.ftcr_amount = amount_to_give_to_user;
        staker_info.pending_bcdev_reward = 0;
        staker_info.stake_size = 0;

        Ok(())
    }

    pub fn buy_fctr(ctx: Context<BuyFctr>, amount: u64) -> Result<()> {
        let staking = &mut ctx.accounts.staking;
        let staker_info = &mut ctx.accounts.staker_info;

        require!(!staking.finished, StakingError::StakingFinished);
        require!(amount >= 10 * ONE_FCTR, StakingError::TooFewAmount);
        require!(ctx.accounts.fctr_mint.key() == staking.fctr_mint, StakingError::InvalidMint);
        require!(!staker_info.is_in_trust_program, StakingError::CantBuyInTrustProgram);

        let current_time = Clock::get().unwrap().unix_timestamp as u64;
        let period = current_time - staker_info.last_update_timestamp;
        staker_info.pending_bcdev_reward += period * staker_info.stake_size * staker_info.user_rpr;
        staker_info.last_update_timestamp = current_time;

        staker_info.bought_fctr += amount;
        staker_info.ftcr_amount += amount;

        //fctr_amount / ONE_FCTR = 101 * sol_amount / LAMPORTS_PER_SOL; => 
        let sol_to_take = amount / (ONE_FCTR / LAMPORTS_PER_SOL) / 109; 

        let transfer_instruction = system_instruction::transfer(&ctx.accounts.user.key(), &staking.key(), sol_to_take);

        invoke(&transfer_instruction, &[
            ctx.accounts.user.to_account_info(),
            staking.to_account_info()
        ])?;
        
        let staking_bump = staking.bump.to_le_bytes();
        let seeds = &[b"staking".as_ref(), staking_bump.as_ref()];
        let signer_seeds = [&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(), 
            MintTo { mint: ctx.accounts.fctr_mint.to_account_info(), to: ctx.accounts.user_fctr_account.to_account_info(), authority: staking.to_account_info() }, 
            &signer_seeds
        );

        token::mint_to(cpi_ctx, amount)?;

        staking.total_fctr_bought_by_users += amount;

        Ok(())
    }

    pub fn sell_fctr(ctx: Context<SellFctr>, amount: u64) -> Result<()> {
        let staking = &mut ctx.accounts.staking;
        let staker_info = &mut ctx.accounts.staker_info;

        require!(!staking.finished, StakingError::StakingFinished);
        require!(ctx.accounts.fctr_mint.key() == staking.fctr_mint, StakingError::InvalidMint);
        require!(staker_info.ftcr_amount >= amount && ctx.accounts.user_fctr_account.amount >= amount, StakingError::NotEnoughTokens);

        //fctr_amount / ONE_FCTR = 101 * sol_amount / LAMPORTS_PER_SOL; => 
        let sol_to_give = amount / (ONE_FCTR / LAMPORTS_PER_SOL) / 101; 

        require!(**staking.to_account_info().lamports.borrow() >= sol_to_give, StakingError::NotEnoughFunds);

        msg!("{:?} {:?} {}", staking.to_account_info().lamports, ctx.accounts.user.to_account_info().lamports, sol_to_give);

        **staking.to_account_info().try_borrow_mut_lamports()? -= sol_to_give;
        **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? += sol_to_give;

        let staking_bump = staking.bump.to_le_bytes();
        let seeds = &[b"staking".as_ref(), staking_bump.as_ref()];
        let signer_seeds = [&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(), 
            Burn { mint: ctx.accounts.fctr_mint.to_account_info(), from: ctx.accounts.user_fctr_account.to_account_info(), authority: ctx.accounts.user.to_account_info() }, 
            &signer_seeds
        );

        token::burn(cpi_ctx, amount)?;

        staking.total_fctr_sold_by_users += amount;
        staker_info.ftcr_amount -= amount;

        Ok(())
    }

    pub fn sell_bcdev(ctx: Context<SellBcdev>, amount: u64) -> Result<()> {
        let staking = &mut ctx.accounts.staking;
        let staker_info = &mut ctx.accounts.staker_info;

        require!(!staking.finished, StakingError::StakingFinished);
        require!(ctx.accounts.bcdev_mint.key() == staking.bcdev_mint, StakingError::InvalidMint);
        require!(staker_info.bcdev_amount >= amount && ctx.accounts.user_bcdev_account.amount >= amount, StakingError::NotEnoughTokens);

        let sol_to_give = amount / (ONE_BCDEV / LAMPORTS_PER_SOL) / 11; 

        **staking.to_account_info().try_borrow_mut_lamports()? -= sol_to_give;
        **ctx.accounts.user.try_borrow_mut_lamports()? += sol_to_give;

        let staking_bump = staking.bump.to_le_bytes();
        let seeds = &[b"staking".as_ref(), staking_bump.as_ref()];
        let signer_seeds = [&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(), 
            Burn { mint: ctx.accounts.bcdev_mint.to_account_info(), from: ctx.accounts.user_bcdev_account.to_account_info(), authority: ctx.accounts.user.to_account_info() }, 
            &signer_seeds
        );

        token::burn(cpi_ctx, amount)?;

        staking.total_bcdev_sold_by_users += amount;
        staker_info.bcdev_amount -= amount;


        Ok(())
    }

    pub fn entrust(ctx: Context<Entrust>, confidant: Pubkey) -> Result<()> {
        let principal_fctr_account = &mut ctx.accounts.principal_fctr_account;

        let principal_info = &mut ctx.accounts.principal_info;
        let confidant_info = &mut ctx.accounts.confidant_info;
        let staking = &ctx.accounts.staking;

        let amount = principal_fctr_account.amount / 2;

        require!(!staking.finished, StakingError::StakingFinished);
        require!(confidant_info.principals.len() <= 4, StakingError::TooMuchPrincipals);
        require!(principal_fctr_account.amount >= amount && principal_info.ftcr_amount >= amount, StakingError::InvalidTokenAccount);
        require!(principal_fctr_account.amount >= principal_info.bought_fctr / 4 && principal_info.ftcr_amount >= principal_info.bought_fctr / 4, StakingError::InvalidAmountEntrusted);
        require!(principal_info.ftcr_amount / 2 <= confidant_info.ftcr_amount && confidant_info.ftcr_amount <= principal_info.ftcr_amount * 2, StakingError::InvalidDepositDiff);

        principal_info.is_in_trust_program = true;
        confidant_info.is_in_trust_program = true;

        let staking_bump = staking.bump.to_le_bytes();
        let seeds = &[b"staking".as_ref(), staking_bump.as_ref()];
        let signer_seeds = [&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(), 
            Burn { mint: ctx.accounts.fctr_mint.to_account_info(), from: principal_fctr_account.to_account_info(), authority: ctx.accounts.principal.to_account_info() }, 
            &signer_seeds
        );

        token::burn(cpi_ctx, amount)?;

        if confidant_info.is_staked {
            confidant_info.stake_size += amount;
        } else {
            confidant_info.ftcr_amount += amount;
        }
        confidant_info.principals.push(state::EntrustInfo { principal: principal_info.staker, amount });
        principal_info.user_rpr += 2;
        principal_info.ftcr_amount -= amount;
        confidant_info.ftcr_amount += amount;

        Ok(())
    }

    pub fn demand_back(ctx: Context<DemandBack>, confidant: Pubkey) -> Result<()> {
        let principal_info = &mut ctx.accounts.principal_info;
        let confididant_info = &mut ctx.accounts.confidant_info;
        let staking = &mut ctx.accounts.staking;

        let entrust_info = confididant_info.principals.iter_mut().find(|x|x.principal == principal_info.staker).ok_or(StakingError::NoSuchPrincipal)?;
        let amount_to_take = entrust_info.amount;
        
        require!(amount_to_take > 0, StakingError::NothingToDemandBack);

        entrust_info.amount = 0;
        confididant_info.ftcr_amount -= amount_to_take;
        principal_info.ftcr_amount += amount_to_take;

        let staking_bump = staking.bump.to_le_bytes();
        let seeds = &[b"staking".as_ref(), staking_bump.as_ref()];
        let signer_seeds = [&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(), 
            MintTo { mint: ctx.accounts.fctr_mint.to_account_info(), to: ctx.accounts.principal_fctr_account.to_account_info(), authority: ctx.accounts.fctr_mint.to_account_info() }, 
            &signer_seeds
        );
        token::mint_to(cpi_ctx, amount_to_take)?;

        if confididant_info.is_staked {
            confididant_info.stake_size -= amount_to_take;
        } else {
            confididant_info.ftcr_amount += amount_to_take;
        }

        return Ok(())
    }

    pub fn stop(ctx: Context<Stop>) -> Result<()> {
        let staking = &mut ctx.accounts.staking;
        let current_time = Clock::get().unwrap().unix_timestamp as u64;

        staking.finished = true;
        staking.finish_time = current_time;
        return Ok(());
    }

}

