use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use mpl_token_metadata::ID as mpl_metadata_program_id;
use solana_program::{
    clock::{Clock, UnixTimestamp},
    program::{invoke, invoke_signed},
};
use spl_token::ID as spl_token_program_id;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod staking {
    use super::*;

    pub fn process_stake(ctx: Context<Stake>) -> Result<()> {
        let clock = Clock::get()?;

        msg!("CPI to approve ix on token program");
        invoke(
            &spl_token::instruction::approve(
                &spl_token_program_id,
                &ctx.accounts.nft_token_account.key(),
                &ctx.accounts.program_authority.key(),
                &ctx.accounts.user.key(),
                &[&ctx.accounts.user.key()],
                1,
            )?,
            &[
                ctx.accounts.nft_token_account.to_account_info(),
                ctx.accounts.program_authority.to_account_info(),
                ctx.accounts.user.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
            ],
        )?;

        msg!("CPI to invoke freeze ix on token program");
        invoke_signed(
            &mpl_token_metadata::instruction::freeze_delegated_account(
                mpl_metadata_program_id,
                ctx.accounts.program_authority.key(),
                ctx.accounts.nft_token_account.key(),
                ctx.accounts.nft_edition.key(),
                ctx.accounts.nft_mint.key(),
            ),
            &[
                ctx.accounts.program_authority.to_account_info(),
                ctx.accounts.nft_token_account.to_account_info(),
                ctx.accounts.nft_edition.to_account_info(),
                ctx.accounts.nft_mint.to_account_info(),
                ctx.accounts.metadata_program.to_account_info(),
            ],
            &[&[
                b"authority",
                &[*ctx.bumps.get("program_authority").unwrap()],
            ]],
        )?;

        let account_data = &mut ctx.accounts.stake_state;
        account_data.token_account = ctx.accounts.nft_token_account.key();
        account_data.user_pubkey = ctx.accounts.user.key();
        account_data.stake_state = true;
        account_data.stake_start_time = clock.unix_timestamp;
        account_data.last_stake_redeem = clock.unix_timestamp;
        account_data.is_initialized = true;

        Ok(())
    }

    pub fn process_redeem(ctx: Context<Redeem>) -> Result<()> {
        Ok(())
    }

    pub fn process_unstake(ctx: Context<Unstake>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub nft_token_account: Account<'info, TokenAccount>,
    pub nft_mint: Account<'info, Mint>,
    /// CHECK
    pub nft_edition: AccountInfo<'info>,
    #[account(
        init,
        seeds = [user.key().as_ref(), nft_token_account.key().as_ref()],
        bump,
        payer = user,
        space = 1000 // TODO: update
    )]
    pub stake_state: Account<'info, UserStakeInfo>,
    #[account(
        init,
        seeds = [b"authority"],
        bump,
        payer = user,
        space = 1000 // TODO: update
    )]
    /// CHECK:
    pub program_authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    /// CHECK:
    pub metadata_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Redeem {}

#[derive(Accounts)]
pub struct Unstake {}

#[account]
pub struct UserStakeInfo {
    pub token_account: Pubkey,
    pub stake_start_time: UnixTimestamp,
    pub last_stake_redeem: UnixTimestamp,
    pub user_pubkey: Pubkey,
    pub stake_state: bool,
    pub is_initialized: bool,
}
