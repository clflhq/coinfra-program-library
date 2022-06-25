use anchor_lang::prelude::*;
use anchor_spl::token::{Transfer, self, Token};

use crate::{utils::{assert_is_ata, assert_is_pda}, errors::MyError, state::{EscrowAccount, VaultAuthority}, traits::Common};

use crate::state::VAULT_AUTHORITY_PDA_SEED;

#[derive(Accounts)]
#[instruction(initializer_additional_sol_amount: u64, taker_additional_sol_amount: u64)]
pub struct Exchange<'info> {
    #[account(
        mut, 
        constraint = taker_additional_sol_amount as usize + escrow_account.taker_nft_token_accounts.len() > 0 @ MyError::NotProvidedTakerAssets,
        constraint = taker.to_account_info().try_lamports().unwrap() >= taker_additional_sol_amount @ MyError::TakerInsufficientFunds,
        constraint = taker_additional_sol_amount == escrow_account.taker_additional_sol_amount @ MyError::TakerAdditionalSolAmountMismatch
    )]
    pub taker: Signer<'info>,
    #[account(
        mut, // mutであることが必須
        constraint = initializer_additional_sol_amount as usize + escrow_account.initializer_nft_token_accounts.len() > 0 @ MyError::NotProvidedInitializerAssets,
        constraint = initializer.to_account_info().try_lamports().unwrap() >= initializer_additional_sol_amount @ MyError::InitializerInsufficientFunds,
        constraint = initializer_additional_sol_amount == escrow_account.initializer_additional_sol_amount @ MyError::InitializerAdditionalSolAmountMismatch
    )]
    pub initializer: SystemAccount<'info>,
    #[account(
        mut,
        constraint = escrow_account.initializer_key == *initializer.key @ MyError::InitializerPublicKeyMismatch,
        constraint = escrow_account.taker_key == *taker.key @ MyError::TakerPublicKeyMismatch,
        close = initializer // 関係なし Error: failed to send transaction: Transaction simulation failed: Error processing Instruction 0: instruction spent from the balance of an account it does not own
    )]
    pub escrow_account: Box<Account<'info, EscrowAccount>>,
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
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}


pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, Exchange<'info>>,
    initializer_additional_sol_amount: u64,
    _taker_additional_sol_amount: u64,
) -> Result<()> {
    msg!("start exchange");

    // remaining accountsの数の検証
    let initializer_nft_amount = ctx
        .accounts
        .escrow_account
        .initializer_nft_token_accounts
        .len();

    let taker_nft_amount = ctx.accounts.escrow_account.taker_nft_token_accounts.len();
    let initializer_nft_amount_count = initializer_nft_amount;
    let taker_nft_amount_count = taker_nft_amount;
    let remaining_accounts_count = (initializer_nft_amount_count * 3 + taker_nft_amount_count * 2)
        as usize
        + (initializer_nft_amount_count + taker_nft_amount_count) * 2 as usize;
    require_eq!(
        ctx.remaining_accounts.len(),
        remaining_accounts_count,
        MyError::NftAmountMismatch
    );

    for index in 0..initializer_nft_amount_count {
        let token_account = &ctx.remaining_accounts[index * 3 + 0];
        let vault_account = &ctx.remaining_accounts[index * 3 + 1];
        let mint_account = &ctx.remaining_accounts[index * 3 + 2];

        // initializerのvaultがあるToken Accountの検証
        assert_is_ata(
            token_account,
            ctx.accounts.initializer.key,
            mint_account,
            false,
        )?;

        // Vaultの検証
        assert_is_pda(
            &token_account,
            ctx.accounts.escrow_account.vault_account_bumps[index],
            vault_account,
            &ctx.accounts.vault_authority.key(),
            ctx.program_id,
        )?;
    }

    for index in 0..taker_nft_amount_count {
        let token_account = &ctx.remaining_accounts[initializer_nft_amount_count * 3 + index * 2];
        let mint_account = &ctx.remaining_accounts[initializer_nft_amount_count * 3 + index * 2 + 1];

        // initializerのvaultがないToken Accountの検証
        assert_is_ata(
            token_account,
            ctx.accounts.initializer.key,
            mint_account,
            false,
        )?;
    }

    let offset = initializer_nft_amount_count * 3 + taker_nft_amount_count * 2;

    for index in 0..initializer_nft_amount_count + taker_nft_amount_count {
        // takerのToken Accountの検証
        let token_account = &ctx.remaining_accounts[offset + index * 2];
        let mint_account = &ctx.remaining_accounts[offset + index * 2 + 1];

        if index >= initializer_nft_amount_count {
            assert_is_ata(token_account, ctx.accounts.taker.key, mint_account, true)?;
        } else {
            assert_is_ata(token_account, ctx.accounts.taker.key, mint_account, false)?;
        }
    }

    // initializerがtokenをget
    for index in 0..taker_nft_amount_count {
        token::transfer(
            ctx.accounts.into_transfer_to_initializer_context(
                &ctx.remaining_accounts[offset + initializer_nft_amount_count * 2 + index * 2],
                &ctx.remaining_accounts[initializer_nft_amount_count * 3 + index * 2],
            ),
            1,
        )?;
    }

    // initializerがsolをget
    // walletからの引き出しなら以下のようにやる
    // taker mutでOK　Error: failed to send transaction: Transaction simulation failed: Error processing Instruction 0: Cross-program invocation with unauthorized signer or writable account
    let ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.taker.key(),
        &ctx.accounts.initializer.key(), // sends to event host pda
        ctx.accounts.escrow_account.taker_additional_sol_amount,
    );
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.taker.to_account_info().clone(),
            ctx.accounts.initializer.to_account_info().clone(),
        ],
    )?;

    // takerがtokenをget
    for index in 0..initializer_nft_amount_count {
        let vault_account = &ctx.remaining_accounts[index * 3 + 1];
        let taker_nft_token_account = &ctx.remaining_accounts[offset + index * 2];

        token::transfer(
            ctx.accounts
                .into_transfer_to_taker_context(vault_account, taker_nft_token_account)
                .with_signer(&[&[
                    VAULT_AUTHORITY_PDA_SEED,
                    ctx.accounts.initializer.key().as_ref(),
                    ctx.accounts.taker.key().as_ref(),
                    &[ctx.accounts.vault_authority.bump],
                ]]),
            1,
        )?;

        token::close_account(
            ctx.accounts
                .into_close_context(vault_account)
                .with_signer(&[&[
                    VAULT_AUTHORITY_PDA_SEED,
                    ctx.accounts.initializer.key().as_ref(),
                    ctx.accounts.taker.key().as_ref(),
                    &[ctx.accounts.vault_authority.bump],
                ]]),
        )?;
    }

    // takerがsolをget
    // PDAからの引き出しなら以下のようにやる
    // 　Error: failed to send transaction: Transaction simulation failed: Error processing Instruction 0: instruction changed the balance of a read-only account
    // 以下の構文はPDAのときだけしか使えない
    **ctx
        .accounts
        .escrow_account
        .to_account_info()
        .try_borrow_mut_lamports()? -= initializer_additional_sol_amount;
    **ctx.accounts.taker.try_borrow_mut_lamports()? += initializer_additional_sol_amount;

    /*
    //　vault_sol_accountから齋藤に送る
    // programのownerと齋藤の一致を確認する
    // metaplexのwithdrawが参考になるかも

    // vault_sol_accountをcloseする
    msg!("end exchange");
    */
    Ok(())
}

impl<'info> Exchange<'info> {
    fn into_transfer_to_initializer_context(
        &self,
        taker_nft_token_account: &AccountInfo<'info>,
        initializer_nft_token_account: &AccountInfo<'info>,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: taker_nft_token_account.clone(),
            to: initializer_nft_token_account.clone(),
            authority: self.taker.to_account_info().clone(),
        };
        CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
    }

    fn into_transfer_to_taker_context(
        &self,
        vault_account: &AccountInfo<'info>,
        taker_nft_token_account: &AccountInfo<'info>,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: vault_account.clone(),
            to: taker_nft_token_account.clone(),
            authority: self.vault_authority.to_account_info().clone(),
        };
        CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
    }
}

impl<'info> Common<'info> for Exchange<'info> {
    fn vault_authority(&self) -> &Box<Account<'info, VaultAuthority>> {
        return &self.vault_authority;
    }

    fn initializer(&self) -> &AccountInfo<'info> {
        return &self.initializer;
    }

    fn token_program(&self) -> &AccountInfo<'info> {
        return &self.token_program;
    }
}