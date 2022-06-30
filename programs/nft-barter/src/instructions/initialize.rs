use anchor_lang::prelude::*;
use anchor_spl::token::{Token, self, SetAuthority, Transfer};
use spl_token::instruction::AuthorityType;

use crate::state::{EscrowAccount, VAULT_AUTHORITY_PDA_SEED, VaultAuthority};
use crate::errors::*;
use crate::utils::{assert_is_ata, assert_rent_exempt};

use anchor_lang::solana_program::{
    program::{invoke, invoke_signed},
    program_pack::Pack,
    system_instruction,
};

#[derive(Accounts)]
#[instruction(initializer_additional_sol_amount: u64, taker_additional_sol_amount: u64, initializer_nft_amount: u8, taker_nft_amount: u8, vault_account_bumps: Vec<u8>)]
pub struct Initialize<'info> {
    #[account(
        mut, 
        constraint = initializer_additional_sol_amount as usize + initializer_nft_amount as usize > 0 @ MyError::NotProvidedInitializerAssets,
        constraint = initializer.to_account_info().try_lamports().unwrap() >= initializer_additional_sol_amount @ MyError::InitializerInsufficientFunds,
        constraint = initializer_nft_amount as usize == vault_account_bumps.len() @ MyError::VaultAccountBumpsMismatch
    )]
    pub initializer: Signer<'info>,
    #[account(
        constraint = taker_additional_sol_amount as usize + taker_nft_amount as usize > 0 @ MyError::NotProvidedTakerAssets,
        constraint = taker.to_account_info().try_lamports().unwrap() >= taker_additional_sol_amount @ MyError::TakerInsufficientFunds,
    )]
    pub taker: SystemAccount<'info>,
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
    pub token_program: Program<'info, Token>,
    #[account(
        init,
        payer = initializer,
        space = 8 // internal anchor discriminator 
            + 1 , // bump
        seeds = [
            VAULT_AUTHORITY_PDA_SEED,
            initializer.key().as_ref(),
            taker.key().as_ref()
        ],
        bump,
      )]
    pub vault_authority: Box<Account<'info, VaultAuthority>>, // SystemAccountだとrent exemptにならない
}

// pub fn initialize(
//     ctx: Context<Initialize>,だと以下エラー
// but data from `ctx` flows into `ctx` here
// lifetime mismatch these two types are declared with different lifetimes...
pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, Initialize<'info>>,
    initializer_additional_sol_amount: u64, // こいつはstateで使っているから変数の先にもってきている
    taker_additional_sol_amount: u64,       // こいつはstateで使っているから変数の先にもってきている
    initializer_nft_amount: u8,
    taker_nft_amount: u8,
    vault_account_bumps: Vec<u8>,
) -> Result<()> {
    msg!("start initialize");

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
                ctx.accounts.initializer.to_account_info().clone(),
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
                ctx.accounts.initializer.to_account_info().clone(),
                vault_account.clone(),
                mint_account.clone(), // なくすとmissing error
                ctx.accounts.rent.to_account_info().clone(), // 超難関ポイント　rentがないと動かない　Error: failed to send transaction: Transaction simulation failed: Error processing Instruction 1: An account required by the instruction is missing
            ],
        )?;

        // check rent exempt
        assert_rent_exempt(&ctx.accounts.rent, vault_account)?;

        // set_authorityをしないとError: failed to send transaction: Transaction simulation failed: Error processing Instruction 1: Cross-program invocation with unauthorized signer or writable account?
        // classでdeserializeするときは、token::authority = initializerを指定しているので、initializerがownerになっている　今回はPDAをわたしているだけなので、signが必要
        token::set_authority(
            ctx.accounts.into_set_authority_context(vault_account),
            AuthorityType::AccountOwner,
            Some(ctx.accounts.vault_authority.key()),
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
    ctx.accounts.vault_authority.bump = *ctx.bumps.get("vault_authority").unwrap();

    if initializer_additional_sol_amount > 0 {
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
                ctx.accounts.initializer.to_account_info().clone(),
                ctx.accounts.escrow_account.to_account_info().clone(),
            ],
        )?;
    }

    msg!("end initialize");
    Ok(())
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
            authority: self.initializer.to_account_info().clone(),
        };
        CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
    }

    fn into_set_authority_context(
        &self,
        vault_account: &AccountInfo<'info>,
    ) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: vault_account.clone(),
            current_authority: self.initializer.to_account_info().clone(),
        };
        CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
    }
}