use anchor_lang::prelude::*;
use anchor_spl::token::Token;

use crate::{
    errors::MyError,
    state::{EscrowAccount, VAULT_AUTHORITY_PDA_SEED},
    traits::*,
};
// cancelの前に何かトランザクションを差し込まれても不利な取引が成立することはないのでcancelの場合のfrontrunningの考慮は不要
#[derive(Accounts)]
#[instruction(vault_authority_bump: u8)]
pub struct CancelByInitializer<'info> {
    // signerはトランザクションに署名したことをcheckするので、実際には、initializerによるキャンセルとtakerによるキャンセルをわける必要あり
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account()]
    pub taker: SystemAccount<'info>,
    #[account(
        seeds = [
            VAULT_AUTHORITY_PDA_SEED,
            initializer.key().as_ref(),
            taker.key().as_ref()
        ],
        bump = vault_authority_bump,
    )]
    pub vault_authority: SystemAccount<'info>,
    #[account(
        mut,
        constraint = escrow_account.initializer_key == *initializer.key @ MyError::InitializerPublicKeyMismatch,
        constraint = escrow_account.taker_key == *taker.key @ MyError::TakerPublicKeyMismatch,
        close = initializer // accountを実行後にcloseし、initializerにrentをreturnする　
    )]
    pub escrow_account: Box<Account<'info, EscrowAccount>>,
    pub token_program: Program<'info, Token>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, CancelByInitializer<'info>>,
    vault_authority_bump: u8,
) -> Result<()> {
    let cancel_context = &CancelContext {
        accounts: &CancelContextAccounts {
            initializer: ctx.accounts.initializer.to_account_info().clone(),
            taker: ctx.accounts.taker.to_account_info().clone(),
            vault_authority: ctx.accounts.vault_authority.clone(),
            escrow_account: ctx.accounts.escrow_account.clone(),
            token_program: ctx.accounts.token_program.clone(),
        },
        remaining_accounts: ctx.remaining_accounts,
        program_id: &ctx.program_id,
        vault_authority_bump,
    };
    cancel(cancel_context)?;
    Ok(())
}
