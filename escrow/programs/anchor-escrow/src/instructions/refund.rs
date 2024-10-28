use anchor_lang::prelude::*;
use anchor_spl::token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked, CloseAccount, close_account};

use crate::state::Escrow;

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    maker: Signer<'info>,
    mint_a: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
    )]
    maker_ata_a: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        // It specs the account receiving the remaining lamports back once have been closed, in this will be the maker
        // who was actually the one paying the rent to open this same account.
        close = maker,
        has_one = mint_a,
        has_one = maker,
        seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        // already calculated in make.rs escrow account so we dont need to spend extra compute units here 
        // but just pass through
        bump = escrow.bump, 

    )]
    pub escrow: Account<'info, Escrow>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
    )]
    vault: InterfaceAccount<'info, TokenAccount>,
    token_program: Interface<'info, TokenInterface>,
    system_program: Program<'info, System>,
}

impl<'info> Refund<'info> {
    // Function to empty the vault and send everything back to the maker account
    pub fn refund_and_close_vault(&mut self) -> Result<()> {
        
        // We need to calculate seeds here because the vault is a PDA so in order for our program
        // to be able to sign on behalf of the PDA, important to keeep in mind that PDA dont have a
        // associated Private Key so we need to pass the seeds. Solana runtime will take the seeds + 
        // program_id and after checking the address match it will then be able to sign
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"escrow",
            self.maker.key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump]
        ]];

        // the one who can sign in behalf of our vault is our escrow PDA as the vault authority
        // is the escrow and the escrow itself is a PDA.
        
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            to: self.maker_ata_a.to_account_info(),
            mint: self.mint_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, &signer_seeds);

        transfer_checked(cpi_context, self.vault.amount, self.mint_a.decimals)?;

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, &signer_seeds);

        close_account(cpi_context)?;
        
        Ok(())
    }
}