use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked}};

use crate::state::Escrow;

#[derive(Accounts)]
// Macro that allows maker initialize several escrow accounts
#[instruction(seed: u64)]
pub struct Make<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    // Token deposited by the maker into the vault
    pub mint_a: InterfaceAccount<'info, Mint>,
    // Token to be received by the Maker from the Taker
    pub mint_b: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
    )]
    // We dont initialize this account because it is the token being deposit by the maker so
    // it esentially means that the maker already posses this token account hence dont need to be initialized
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init,
        payer = maker,
        // seeds used here to ensure maker account is the one associated to this escrow account
        seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()],
        // anchor in this case will try to calculate the canonical bump, each iteration it needs to 
        // calculate the can bump (255, 254, 253,  252 -n) is comsuming compute units
        // once calculated here we can after pass the calc bump to the escrow account in the refund.ts for example
        bump, 
        space = 8 + Escrow::INIT_SPACE,
    )]
    pub escrow: Account<'info, Escrow>,
    #[account(
        // we need to init this account as is being just created to be managed by the vault
        init,
        payer = maker,
        associated_token::mint = mint_a,
        // Escrow account will be the authority managing the vault funds as expected
        associated_token::authority = escrow,
    )]
    // In this vault account case, a token account is being created (init) as the vault 
    // had no that token previously this account was initialized
    pub vault: InterfaceAccount<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    // why Token Program ? We will be doing cpi to the token program in order to transfer SPL Tokens  
    pub token_program: Interface<'info, TokenInterface>,
    // Every single time you initialize an account you need to pass the System Program, it will be 
    // actually the first invoked and once initialized will be passing the ownership to the token program
    pub system_program: Program<'info, System>,
}

// MakeBumps: Anchor under the hood will automatically create this by checking the canonical bumps
// calculated previously and will save them in a strcut named "ContextName+Bumps"    

impl<'info> Make<'info> {
    pub fn init_escrow(&mut self, seed: u64, receive: u64, bumps: &MakeBumps) -> Result<()> {
        self.escrow.set_inner(Escrow {
            seed,
            maker: self.maker.key(),
            mint_a: self.mint_a.key(),
            mint_b: self.mint_b.key(),
            receive,
            bump: bumps.escrow, // Saving a lot of compute units - calculated before (canonical)
        });

        Ok(())
    }

    pub fn deposit(&mut self, deposit: u64) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = TransferChecked {
            from: self.maker_ata_a.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.maker.to_account_info(),
            mint: self.mint_a.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer_checked(cpi_ctx, deposit, self.mint_a.decimals)?;

        Ok(())
    }
}