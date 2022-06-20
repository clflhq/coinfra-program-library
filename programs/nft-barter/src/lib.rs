use crate::errors::MyError;
use crate::utils::assert_is_ata;

use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Mint, SetAuthority, TokenAccount, Transfer};
use spl_token::instruction::AuthorityType;

pub mod errors;
pub mod utils;

declare_id!("FRd6p3td6akTgfhHgJZHyhVeyYUhGWiM9dApVucDGer2");

/*
Controller
*/
#[program]
pub mod nft_barter {
    use anchor_lang::solana_program::{
        program::{invoke, invoke_signed},
        program_pack::Pack,
        system_instruction,
    };

    use super::*;

    const VAULT_AUTHORITY_PDA_SEED: &[u8] = b"vault-authority";

    pub fn initialize2<'info>(
        ctx: Context<'_, '_, '_, 'info, InitializeForTest<'info>>,
        initializer_additional_sol_amount: u64, // こいつはstateで使っているから変数の先にもってきている
        taker_additional_sol_amount: u64, // こいつはstateで使っているから変数の先にもってきている
        initializer_nft_amount: u8,
        taker_nft_amount: u8,
        vault_account_bumps: Vec<u8>,
    ) -> Result<()> {
        Ok(())
    }

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

        /* ok*/
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

        // NFTの数の検証
        let initializer_nft_amount_count = initializer_nft_amount as usize;
        let taker_nft_amount_count = taker_nft_amount as usize;
        let remaining_accounts_count =
            initializer_nft_amount_count * 3 as usize + taker_nft_amount_count * 2 as usize; // initializerはtoken accountとbump takerは直接initializerに払い出すのでtoken accountのみ
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

        // Check, that provided destination is associated token account
        // ownerがtoken programかのチェック
        // remaining_accountsはnot deserialized or validatedなので事前チェックが必要
        // https://github.com/c0wjay/mixture_machine/blob/c0096cc66eb1cec581925ef3c2f4c25be0a1f209/programs/mixture_machine/src/utils.rs#L79-L89　がそのままいけそう
        /*
                if *treasury_holder.owner != spl_token::id() {
                    return Err(ProgramError::InvalidArgument.into());
                }
        */

        // takerにはvaultがないため、token accountとmintだけ
        for index in 0..taker_nft_amount_count {
            // Token Accountの検証
            let account = assert_is_ata(
                &ctx.remaining_accounts[initializer_nft_amount_count * 3 + index * 2],
                ctx.accounts.taker.key,
                &ctx.remaining_accounts[initializer_nft_amount_count * 3 + index * 2 + 1],
            )?;

            // NFTをちゃんと持っていることの検証
            require_eq!(account.amount, 1, MyError::NotFoundNft);
        }

        /* ok */
        let (vault_authority, _vault_authority_bump) = Pubkey::find_program_address(
            &[
                VAULT_AUTHORITY_PDA_SEED,
                ctx.accounts.initializer.key().as_ref(),
                ctx.accounts.taker.key().as_ref(),
            ],
            ctx.program_id,
        );

        // 3で割ってあまり0にtoken account 1にvault account 2にmint account これがないとspl_token::instruction::initialize_accountが無理
        for index in 0..initializer_nft_amount_count {
            // Token Accountの検証
            let token_account = &ctx.remaining_accounts[index * 3];
            let vault_account = &ctx.remaining_accounts[index * 3 + 1];
            let mint_account = &ctx.remaining_accounts[index * 3 + 2];
            let account = assert_is_ata(token_account, ctx.accounts.initializer.key, mint_account)?;

            // NFTをちゃんと持っていることの検証
            require_eq!(account.amount, 1, MyError::NotFoundNft);

            // bumpからPDAの生成 PDA使う必要なし
            // let vault_account = &ctx.remaining_accounts[index * 2 + 1];

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

            // 渡されたPDAの検証

            require_keys_eq!(
                vault_pda,
                vault_account.key(),
                MyError::PdaPublicKeyMismatch
            );

            msg!("vault_account.owner {}", vault_account.owner);

            msg!(
                "escrow_account.owner {}",
                *ctx.accounts.escrow_account.to_account_info().owner
            );
            /*
            msg!(
                "vault_account.owner {}",
                *ctx.accounts.vault_account.to_account_info().owner
            );*/

            // set_authorityをしないとError: failed to send transaction: Transaction simulation failed: Error processing Instruction 1: Cross-program invocation with unauthorized signer or writable account?
            // classでdeserializeするときは、token::authority = initializerを指定しているので、initializerがownerになっている　今回はPDAをわたしているだけなので、signが必要
            // let authority_seeds = &[&b"vault-account", &[vault_account_bump]];
            // let authority_seeds = [b"vault-account", &[vault_account_bump]]; // , ah_key.as_ref()

            /*signer例　動かず
            .with_signer(&[&[
                        b"vault-account",
                        token_account.key().as_ref(),
                        &[vault_account_bump],
                    ]])
            */

            /* 生のSolanaでやっても駄目 invoke_signedでも駄目
            let owner_change_ix = spl_token::instruction::set_authority(
                ctx.accounts.token_program.key,
                vault_account.key,
                Some(&vault_authority),
                spl_token::instruction::AuthorityType::AccountOwner,
                ctx.accounts.initializer.key,
                &[&ctx.accounts.initializer.key],
            )?;

            msg!("Calling the token program to transfer token account ownership...");
            invoke(
                &owner_change_ix,
                &[
                    vault_account.clone(),
                    ctx.accounts.initializer.clone(),
                    ctx.accounts.token_program.clone(),
                ],
            )?;*/
            /* seed 例
            .with_signer(&[&[
                        b"vault-account",
                        token_account.key().as_ref(),
                        &[vault_account_bump],
                    ]])
            */

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
                &[
                    ctx.accounts.initializer.clone(),
                    vault_account.clone(), // これないとError: failed to send transaction: Transaction simulation failed: Error processing Instruction 1: An account required by the instruction is missing
                                           // ctx.accounts.token_program.clone(), token programはなくても通る
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

            // writableにしてwith_signerつけても駄目
            token::set_authority(
                ctx.accounts.into_set_authority_context(vault_account),
                AuthorityType::AccountOwner,
                Some(vault_authority),
            )?;

            // NFTをinitializerに戻す
            token::transfer(
                ctx.accounts
                    .into_transfer_to_pda_context(&token_account, &vault_account),
                1,
            )?;
        }

        /* ok*/
        // TODO: token accountとか全部いれておきexchangeのときに検証すべき
        ctx.accounts.escrow_account.initializer_key = *ctx.accounts.initializer.key;
        ctx.accounts.escrow_account.initializer_nft_amount = initializer_nft_amount;
        ctx.accounts
            .escrow_account
            .initializer_additional_sol_amount = initializer_additional_sol_amount;

        ctx.accounts.escrow_account.taker_key = *ctx.accounts.taker.key;
        ctx.accounts.escrow_account.taker_nft_amount = taker_nft_amount;
        ctx.accounts.escrow_account.taker_additional_sol_amount = taker_additional_sol_amount;

        // ctx.accounts.escrow_account.vault_account_bumps = vault_account_bumps;
        // ctx.accounts.vault_sol_account.bump = *ctx.bumps.get("vault_sol_account").unwrap();
        /*
        msg!(
            "ctx.accounts.escrow_account.vault_account_bump {}",
            ctx.accounts.escrow_account.vault_account_bump
        );
        /*
        msg!(
            "ctx.accounts.vault_sol_account.bump {}",
            ctx.accounts.vault_sol_account.bump
        );*/

        // set_authorityをしないと　ts側のassert.ok(_vault.owner.equals(vault_authority_pda));　がパスしない
        token::set_authority(
            ctx.accounts.into_set_authority_context(),
            AuthorityType::AccountOwner,
            Some(vault_authority),
        )?;*/

        // NFTの移動
        for index in 0..initializer_nft_amount_count {
            /* 直書きしても駄目 forの外に出しても駄目
            let referer2_usdc = Account::<TokenAccount>::try_from(&ctx.remaining_accounts[index])?;
            let cpi_accounts = Transfer {
                from: referer2_usdc.to_account_info().clone(),
                to: ctx.accounts.vault_account.to_account_info().clone(),
                authority: ctx.accounts.initializer.clone(),
            };
            let cpi_ctx = CpiContext::new(ctx.accounts.token_program.clone(), cpi_accounts);
            token::transfer(cpi_ctx, 1)?;*/
        }

        // Error: failed to send transaction: Transaction simulation failed: Error processing Instruction 1: invalid program argument
        /* ok
        token::set_authority(
            ctx.accounts
                .into_add_authority_to_vault_sol_account_context(),
            AuthorityType::AccountOwner,
            Some(vault_authority),
        )?;*/

        /* OK */
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.initializer.key(),
            &ctx.accounts.escrow_account.key(), // sends to event host pda
            initializer_additional_sol_amount,
        );
        // TODO: escrowに入っているお金をinitializerが抜けないかチェック

        // account_infosにctx.accounts.system_program.to_account_info()はなくてもいい
        // invoke_signedの必要もなし だが、account_infoはいずれにしても必須
        // ctx.accounts.initializer.clone() ctx.accounts.vault_sol_account.clone()は必須 ないとAn account required by the instruction is missing
        /*  OK*/
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.initializer.clone(),
                // ctx.accounts.vault_sol_account.to_account_info(),
                ctx.accounts.escrow_account.to_account_info().clone(),
            ],
        )?;

        // 以下はPDAのときだけしか使えない
        /*
                        **ctx.accounts.initializer.try_borrow_mut_lamports()? -= initializer_additional_sol_amount;
                        **ctx.accounts.vault_sol_account.try_borrow_mut_lamports()? +=
                            initializer_additional_sol_amount;
        */
        /*
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
        );*/

        msg!("end initialize");

        Ok(())
    }

    // programでやるとvaultのお金を自由に移動できる
    pub fn exchange2<'info>(
        ctx: Context<'_, '_, '_, 'info, Exchange2<'info>>,
        initializer_additional_sol_amount: u64, // こいつはstateで使っているから変数の先にもってきている
        taker_additional_sol_amount: u64, // こいつはstateで使っているから変数の先にもってきている
        initializer_nft_amount: u8,
        taker_nft_amount: u8,
    ) -> Result<()> {
        **ctx
            .accounts
            .escrow_account
            .to_account_info()
            .try_borrow_mut_lamports()? -= 500_000_000;
        **ctx.accounts.initializer.try_borrow_mut_lamports()? += 500_000_000;
        Ok(())
    }

    pub fn exchange<'info>(
        ctx: Context<'_, '_, '_, 'info, Exchange<'info>>,
        initializer_additional_sol_amount: u64, // こいつはstateで使っているから変数の先にもってきている
        taker_additional_sol_amount: u64, // こいつはstateで使っているから変数の先にもってきている
        initializer_nft_amount: u8,
        taker_nft_amount: u8,
    ) -> Result<()> {
        msg!("start exchange");

        /* ok*/
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

        // NFTの数の検証
        let initializer_nft_amount_count = initializer_nft_amount as usize;
        let taker_nft_amount_count = taker_nft_amount as usize;
        let remaining_accounts_count = (initializer_nft_amount_count + taker_nft_amount_count)
            * 2 as usize
            + (initializer_nft_amount_count + taker_nft_amount_count) * 2 as usize; // initializerはtoken accountとbump takerは直接initializerに払い出すのでtoken accountのみ
        require_eq!(
            ctx.remaining_accounts.len(),
            remaining_accounts_count,
            MyError::NftAmountMismatch
        );

        // TODO: vaultの中のNFT検証

        for index in
            initializer_nft_amount_count..initializer_nft_amount_count + taker_nft_amount_count
        {
            // Token Accountの検証
            let _account = assert_is_ata(
                &ctx.remaining_accounts[index * 2],
                ctx.accounts.initializer.key,
                &ctx.remaining_accounts[index * 2 + 1],
            )?;
        }

        let offset = (initializer_nft_amount_count + taker_nft_amount_count) * 2;

        for index in 0..initializer_nft_amount_count + taker_nft_amount_count {
            // Token Accountの検証
            let account = assert_is_ata(
                &ctx.remaining_accounts[offset + index * 2],
                ctx.accounts.taker.key,
                &ctx.remaining_accounts[offset + index * 2 + 1],
            )?;
            if index >= initializer_nft_amount_count {
                // NFTを現状持っていることの検証
                require_eq!(account.amount, 1, MyError::NotFoundNft);
            }
        }

        let (_vault_authority, vault_authority_bump) = Pubkey::find_program_address(
            &[
                VAULT_AUTHORITY_PDA_SEED,
                ctx.accounts.initializer.key().as_ref(),
                ctx.accounts.taker.key().as_ref(),
            ],
            ctx.program_id,
        );
        // let authority_seeds =  as [&[u8]];

        // initializerがtokenをget
        for index in
            initializer_nft_amount_count..initializer_nft_amount_count + taker_nft_amount_count
        {
            token::transfer(
                ctx.accounts.into_transfer_to_initializer_context(
                    &ctx.remaining_accounts[offset + index * 2],
                    &ctx.remaining_accounts[index * 2],
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
                ctx.accounts.taker.clone(),
                ctx.accounts.initializer.clone(),
                // ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // takerがtokenをget
        for index in 0..initializer_nft_amount_count {
            token::transfer(
                ctx.accounts
                    .into_transfer_to_taker_context(
                        &ctx.remaining_accounts[index * 2],
                        &ctx.remaining_accounts[offset + index * 2],
                    )
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
                    .into_close_context(&ctx.remaining_accounts[index * 2])
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

        msg!(
            "ctx
        .accounts
        .escrow_account
        .initializer_additional_sol_amount {}",
            ctx.accounts
                .escrow_account
                .initializer_additional_sol_amount
        );
        msg!(
            "initializer_additional_sol_amount {}",
            initializer_additional_sol_amount
        );

        **ctx
            .accounts
            .escrow_account
            .to_account_info()
            .try_borrow_mut_lamports()? -= initializer_additional_sol_amount;
        **ctx.accounts.taker.try_borrow_mut_lamports()? += initializer_additional_sol_amount;

        // token accountのclose
        for index in 0..initializer_nft_amount_count {}
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
        msg!("start cancel_by_initializer");

        let (_vault_authority, vault_authority_bump) = Pubkey::find_program_address(
            &[
                VAULT_AUTHORITY_PDA_SEED,
                ctx.accounts.initializer.key().as_ref(),
                ctx.accounts.taker.key().as_ref(),
            ],
            ctx.program_id,
        );

        // 追加のsolをinitializerに戻す
        msg!(
            "ctx
            .accounts
            .escrow_account
            .initializer_additional_sol_amount {}",
            ctx.accounts
                .escrow_account
                .initializer_additional_sol_amount
        );
        msg!(
            "ctx.accounts.escrow_account.to_account_info().lamports {}",
            ctx.accounts
                .escrow_account
                .to_account_info()
                .try_borrow_mut_lamports()?
        );
        msg!(
            "ctx.accounts.initializer.lamports {}",
            ctx.accounts.initializer.try_borrow_mut_lamports()?
        );

        // TODO: vaultの一致の確認

        // NFTをinitializerに戻す

        let initializer_nft_amount_count = &ctx.remaining_accounts.len() / 3;
        for index in 0..initializer_nft_amount_count {
            token::transfer(
                ctx.accounts
                    .into_transfer_to_initializer_context(
                        &ctx.remaining_accounts[index * 3 + 1],
                        &ctx.remaining_accounts[index * 3],
                    )
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
                    .into_close_context(&ctx.remaining_accounts[index * 3 + 1])
                    .with_signer(&[&[
                        VAULT_AUTHORITY_PDA_SEED,
                        ctx.accounts.initializer.key().as_ref(),
                        ctx.accounts.taker.key().as_ref(),
                        &[vault_authority_bump],
                    ]]),
            )?;
        }

        msg!(
            "ctx
            .accounts
            .escrow_account
            .initializer_additional_sol_amount2 {}",
            ctx.accounts
                .escrow_account
                .initializer_additional_sol_amount
        );
        msg!(
            "ctx.accounts.escrow_account.to_account_info().lamports2 {}",
            ctx.accounts
                .escrow_account
                .to_account_info()
                .try_borrow_mut_lamports()?
        );
        msg!(
            "ctx.accounts.initializer.lamports2 {}",
            ctx.accounts.initializer.try_borrow_mut_lamports()?
        );
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

        msg!(
            "ctx
            .accounts
            .escrow_account
            .initializer_additional_sol_amount3 {}",
            ctx.accounts
                .escrow_account
                .initializer_additional_sol_amount
        );
        msg!(
            "ctx.accounts.escrow_account.to_account_info().lamports3 {}",
            ctx.accounts
                .escrow_account
                .to_account_info()
                .try_borrow_mut_lamports()?
        );
        msg!(
            "ctx.accounts.initializer.lamports3 {}",
            ctx.accounts.initializer.try_borrow_mut_lamports()?
        );

        // TODO: initializer_additional_sol_amountの一致を確認

        // vault_sol_accountをcloseする
        // TODO: typescript側でチェック

        msg!("end cancel_by_initializer");
        Ok(())
    }
    /*
    pub fn cancel_by_taker<'info>(
        ctx: Context<'_, '_, '_, 'info, CancelByTaker<'info>>,
    ) -> Result<()> {
        msg!("start cancel_by_taker");

        let (_vault_authority, vault_authority_bump) =
            Pubkey::find_program_address(&[VAULT_AUTHORITY_PDA_SEED], ctx.program_id);
        let authority_seeds = &[&VAULT_AUTHORITY_PDA_SEED[..], &[vault_authority_bump]];

        // NFTをinitializerに戻す
        token::transfer(
            ctx.accounts
                .into_transfer_to_initializer_context()
                .with_signer(&[&authority_seeds[..]]),
            1,
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

        // NFTのtoken accountをcloseする
        token::close_account(
            ctx.accounts
                .into_close_context()
                .with_signer(&[&authority_seeds[..]]),
        )?;

        // vault_sol_accountをcloseする

        msg!("end cancel_by_taker");

        Ok(())
    }
    */
}

/*
Model
*/
#[derive(Accounts)]
#[instruction(initializer_additional_sol_amount: u64, taker_additional_sol_amount: u64, initializer_nft_amount: u8, taker_nft_amount: u8)]
pub struct InitializeForTest<'info> {
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(initializer_additional_sol_amount: u64, taker_additional_sol_amount: u64, initializer_nft_amount: u8, taker_nft_amount: u8)]
pub struct Initialize<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
    pub initializer: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub taker: AccountInfo<'info>,
    // pub mint: Account<'info, Mint>,
    /*
        #[account(
            init,
            seeds = [b"vault-account-test".as_ref()],
            bump,
            payer = initializer,
            token::mint = mint,
            token::authority = initializer,
        )]
        pub vault_account: Account<'info, TokenAccount>,*/
        // ここをmutにすると、Error: 3012: The program expected this account to be already initialized
        /*
        #[account(
            init,
            // initializer.key().as_ref()を加えるとError: failed to send transaction: Transaction simulation failed: Error processing Instruction 1: Cross-program invocation with unauthorized signer or writable account
            // これはts側でseedをちゃんと設定してないからと思われる
            seeds = [b"vault-sol-account", initializer.key().as_ref(), taker.key().as_ref()], // b"vault-sol-account"のあとに.as_ref()をつけるとError: 3003: Failed to deserialize the account?　ここではなくEscrowAccountのbumps
            bump,
            payer = initializer,
            space = 100, // spaceが足りないとError: failed to send transaction: Transaction simulation failed: Error processing Instruction 1: Program failed to complete
            constraint = initializer.lamports() >= initializer_additional_sol_amount,
            constraint = taker.lamports() >= taker_additional_sol_amount,
        )]
        pub vault_sol_account: Account<'info, VaultSolAccount>, // ownerはFRd6p3td6akTgfhHgJZHyhVeyYUhGWiM9dApVucDGer2　手数料と追加のsolを入れる箱 yawwwでも取引ごとに作られている　ここBoxにしてもError: 3007: The given account is owned by a different program than expected
    */
    #[account(zero)]
    pub escrow_account: Box<Account<'info, EscrowAccount>>, // ownerはFRd6p3td6akTgfhHgJZHyhVeyYUhGWiM9dApVucDGer2
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(initializer_additional_sol_amount: u64, taker_additional_sol_amount: u64, initializer_nft_amount: u8, taker_nft_amount: u8)]
pub struct Exchange2<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub taker: AccountInfo<'info>,
    /*
    #[account(mut)]
    pub taker_deposit_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub taker_receive_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub initializer_deposit_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub initializer_receive_token_account: Box<Account<'info, TokenAccount>>,
     */
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
    pub initializer: AccountInfo<'info>,
    #[account(
        mut,
        constraint = escrow_account.initializer_key == *initializer.key,
        constraint = escrow_account.taker_key == *taker.key, // 関係なし Error: Invalid arguments: taker not provided.
        close = initializer // 関係なし Error: failed to send transaction: Transaction simulation failed: Error processing Instruction 0: instruction spent from the balance of an account it does not own
    )]
    pub escrow_account: Box<Account<'info, EscrowAccount>>,
    // close = initializerを加えるとError: failed to send transaction: Transaction simulation failed: Error processing Instruction 0: instruction spent from the balance of an account it does not own
    // #[account(mut, seeds = [b"vault-account".as_ref()], bump = escrow_account.vault_account_bump)]
    //pub vault_account: Box<Account<'info, TokenAccount>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub vault_authority: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,
    // close = initializerをいれると残高がずれる
    /// CHECK: This is not dangerous because we don't read or write from this account
    /// 関係なし Error: Invalid arguments: taker not provided.
    // #[account(mut, seeds = [b"vault-sol-account".as_ref(), initializer.key().as_ref(), taker.key().as_ref()], bump = vault_sol_account.bump, constraint = taker.lamports() >= escrow_account.taker_additional_sol_amount)]
    // pub vault_sol_account: Box<Account<'info, VaultSolAccount>>, // BoxにしてもError: 3012: The program expected this account to be already initialized
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(initializer_additional_sol_amount: u64, taker_additional_sol_amount: u64, initializer_nft_amount: u8, taker_nft_amount: u8)]
pub struct Exchange<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
    pub taker: AccountInfo<'info>,
    /*
    #[account(mut)]
    pub taker_deposit_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub taker_receive_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub initializer_deposit_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub initializer_receive_token_account: Box<Account<'info, TokenAccount>>,
     */
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
    // close = initializerを加えるとError: failed to send transaction: Transaction simulation failed: Error processing Instruction 0: instruction spent from the balance of an account it does not own
    // #[account(mut, seeds = [b"vault-account".as_ref()], bump = escrow_account.vault_account_bump)]
    //pub vault_account: Box<Account<'info, TokenAccount>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub vault_authority: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,
    // close = initializerをいれると残高がずれる
    /// CHECK: This is not dangerous because we don't read or write from this account
    /// 関係なし Error: Invalid arguments: taker not provided.
    // #[account(mut, seeds = [b"vault-sol-account".as_ref(), initializer.key().as_ref(), taker.key().as_ref()], bump = vault_sol_account.bump, constraint = taker.lamports() >= escrow_account.taker_additional_sol_amount)]
    // pub vault_sol_account: Box<Account<'info, VaultSolAccount>>, // BoxにしてもError: 3012: The program expected this account to be already initialized
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
    // #[account(mut, seeds = [b"vault-account".as_ref()], bump = escrow_account.vault_account_bump)]
    // pub vault_account: Account<'info, TokenAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub vault_authority: AccountInfo<'info>,
    //[account(mut)]
    //pub initializer_deposit_token_account: Account<'info, TokenAccount>,
    // #[account(mut)]
    // pub vault_sol_account: Account<'info, TokenAccount>,
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
    // #[account(mut, seeds = [b"vault-account".as_ref()], bump = escrow_account.vault_account_bump)]
    // pub vault_account: Account<'info, TokenAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub vault_authority: AccountInfo<'info>,
    // #[account(mut)]
    // pub initializer_deposit_token_account: Account<'info, TokenAccount>,
    // #[account(mut)]
    // pub vault_sol_account: Account<'info, TokenAccount>,
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
    //pub initializer_deposit_token_account: Pubkey,
    //pub initializer_receive_token_account: Pubkey,
    //pub initializer_amount: u64,
    pub initializer_nft_amount: u8,
    pub initializer_additional_sol_amount: u64,
    pub taker_key: Pubkey,
    // pub taker_amount: u64,
    pub taker_nft_amount: u8,
    pub taker_additional_sol_amount: u64,
    pub vault_account_bump: u8,
    // pub vault_sol_account_bump: u8,
    // pub vault_account_bumps: Vec<u8>,
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
    /* ok*/
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
    /*
    fn into_add_authority_to_vault_sol_account_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.vault_sol_account.to_account_info().clone(),
            current_authority: self.initializer.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    } */
}

impl<'info> Exchange<'info> {
    fn into_transfer_to_initializer_context(
        &self,
        taker_nft_token_account: &AccountInfo<'info>,
        initializer_nft_token_account: &AccountInfo<'info>,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        // 読んだ
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
        // 読んだ
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

impl<'info> Cancel<'info> for CancelByInitializer<'info> {
    fn vault_authority(&self) -> &AccountInfo<'info> {
        return &self.vault_authority;
    }

    fn token_program(&self) -> &AccountInfo<'info> {
        return &self.token_program;
    }
}

impl<'info> Common<'info> for CancelByInitializer<'info> {
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

impl<'info> Cancel<'info> for CancelByTaker<'info> {
    fn vault_authority(&self) -> &AccountInfo<'info> {
        return &self.vault_authority;
    }

    fn token_program(&self) -> &AccountInfo<'info> {
        return &self.token_program;
    }
}

impl<'info> Common<'info> for CancelByTaker<'info> {
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
        // 読んだ
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
        // 読んだ
        let cpi_accounts = CloseAccount {
            account: vault_account.clone(),
            destination: self.initializer().clone(), // initializerに権限を返却する
            authority: self.vault_authority().clone(),
        };
        CpiContext::new(self.token_program().clone(), cpi_accounts)
    }
}
