use anchor_lang::prelude::*;
use anchor_spl::token::Token;

use crate::{
    errors::MyError,
    state::{EscrowAccount, VAULT_AUTHORITY_PDA_SEED, VaultAuthority},
    traits::*,
};

#[derive(Accounts)]
pub struct CancelByTaker<'info> {
    #[account(mut)]
    pub initializer: SystemAccount<'info>,
    #[account()]
    pub taker: Signer<'info>,
    #[account(
        mut, 
        seeds = [
            VAULT_AUTHORITY_PDA_SEED,
            initializer.key().as_ref(),
            taker.key().as_ref()
        ], 
        bump = vault_authority.bump,
        close = initializer
    )]
    pub vault_authority: Box<Account<'info, VaultAuthority>>,
    #[account(
        mut,
        constraint = escrow_account.initializer_key == *initializer.key @ MyError::InitializerPublicKeyMismatch,
        constraint = escrow_account.taker_key == *taker.key @ MyError::TakerPublicKeyMismatch,
        close = initializer // accountを実行後にcloseし、initializerにrentをreturnする　
    )]
    pub escrow_account: Box<Account<'info, EscrowAccount>>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, CancelByTaker<'info>>,
) -> Result<()> {
    let cancel_context = &CancelContext {
        accounts: &CancelContextAccounts {
            initializer: ctx.accounts.initializer.to_account_info().clone(),
            taker: ctx.accounts.taker.to_account_info().clone(),
            vault_authority: ctx.accounts.vault_authority.clone(),
            escrow_account: ctx.accounts.escrow_account.clone(),
            token_program: ctx.accounts.token_program.clone(),
            rent: ctx.accounts.rent.clone(),
        },
        remaining_accounts: ctx.remaining_accounts,
        program_id: &ctx.program_id,
    };
    cancel(cancel_context)?;
    Ok(())
}
