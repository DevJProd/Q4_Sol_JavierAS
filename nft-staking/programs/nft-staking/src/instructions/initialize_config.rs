use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

use crate::state::StakeConfig;

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init, 
        payer = admin,
        seeds = [b"config".as_ref()],
        bump,
        space = 8 + StakeConfig::INIT_SPACE,
    )]
    // Account to store the main staking configuration info
    pub config: Account<'info, StakeConfig>,
    #[account(
        init_if_needed,
        payer = admin,
        // PDA seeds created with rewards and config key to make unique for each config account
        seeds = [b"rewards".as_ref(), config.key().as_ref()],
        bump,
        mint::decimals = 6,
        // Only autohrized entity for minting is config account
        mint::authority = config, 
    )]
    // Token mint account used for distributing rewards to stakers
    pub rewards_mint: Account<'info, Mint>,
    // Interaction with Solana System program for account creation
    pub system_program: Program<'info, System>,
    // Interaction with SPL Token Program for minting tokens
    pub token_program: Program<'info, Token>,
}

impl<'info> InitializeConfig<'info> {
    // Setting parameters for Staking configuration
    pub fn initialize_config(&mut self, points_per_stake: u8, max_stake: u8, freeze_period: u32, bumps: &InitializeConfigBumps) -> Result<()> {
        self.config.set_inner(StakeConfig {
            points_per_stake,
            max_stake,
            freeze_period,
            rewards_bump: bumps.rewards_mint,
            bump: bumps.config,
        });

        Ok(())
    }
}