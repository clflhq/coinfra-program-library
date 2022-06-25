use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Token, Transfer};

use crate::{
    state::{EscrowAccount, VAULT_AUTHORITY_PDA_SEED},
    utils::{assert_is_ata, assert_is_pda},
};

/*　難関　ここで以下の仕様にあわせてlifetimeのa b cを設定しないとlifetimeエラー
pub struct Context<'a, 'b, 'c, 'info, T> {
    /// Currently executing program id.
    pub program_id: &'a Pubkey,
    /// Deserialized accounts.
    pub accounts: &'b mut T,
    /// Remaining accounts given but not deserialized or validated.
    /// Be very careful when using this directly.
    pub remaining_accounts: &'c [AccountInfo<'info>],
    /// Bump seeds found during constraint validation. This is provided as a
    /// convenience so that handlers don't have to recalculate bump seeds or
    /// pass them in as arguments.
    pub bumps: BTreeMap<String, u8>,
} */
pub struct CancelContext<'a, 'b, 'c, 'info> {
    pub program_id: &'a Pubkey,
    pub accounts: &'b CancelContextAccounts<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub remaining_accounts: &'c [AccountInfo<'info>],
    pub vault_authority_bump: u8,
}

pub fn cancel(cancel_context: &CancelContext) -> Result<()> {
    msg!("start cancel");

    let ctx = cancel_context;

    // NFTをinitializerに戻す
    let initializer_nft_amount_count = &ctx.remaining_accounts.len() / 3;
    for index in 0..initializer_nft_amount_count {
        let initializer_nft_token_account = &ctx.remaining_accounts[index * 3];
        let vault_account = &ctx.remaining_accounts[index * 3 + 1];
        let mint_account = &ctx.remaining_accounts[index * 3 + 2];

        // token accountの検証
        let _account = assert_is_ata(
            initializer_nft_token_account,
            ctx.accounts.initializer.key,
            mint_account,
            false,
        )?;

        // PDAの検証
        assert_is_pda(
            initializer_nft_token_account,
            ctx.accounts.escrow_account.vault_account_bumps[index],
            vault_account,
            &ctx.accounts.vault_authority.key(),
            ctx.program_id,
        )?;

        token::transfer(
            ctx.accounts
                .into_transfer_to_initializer_context(vault_account, initializer_nft_token_account)
                .with_signer(&[&[
                    VAULT_AUTHORITY_PDA_SEED,
                    ctx.accounts.initializer.key().as_ref(),
                    ctx.accounts.taker.key().as_ref(),
                    &[ctx.vault_authority_bump],
                ]]),
            1,
        )?;

        // NFTのtoken accountをcloseする
        token::close_account(
            ctx.accounts
                .into_close_context(vault_account)
                .with_signer(&[&[
                    VAULT_AUTHORITY_PDA_SEED,
                    ctx.accounts.initializer.key().as_ref(),
                    ctx.accounts.taker.key().as_ref(),
                    &[ctx.vault_authority_bump],
                ]]),
        )?;
    }

    // 追加のsolをinitializerに戻す
    // 最難関： solanaのbugで金額を動かすのはinto_close_contextの後にする必要がある Ref: https://discord.com/channels/889577356681945098/889584618372734977/915190505002921994
    let initializer_additional_sol_amount = ctx
        .accounts
        .escrow_account
        .initializer_additional_sol_amount;
    **ctx
        .accounts
        .escrow_account
        .to_account_info()
        .try_borrow_mut_lamports()? -= initializer_additional_sol_amount;
    **ctx.accounts.initializer.try_borrow_mut_lamports()? += initializer_additional_sol_amount; // ここを減らそうとすると　 Error: failed to send transaction: Transaction simulation failed: Error processing Instruction 0: instruction spent from the balance of an account it does not own

    msg!("end cancel");
    Ok(())
}

pub struct CancelContextAccounts<'info> {
    /// CHECK: This is not dangerous because we have already validated it in the account context
    pub initializer: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we have already validated it in the account context
    pub taker: AccountInfo<'info>,
    pub vault_authority: SystemAccount<'info>,
    pub escrow_account: Box<Account<'info, EscrowAccount>>,
    pub token_program: Program<'info, Token>,
}

impl<'info> Cancel<'info> for CancelContextAccounts<'info> {
    fn vault_authority(&self) -> &AccountInfo<'info> {
        return &self.vault_authority;
    }

    fn token_program(&self) -> &AccountInfo<'info> {
        return &self.token_program;
    }
}

impl<'info> Common<'info> for CancelContextAccounts<'info> {
    fn vault_authority(&self) -> &AccountInfo<'info> {
        return &self.vault_authority;
    }

    fn initializer(&self) -> &AccountInfo<'info> {
        return &self.initializer;
    }

    fn token_program(&self) -> &AccountInfo<'info> {
        return &self.token_program;
    }
}

pub trait Cancel<'info> {
    fn vault_authority(&self) -> &AccountInfo<'info>;
    fn token_program(&self) -> &AccountInfo<'info>;

    fn into_transfer_to_initializer_context(
        &self,
        vault_account: &AccountInfo<'info>,
        initializer_nft_token_account: &AccountInfo<'info>,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: vault_account.clone(),
            to: initializer_nft_token_account.clone(),
            authority: self.vault_authority().clone(),
        };
        CpiContext::new(self.token_program().clone(), cpi_accounts)
    }
}

pub trait Common<'info> {
    fn vault_authority(&self) -> &AccountInfo<'info>;
    fn initializer(&self) -> &AccountInfo<'info>;
    fn token_program(&self) -> &AccountInfo<'info>;

    fn into_close_context(
        &self,
        vault_account: &AccountInfo<'info>,
    ) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        let cpi_accounts = CloseAccount {
            account: vault_account.clone(),
            destination: self.initializer().clone(), // initializerに権限を返却する
            authority: self.vault_authority().clone(),
        };
        CpiContext::new(self.token_program().clone(), cpi_accounts)
    }
}
