use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Mint, SetAuthority, TokenAccount, Transfer};
use spl_token::instruction::AuthorityType;

declare_id!("FRd6p3td6akTgfhHgJZHyhVeyYUhGWiM9dApVucDGer2");

/*
Controller
*/
#[program]
pub mod nft_barter {
    use anchor_lang::solana_program::entrypoint::ProgramResult;

    use super::*;

    const ESCROW_PDA_SEED: &[u8] = b"escrow";

    pub fn initialize(
        ctx: Context<Initialize>,
        initializer_amount: u64,
        initializer_additional_sol_amount: u64,
        taker_amount: u64,
        taker_additional_sol_amount: u64,
    ) -> ProgramResult {
        msg!("start initialize");
        ctx.accounts.escrow_account.initializer_key = *ctx.accounts.initializer.key;
        ctx.accounts
            .escrow_account
            .initializer_deposit_token_account = *ctx
            .accounts
            .initializer_deposit_token_account
            .to_account_info()
            .key;
        ctx.accounts
            .escrow_account
            .initializer_receive_token_account = *ctx
            .accounts
            .initializer_receive_token_account
            .to_account_info()
            .key;
        ctx.accounts.escrow_account.initializer_amount = initializer_amount;
        ctx.accounts
            .escrow_account
            .initializer_additional_sol_amount = initializer_additional_sol_amount;
        ctx.accounts.escrow_account.taker_key = *ctx.accounts.taker.key;
        ctx.accounts.escrow_account.taker_amount = taker_amount;
        ctx.accounts.escrow_account.taker_additional_sol_amount = taker_additional_sol_amount;
        ctx.accounts.escrow_account.vault_account_bump = *ctx.bumps.get("vault_account").unwrap();
        ctx.accounts.vault_sol_account.bump = *ctx.bumps.get("vault_sol_account").unwrap();

        /*
        msg!(
            "ctx.accounts.escrow_account.vault_account_bump {}",
            ctx.accounts.escrow_account.vault_account_bump
        );
        msg!(
            "ctx.accounts.vault_sol_account.bump {}",
            ctx.accounts.vault_sol_account.bump
        );*/

        let (vault_authority, _vault_authority_bump) =
            Pubkey::find_program_address(&[ESCROW_PDA_SEED], ctx.program_id);

        // set_authorityをしないと　assert.ok(_vault.owner.equals(vault_authority_pda));　がパスしない
        token::set_authority(
            ctx.accounts.into_set_authority_context(),
            AuthorityType::AccountOwner,
            Some(vault_authority),
        )?;

        // NFTの移動
        token::transfer(
            ctx.accounts.into_transfer_to_pda_context(),
            ctx.accounts.escrow_account.initializer_amount,
        )?;

        // Error: failed to send transaction: Transaction simulation failed: Error processing Instruction 1: invalid program argument
        /*
        token::set_authority(
            ctx.accounts.into_set_authority_context2(),
            AuthorityType::AccountOwner,
            Some(vault_authority),
        )?;*/
        /*
        token::transfer(
            ctx.accounts.into_transfer_to_pda_context2(),
            initializer_additional_sol_amount,
        )?; */

        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.initializer.key(),
            &ctx.accounts.vault_sol_account.key(), // sends to event host pda
            initializer_additional_sol_amount,
        );

        // account_infosにctx.accounts.system_program.to_account_info()はなくてもいい
        // invoke_signedの必要もなし だが、account_infoはいずれにしても必須
        // ctx.accounts.initializer.clone() ctx.accounts.vault_sol_account.clone()は必須 ないとAn account required by the instruction is missing
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.initializer.clone(),
                ctx.accounts.vault_sol_account.to_account_info(),
            ],
        )?;

        // 以下はPDAのときだけしか使えない
        /*
                        **ctx.accounts.initializer.try_borrow_mut_lamports()? -= initializer_additional_sol_amount;
                        **ctx.accounts.vault_sol_account.try_borrow_mut_lamports()? +=
                            initializer_additional_sol_amount;
        */
        msg!(
            "vault_sol_account {}",
            &ctx.accounts.vault_sol_account.key()
        );
        msg!(
            "&ctx.accounts.vault_account.to_account_info().owner {}",
            &ctx.accounts.vault_account.to_account_info().owner
        );
        msg!(
            "&ctx.accounts.vault_sol_account.to_account_info().owner {}",
            &ctx.accounts.vault_sol_account.to_account_info().owner
        );

        msg!("end initialize");
        Ok(())
    }

    pub fn exchange(ctx: Context<Exchange>) -> ProgramResult {
        msg!("start exchange");

        let (_vault_authority, vault_authority_bump) =
            Pubkey::find_program_address(&[ESCROW_PDA_SEED], ctx.program_id);
        let authority_seeds = &[&ESCROW_PDA_SEED[..], &[vault_authority_bump]];
        // initializerがtokenをget
        token::transfer(
            ctx.accounts.into_transfer_to_initializer_context(),
            ctx.accounts.escrow_account.taker_amount,
        )?;

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
                ctx.accounts.taker.clone(),
                ctx.accounts.initializer.clone(),
                // ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // takerがtokenをget
        token::transfer(
            ctx.accounts
                .into_transfer_to_taker_context()
                .with_signer(&[&authority_seeds[..]]),
            ctx.accounts.escrow_account.initializer_amount,
        )?;

        // takerがsolをget
        // PDAからの引き出しなら以下のようにやる
        // 　Error: failed to send transaction: Transaction simulation failed: Error processing Instruction 0: instruction changed the balance of a read-only account
        **ctx
            .accounts
            .vault_sol_account
            .to_account_info()
            .try_borrow_mut_lamports()? -= ctx
            .accounts
            .escrow_account
            .initializer_additional_sol_amount;
        **ctx.accounts.taker.try_borrow_mut_lamports()? += ctx
            .accounts
            .escrow_account
            .initializer_additional_sol_amount;

        // token accountのclose
        token::close_account(
            ctx.accounts
                .into_close_context()
                .with_signer(&[&authority_seeds[..]]),
        )?;

        msg!("end exchange");

        Ok(())
    }

    pub fn cancel_by_initializer(ctx: Context<CancelByInitializer>) -> ProgramResult {
        msg!("start cancel_by_initializer");

        let (_vault_authority, vault_authority_bump) =
            Pubkey::find_program_address(&[ESCROW_PDA_SEED], ctx.program_id);
        let authority_seeds = &[&ESCROW_PDA_SEED[..], &[vault_authority_bump]];

        // NFTをinitializerに戻す
        token::transfer(
            ctx.accounts
                .into_transfer_to_initializer_context()
                .with_signer(&[&authority_seeds[..]]),
            ctx.accounts.escrow_account.initializer_amount,
        )?;

        // 追加のsolをinitializerに戻す
        **ctx
            .accounts
            .vault_sol_account
            .to_account_info()
            .try_borrow_mut_lamports()? -= ctx
            .accounts
            .escrow_account
            .initializer_additional_sol_amount;
        **ctx.accounts.initializer.try_borrow_mut_lamports()? += ctx
            .accounts
            .escrow_account
            .initializer_additional_sol_amount;

        token::close_account(
            ctx.accounts
                .into_close_context()
                .with_signer(&[&authority_seeds[..]]),
        )?;

        msg!("end cancel_by_initializer");
        Ok(())
    }

    pub fn cancel_by_taker(ctx: Context<CancelByTaker>) -> ProgramResult {
        msg!("start cancel_by_taker");

        let (_vault_authority, vault_authority_bump) =
            Pubkey::find_program_address(&[ESCROW_PDA_SEED], ctx.program_id);
        let authority_seeds = &[&ESCROW_PDA_SEED[..], &[vault_authority_bump]];

        // NFTをinitializerに戻す
        token::transfer(
            ctx.accounts
                .into_transfer_to_initializer_context()
                .with_signer(&[&authority_seeds[..]]),
            ctx.accounts.escrow_account.initializer_amount,
        )?;

        // 追加のsolをinitializerに戻す
        **ctx
            .accounts
            .vault_sol_account
            .to_account_info()
            .try_borrow_mut_lamports()? -= ctx
            .accounts
            .escrow_account
            .initializer_additional_sol_amount;
        **ctx.accounts.initializer.try_borrow_mut_lamports()? += ctx
            .accounts
            .escrow_account
            .initializer_additional_sol_amount;

        token::close_account(
            ctx.accounts
                .into_close_context()
                .with_signer(&[&authority_seeds[..]]),
        )?;

        msg!("end cancel_by_taker");

        Ok(())
    }
}

/*
Model
*/
#[derive(Accounts)]
#[instruction(initializer_amount: u64, initializer_additional_sol_amount: u64, taker_amount: u64, taker_additional_sol_amount: u64)]
pub struct Initialize<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
    pub initializer: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub taker: AccountInfo<'info>,
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        seeds = [b"token-seed".as_ref()],
        bump,
        payer = initializer,
        token::mint = mint,
        token::authority = initializer,
    )]
    pub vault_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = initializer_deposit_token_account.amount >= initializer_amount
    )]
    pub initializer_deposit_token_account: Account<'info, TokenAccount>,
    pub initializer_receive_token_account: Account<'info, TokenAccount>,
    // ここをmutにすると、Error: 3012: The program expected this account to be already initialized
    #[account(
        init,
        seeds = [b"vault-sol-account".as_ref()], // initializer.key().as_ref()を加えるとError: failed to send transaction: Transaction simulation failed: Error processing Instruction 1: Cross-program invocation with unauthorized signer or writable account
        bump,
        payer = initializer,
        space = 100, // spaceが足りないとError: failed to send transaction: Transaction simulation failed: Error processing Instruction 1: Program failed to complete
        constraint = initializer.lamports() >= initializer_additional_sol_amount,
        constraint = taker.lamports() >= taker_additional_sol_amount,
    )]
    pub vault_sol_account: Account<'info, VaultSolAccount>, // 手数料と追加のsolを入れる箱 ここBoxにしてもError: 3007: The given account is owned by a different program than expected
    #[account(zero)]
    pub escrow_account: Box<Account<'info, EscrowAccount>>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Exchange<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
    pub taker: AccountInfo<'info>,
    #[account(mut)]
    pub taker_deposit_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub taker_receive_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub initializer_deposit_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub initializer_receive_token_account: Box<Account<'info, TokenAccount>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub initializer: AccountInfo<'info>,
    #[account(
        mut,
        constraint = escrow_account.taker_amount <= taker_deposit_token_account.amount,
        constraint = escrow_account.initializer_deposit_token_account == *initializer_deposit_token_account.to_account_info().key,
        constraint = escrow_account.initializer_receive_token_account == *initializer_receive_token_account.to_account_info().key,
        constraint = escrow_account.initializer_key == *initializer.key,
        close = initializer // 関係なし Error: failed to send transaction: Transaction simulation failed: Error processing Instruction 0: instruction spent from the balance of an account it does not own
    )]
    pub escrow_account: Box<Account<'info, EscrowAccount>>,
    // close = initializerを加えるとError: failed to send transaction: Transaction simulation failed: Error processing Instruction 0: instruction spent from the balance of an account it does not own
    #[account(mut, seeds = [b"token-seed".as_ref()], bump = escrow_account.vault_account_bump)]
    pub vault_account: Box<Account<'info, TokenAccount>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub vault_authority: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,
    // close = initializerをいれると残高がずれる
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, seeds = [b"vault-sol-account".as_ref()], bump = vault_sol_account.bump, constraint = taker.lamports() >= escrow_account.taker_additional_sol_amount)]
    pub vault_sol_account: Box<Account<'info, VaultSolAccount>>, // BoxにしてもError: 3012: The program expected this account to be already initialized
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CancelByInitializer<'info> {
    // signerはトランザクションに署名したことをcheckするので、実際には、initializerによるキャンセルとtakerによるキャンセルをわける必要あり
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
    pub initializer: AccountInfo<'info>,
    #[account(mut, seeds = [b"token-seed".as_ref()], bump = escrow_account.vault_account_bump)]
    pub vault_account: Account<'info, TokenAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub vault_authority: AccountInfo<'info>,
    #[account(mut)]
    pub initializer_deposit_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_sol_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = escrow_account.initializer_key == *initializer.key,
        constraint = escrow_account.initializer_deposit_token_account == *initializer_deposit_token_account.to_account_info().key,
        close = initializer // accountを実行後にcloseし、initializerにrentをreturnする　
    )]
    pub escrow_account: Box<Account<'info, EscrowAccount>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CancelByTaker<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub initializer: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
    pub taker: AccountInfo<'info>,
    #[account(mut, seeds = [b"token-seed".as_ref()], bump = escrow_account.vault_account_bump)]
    pub vault_account: Account<'info, TokenAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub vault_authority: AccountInfo<'info>,
    #[account(mut)]
    pub initializer_deposit_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_sol_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = escrow_account.taker_key == *taker.key,
        constraint = escrow_account.initializer_deposit_token_account == *initializer_deposit_token_account.to_account_info().key,
        close = initializer // accountを実行後にcloseし、initializerにrentをreturnする　
    )]
    pub escrow_account: Box<Account<'info, EscrowAccount>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,
}

#[account]
pub struct EscrowAccount {
    pub initializer_key: Pubkey,
    pub initializer_deposit_token_account: Pubkey,
    pub initializer_receive_token_account: Pubkey,
    pub initializer_amount: u64,
    pub initializer_additional_sol_amount: u64,
    pub taker_key: Pubkey,
    pub taker_amount: u64,
    pub taker_additional_sol_amount: u64,
    pub vault_account_bump: u8,
    pub vault_sol_account_bump: u8,
}

#[account]
#[derive(Default)]
pub struct VaultSolAccount {
    pub bump: u8,
}

/*
Util
*/
impl<'info> Initialize<'info> {
    fn into_transfer_to_pda_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self
                .initializer_deposit_token_account
                .to_account_info()
                .clone(),
            to: self.vault_account.to_account_info().clone(),
            authority: self.initializer.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_set_authority_context(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.vault_account.to_account_info().clone(),
            current_authority: self.initializer.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_set_authority_context2(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.vault_sol_account.to_account_info().clone(),
            current_authority: self.initializer.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
}

impl<'info> CancelByInitializer<'info> {
    fn into_transfer_to_initializer_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        // 読んだ
        let cpi_accounts = Transfer {
            from: self.vault_account.to_account_info().clone(),
            to: self
                .initializer_deposit_token_account
                .to_account_info()
                .clone(),
            authority: self.vault_authority.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_close_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        // 読んだがExchangeと同じなので共通化したいところ
        let cpi_accounts = CloseAccount {
            account: self.vault_account.to_account_info().clone(),
            destination: self.initializer.clone(),
            authority: self.vault_authority.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
}

impl<'info> CancelByTaker<'info> {
    fn into_transfer_to_initializer_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        // 読んだ
        let cpi_accounts = Transfer {
            from: self.vault_account.to_account_info().clone(),
            to: self
                .initializer_deposit_token_account
                .to_account_info()
                .clone(),
            authority: self.vault_authority.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_close_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        // 読んだがExchangeと同じなので共通化したいところ
        let cpi_accounts = CloseAccount {
            account: self.vault_account.to_account_info().clone(),
            destination: self.initializer.clone(),
            authority: self.vault_authority.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
}

impl<'info> Exchange<'info> {
    fn into_transfer_to_initializer_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        // 読んだ
        let cpi_accounts = Transfer {
            from: self.taker_deposit_token_account.to_account_info().clone(),
            to: self
                .initializer_receive_token_account
                .to_account_info()
                .clone(),
            authority: self.taker.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_transfer_to_taker_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        // 読んだ
        let cpi_accounts = Transfer {
            from: self.vault_account.to_account_info().clone(),
            to: self.taker_receive_token_account.to_account_info().clone(),
            authority: self.vault_authority.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_close_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        // 読んだ
        let cpi_accounts = CloseAccount {
            account: self.vault_account.to_account_info().clone(),
            destination: self.initializer.clone(), // initializerに権限を返却する
            authority: self.vault_authority.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
}
