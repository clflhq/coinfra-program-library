use crate::utils::{assert_is_ata, assert_owned_by};
use crate::{errors::MyError, utils::assert_is_pda};

use anchor_lang::{prelude::*, solana_program};
use anchor_spl::token::{self, CloseAccount, SetAuthority, Transfer};
use spl_token::instruction::AuthorityType;

pub mod errors;
pub mod utils;

declare_id!("FRd6p3td6akTgfhHgJZHyhVeyYUhGWiM9dApVucDGer2");

/*
Controller
*/
const VAULT_AUTHORITY_PDA_SEED: &[u8] = b"vault-authority";

// Context<'_, '_, '_, 'infoとContext<'info, 'info, 'info, 'infoが混じっているとthese two types are declared with different lifetimes　but data from `accounts` flows into `accounts` here
#[program]
pub mod nft_barter {
    use anchor_lang::solana_program::{
        program::{invoke, invoke_signed},
        program_pack::Pack,
        system_instruction,
    };

    use super::*;

    // pub fn initialize(
    //     ctx: Context<Initialize>,だと以下エラー
    // but data from `ctx` flows into `ctx` here
    // lifetime mismatch these two types are declared with different lifetimes...
    pub fn initialize<'info>(
        ctx: Context<'_, '_, '_, 'info, Initialize<'info>>,
        initializer_additional_sol_amount: u64, // こいつはstateで使っているから変数の先にもってきている
        taker_additional_sol_amount: u64, // こいつはstateで使っているから変数の先にもってきている
        initializer_nft_amount: u8,
        taker_nft_amount: u8,
        vault_account_bumps: Vec<u8>,
    ) -> Result<()> {
        msg!("start initialize");

        // check account infos
        // vault authority の検証は後半でしているから不要
        assert_owned_by(&ctx.accounts.initializer, &ctx.accounts.system_program.key)?;
        assert_owned_by(&ctx.accounts.taker, &ctx.accounts.system_program.key)?;
        require_keys_eq!(ctx.accounts.token_program.key(), spl_token::id());

        // 両方0であることはありえない
        require_neq!(
            initializer_additional_sol_amount as usize + initializer_nft_amount as usize,
            0,
            MyError::NotProvidedInitializerAssets
        );
        require_neq!(
            taker_additional_sol_amount as usize + taker_nft_amount as usize,
            0,
            MyError::NotProvidedTakerAssets
        );

        // remaining_accountsの数の検証
        let initializer_nft_amount_count = initializer_nft_amount as usize;
        let taker_nft_amount_count = taker_nft_amount as usize;
        let offset = initializer_nft_amount_count * 3 as usize;
        let remaining_accounts_count = offset as usize + taker_nft_amount_count * 2 as usize; // initializerはtoken accountとbump takerは直接initializerに払い出すのでtoken accountのみ
        require_eq!(
            ctx.remaining_accounts.len(),
            remaining_accounts_count,
            MyError::NftAmountMismatch
        );

        // vault_account_bumpsの数の検証
        require_eq!(
            vault_account_bumps.len(),
            initializer_nft_amount_count,
            MyError::VaultAccountBumpsMismatch
        );

        // SOLの保有状況の検証
        require_gte!(
            ctx.accounts.initializer.try_lamports()?,
            initializer_additional_sol_amount,
            MyError::InitializerInsufficientFunds
        );

        // takerにはvaultがないため、token accountとmintだけ
        for index in 0..taker_nft_amount_count {
            let token_account = &ctx.remaining_accounts[offset + index * 2];
            let mint_account = &ctx.remaining_accounts[offset + index * 2 + 1];

            // Token Accountの検証
            assert_is_ata(token_account, ctx.accounts.taker.key, mint_account, true)?;

            ctx.accounts
                .escrow_account
                .taker_nft_token_accounts
                .push(token_account.key());
        }

        let (vault_authority, _vault_authority_bump) = Pubkey::find_program_address(
            &[
                VAULT_AUTHORITY_PDA_SEED,
                ctx.accounts.initializer.key().as_ref(),
                ctx.accounts.taker.key().as_ref(),
            ],
            ctx.program_id,
        );

        // 3で割ってあまり0にtoken account 1にvault account 2にmint account mint accountがないとspl_token::instruction::initialize_accountが無理
        for index in 0..initializer_nft_amount_count {
            // Token Accountの検証
            let token_account = &ctx.remaining_accounts[index * 3];
            let vault_account = &ctx.remaining_accounts[index * 3 + 1];
            let mint_account = &ctx.remaining_accounts[index * 3 + 2];

            assert_is_ata(
                token_account,
                ctx.accounts.initializer.key,
                mint_account,
                true,
            )?;

            // 渡されたPDAの検証
            let vault_account_bump = vault_account_bumps[index];
            let vault_pda = Pubkey::create_program_address(
                &[
                    b"vault-account",
                    token_account.key().as_ref(),
                    &[vault_account_bump],
                ],
                &ctx.program_id,
            )
            .unwrap();

            require_keys_eq!(
                vault_pda,
                vault_account.key(),
                MyError::PdaPublicKeyMismatch
            );

            let create_account_ix = system_instruction::create_account(
                &ctx.accounts.initializer.key(),
                &vault_account.key(),
                ctx.accounts
                    .rent
                    .minimum_balance(spl_token::state::Account::LEN),
                spl_token::state::Account::LEN as u64,
                &spl_token::id(),
            );

            // ただのinvokeだとError: failed to send transaction: Transaction simulation failed: Error processing Instruction 1: Cross-program invocation with unauthorized signer or writable account
            invoke_signed(
                &create_account_ix,
                // ctx.accounts.token_program.clone(), token programはなくても通る
                &[
                    ctx.accounts.initializer.clone(),
                    vault_account.clone(), // これないとError: failed to send transaction: Transaction simulation failed: Error processing Instruction 1: An account required by the instruction is missing
                ],
                &[&[
                    b"vault-account",
                    token_account.key().as_ref(),
                    &[vault_account_bump],
                ]],
            )?;

            // 以下はpub fn initialize_account<'a, 'b, 'c, 'info>(を使って書き換えられそう
            let initialize_account_ix = spl_token::instruction::initialize_account(
                &spl_token::id(),
                &vault_account.key(),
                &mint_account.key(),
                &ctx.accounts.initializer.key(),
            )?;

            // ここはinvoke_signedでなくても動く ちなみにinvoke_signedでも動く
            invoke(
                &initialize_account_ix,
                &[
                    ctx.accounts.initializer.clone(),
                    vault_account.clone(),
                    mint_account.clone(), // なくすとmissing error
                    ctx.accounts.rent.to_account_info().clone(), // 超難関ポイント　rentがないと動かない　Error: failed to send transaction: Transaction simulation failed: Error processing Instruction 1: An account required by the instruction is missing
                ],
            )?;

            // set_authorityをしないとError: failed to send transaction: Transaction simulation failed: Error processing Instruction 1: Cross-program invocation with unauthorized signer or writable account?
            // classでdeserializeするときは、token::authority = initializerを指定しているので、initializerがownerになっている　今回はPDAをわたしているだけなので、signが必要
            token::set_authority(
                ctx.accounts.into_set_authority_context(vault_account),
                AuthorityType::AccountOwner,
                Some(vault_authority),
            )?;

            // NFTをinitializerからvaultに移す
            token::transfer(
                ctx.accounts
                    .into_transfer_to_pda_context(&token_account, &vault_account),
                1,
            )?;

            ctx.accounts
                .escrow_account
                .initializer_nft_token_accounts
                .push(token_account.key());
        }

        // ここで入れたtoken accountはexchangeのときに検証
        ctx.accounts.escrow_account.initializer_key = *ctx.accounts.initializer.key;
        ctx.accounts
            .escrow_account
            .initializer_additional_sol_amount = initializer_additional_sol_amount;
        ctx.accounts.escrow_account.taker_key = *ctx.accounts.taker.key;
        ctx.accounts.escrow_account.taker_additional_sol_amount = taker_additional_sol_amount;
        ctx.accounts.escrow_account.vault_account_bumps = vault_account_bumps;

        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.initializer.key(),
            &ctx.accounts.escrow_account.key(), // sends to event host pda
            initializer_additional_sol_amount,
        );

        // account_infosにctx.accounts.system_program.to_account_info()はなくてもいい
        // invoke_signedの必要もなし だが、account_infoはいずれにしても必須
        // ctx.accounts.initializer.clone() ないとAn account required by the instruction is missing
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.initializer.clone(),
                ctx.accounts.escrow_account.to_account_info().clone(),
            ],
        )?;

        msg!("end initialize");
        Ok(())
    }

    pub fn exchange<'info>(
        ctx: Context<'_, '_, '_, 'info, Exchange<'info>>,
        initializer_additional_sol_amount: u64, // こいつはstateで使っているから変数の先にもってきている
        taker_additional_sol_amount: u64, // こいつはstateで使っているから変数の先にもってきている
    ) -> Result<()> {
        msg!("start exchange");

        // check account infos
        assert_owned_by(&ctx.accounts.initializer, &ctx.accounts.system_program.key)?;
        assert_owned_by(&ctx.accounts.taker, &ctx.accounts.system_program.key)?;
        //assert_owned_by(&ctx.accounts.vault_authority, &ctx.program_id)?;
        require_keys_eq!(ctx.accounts.token_program.key(), spl_token::id());

        let initializer_nft_amount = ctx
            .accounts
            .escrow_account
            .initializer_nft_token_accounts
            .len();

        let taker_nft_amount = ctx.accounts.escrow_account.taker_nft_token_accounts.len();

        // 両方0であることはありえない
        require_neq!(
            initializer_additional_sol_amount as usize + initializer_nft_amount as usize,
            0,
            MyError::NotProvidedInitializerAssets
        );
        require_neq!(
            taker_additional_sol_amount as usize + taker_nft_amount as usize,
            0,
            MyError::NotProvidedTakerAssets
        );

        // 入力された値の検証
        require_eq!(
            initializer_additional_sol_amount,
            ctx.accounts
                .escrow_account
                .initializer_additional_sol_amount,
            MyError::InitializerAdditionalSolAmountMismatch
        );
        require_eq!(
            taker_additional_sol_amount,
            ctx.accounts.escrow_account.taker_additional_sol_amount,
            MyError::TakerAdditionalSolAmountMismatch
        );

        // remaining accountsの数の検証
        let initializer_nft_amount_count = initializer_nft_amount as usize;
        let taker_nft_amount_count = taker_nft_amount as usize;
        let remaining_accounts_count = (initializer_nft_amount_count * 3
            + taker_nft_amount_count * 2) as usize
            + (initializer_nft_amount_count + taker_nft_amount_count) * 2 as usize;
        require_eq!(
            ctx.remaining_accounts.len(),
            remaining_accounts_count,
            MyError::NftAmountMismatch
        );

        let (vault_authority, vault_authority_bump) = Pubkey::find_program_address(
            &[
                VAULT_AUTHORITY_PDA_SEED,
                ctx.accounts.initializer.key().as_ref(),
                ctx.accounts.taker.key().as_ref(),
            ],
            ctx.program_id,
        );

        // vault authorityのpubkey一致の確認
        require_keys_eq!(
            vault_authority,
            ctx.accounts.vault_authority.key(),
            MyError::PdaPublicKeyMismatch
        );

        // SOLの保有状況の検証
        require_gte!(
            ctx.accounts.taker.try_lamports()?,
            taker_additional_sol_amount,
            MyError::TakerInsufficientFunds
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
                &vault_authority,
                ctx.program_id,
            )?;
        }

        for index in 0..taker_nft_amount_count {
            // initializerのvaultがないToken Accountの検証
            assert_is_ata(
                &ctx.remaining_accounts[initializer_nft_amount_count * 3 + index * 2],
                ctx.accounts.initializer.key,
                &ctx.remaining_accounts[initializer_nft_amount_count * 3 + index * 2 + 1],
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
            &[ctx.accounts.taker.clone(), ctx.accounts.initializer.clone()],
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
                        &[vault_authority_bump],
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
                        &[vault_authority_bump],
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

    pub fn cancel_by_initializer<'info>(
        ctx: Context<'_, '_, '_, 'info, CancelByInitializer<'info>>,
    ) -> Result<()> {
        let cancel_context = &CancelContext {
            accounts: &CancelContextAccounts {
                initializer: ctx.accounts.initializer.clone(),
                taker: ctx.accounts.taker.clone(),
                vault_authority: ctx.accounts.vault_authority.clone(),
                escrow_account: ctx.accounts.escrow_account.clone(),
                token_program: ctx.accounts.token_program.clone(),
            },
            remaining_accounts: ctx.remaining_accounts,
            program_id: &ctx.program_id,
        };
        cancel(cancel_context)?;
        Ok(())
    }

    pub fn cancel_by_taker<'info>(
        ctx: Context<'_, '_, '_, 'info, CancelByTaker<'info>>,
    ) -> Result<()> {
        let cancel_context = &CancelContext {
            accounts: &CancelContextAccounts {
                initializer: ctx.accounts.initializer.clone(),
                taker: ctx.accounts.taker.clone(),
                vault_authority: ctx.accounts.vault_authority.clone(),
                escrow_account: ctx.accounts.escrow_account.clone(),
                token_program: ctx.accounts.token_program.clone(),
            },
            remaining_accounts: ctx.remaining_accounts,
            program_id: &ctx.program_id,
        };
        cancel(cancel_context)?;
        Ok(())
    }
}

/*
Model
*/
#[derive(Accounts)]
#[instruction(initializer_additional_sol_amount: u64, taker_additional_sol_amount: u64, initializer_nft_amount: u8, taker_nft_amount: u8, vault_account_bumps: Vec<u8>)]
pub struct Initialize<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
    pub initializer: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub taker: AccountInfo<'info>,
    // account(zero)でuninitializedを保証できるので、ts側でinitしようとするとなぜかError: 3003: Failed to deserialize the account　エラー　調べる限りspace問題なのでrustでspaceを指定することで解決
    #[account(init, payer = initializer, space = 8 // internal anchor discriminator 
        + 32 // initializerKey
        + 8 // initializerAdditionalSolAmount
        + 4 + 32 * initializer_nft_amount as usize // initializerNftTokenAccounts
        + 32 // takerKey
        + 8 // takerAdditionalSolAmount
        + 4 + 32 * taker_nft_amount as usize // takerNftTokenAccounts
        + 4 + vault_account_bumps.len() // vault_account_bumps vec
    )]
    pub escrow_account: Box<Account<'info, EscrowAccount>>, // ownerはFRd6p3td6akTgfhHgJZHyhVeyYUhGWiM9dApVucDGer2
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(initializer_additional_sol_amount: u64, taker_additional_sol_amount: u64)]
pub struct Exchange<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
    pub taker: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub initializer: AccountInfo<'info>,
    #[account(
        mut,
        constraint = escrow_account.initializer_key == *initializer.key,
        constraint = escrow_account.taker_key == *taker.key, // 関係なし Error: Invalid arguments: taker not provided.
        close = initializer // 関係なし Error: failed to send transaction: Transaction simulation failed: Error processing Instruction 0: instruction spent from the balance of an account it does not own
    )]
    pub escrow_account: Box<Account<'info, EscrowAccount>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub vault_authority: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

// cancelの前に何かトランザクションを差し込まれても不利な取引が成立することはないのでcancelの場合のfrontrunningの考慮は不要
#[derive(Accounts)]
pub struct CancelByInitializer<'info> {
    // signerはトランザクションに署名したことをcheckするので、実際には、initializerによるキャンセルとtakerによるキャンセルをわける必要あり
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
    pub initializer: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub taker: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub vault_authority: AccountInfo<'info>,
    #[account(
        mut,
        constraint = escrow_account.initializer_key == *initializer.key,
        constraint = escrow_account.taker_key == *taker.key,
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
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub vault_authority: AccountInfo<'info>,
    #[account(
        mut,
        constraint = escrow_account.initializer_key == *initializer.key,
        constraint = escrow_account.taker_key == *taker.key,
        close = initializer // accountを実行後にcloseし、initializerにrentをreturnする　
    )]
    pub escrow_account: Box<Account<'info, EscrowAccount>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,
}

#[account]
pub struct EscrowAccount {
    pub initializer_key: Pubkey,
    pub initializer_additional_sol_amount: u64,
    pub initializer_nft_token_accounts: Vec<Pubkey>,
    pub taker_key: Pubkey,
    pub taker_additional_sol_amount: u64,
    pub taker_nft_token_accounts: Vec<Pubkey>,
    pub vault_account_bumps: Vec<u8>,
}

/*
Util
*/
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
struct CancelContext<'a, 'b, 'c, 'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    program_id: &'a Pubkey,
    /// CHECK: This is not dangerous because we don't read or write from this account
    accounts: &'b CancelContextAccounts<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    remaining_accounts: &'c [AccountInfo<'info>],
}

fn cancel(cancel_context: &CancelContext) -> Result<()> {
    msg!("start cancel");

    // check account infos
    // vault authority の検証は後半でしているから不要
    assert_owned_by(
        &cancel_context.accounts.initializer,
        &solana_program::system_program::id(),
    )?;
    assert_owned_by(
        &cancel_context.accounts.taker,
        &solana_program::system_program::id(),
    )?;
    require_keys_eq!(cancel_context.accounts.token_program.key(), spl_token::id());

    let ctx = cancel_context;

    let (vault_authority, vault_authority_bump) = Pubkey::find_program_address(
        &[
            VAULT_AUTHORITY_PDA_SEED,
            ctx.accounts.initializer.key().as_ref(),
            ctx.accounts.taker.key().as_ref(),
        ],
        ctx.program_id,
    );

    // vault authorityのpubkey一致の確認
    require_keys_eq!(
        vault_authority,
        ctx.accounts.vault_authority.key(),
        MyError::PdaPublicKeyMismatch
    );

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
            &vault_authority,
            ctx.program_id,
        )?;

        token::transfer(
            ctx.accounts
                .into_transfer_to_initializer_context(vault_account, initializer_nft_token_account)
                .with_signer(&[&[
                    VAULT_AUTHORITY_PDA_SEED,
                    ctx.accounts.initializer.key().as_ref(),
                    ctx.accounts.taker.key().as_ref(),
                    &[vault_authority_bump],
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
                    &[vault_authority_bump],
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

    // TODO: initializer_additional_sol_amountの一致を確認

    // TODO: typescript側でチェック

    msg!("end cancel");
    Ok(())
}

struct CancelContextAccounts<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    initializer: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    taker: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    vault_authority: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    escrow_account: Box<Account<'info, EscrowAccount>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    token_program: AccountInfo<'info>,
}

impl<'info> Initialize<'info> {
    fn into_transfer_to_pda_context(
        &self,
        initializer_nft_token_account: &AccountInfo<'info>,
        vault_account: &AccountInfo<'info>,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: initializer_nft_token_account.clone(),
            to: vault_account.clone(),
            authority: self.initializer.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_set_authority_context(
        &self,
        vault_account: &AccountInfo<'info>,
    ) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: vault_account.clone(),
            current_authority: self.initializer.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
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
            authority: self.taker.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_transfer_to_taker_context(
        &self,
        vault_account: &AccountInfo<'info>,
        taker_nft_token_account: &AccountInfo<'info>,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: vault_account.clone(),
            to: taker_nft_token_account.clone(),
            authority: self.vault_authority.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
}

impl<'info> Common<'info> for Exchange<'info> {
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
