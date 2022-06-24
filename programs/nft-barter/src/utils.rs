use crate::errors::MyError;
use {
    anchor_lang::{
        prelude::*,
        solana_program::program_pack::{IsInitialized, Pack},
    },
    spl_associated_token_account::get_associated_token_address,
};

pub fn has_sufficient_funds<'a>(
    initializer_account_info: &AccountInfo,
    taker_account_info: &AccountInfo,
    initializer_additional_sol_amount: u64,
    taker_additional_sol_amount: u64,
) -> Result<()> {
    require_gte!(
        initializer_account_info.try_lamports()?,
        initializer_additional_sol_amount,
        MyError::InitializerInsufficientFunds
    );
    require_gte!(
        taker_account_info.try_lamports()?,
        taker_additional_sol_amount,
        MyError::InitializerInsufficientFunds
    );
    Ok(())
}

pub fn assert_is_ata<'a>(
    ata: &AccountInfo,
    wallet: &Pubkey,
    mint: &AccountInfo,
    has_nft: bool,
) -> Result<spl_token::state::Account> {
    // AccountInfoのownerは、Program that owns this account　https://docs.rs/solana-program/1.5.0/solana_program/account_info/struct.AccountInfo.html
    assert_owned_by(ata, &spl_token::id())?;

    let ata_account: spl_token::state::Account = assert_initialized(ata)?;

    // Accountのownerは、The owner of this account. https://docs.rs/spl-token/latest/spl_token/state/struct.Account.html
    assert_keys_equal(
        &ata_account.owner,
        wallet,
        MyError::AssociatedAuthorityMismatch,
    )?;

    assert_keys_equal(
        &get_associated_token_address(wallet, mint.key),
        ata.key,
        MyError::AssociatedTokenPublicKeyMismatch,
    )?;

    // NFTをちゃんと持っていることの検証
    if has_nft {
        require_eq!(ata_account.amount, 1, MyError::NotFoundNft);
    } else {
        require_eq!(ata_account.amount, 0, MyError::NotFoundNft);
    }

    Ok(ata_account)
}

pub fn assert_is_pda<'a>(
    token_account_info: &AccountInfo,
    bump: u8,
    vault_account_info: &AccountInfo,
    vault_authority: &Pubkey,
    program_id: &'a Pubkey,
) -> Result<spl_token::state::Account> {
    assert_owned_by(vault_account_info, &spl_token::id())?;

    let vault_account: spl_token::state::Account = assert_initialized(vault_account_info)?;

    assert_keys_equal(
        &vault_account.owner,
        vault_authority,
        MyError::AssociatedAuthorityMismatch,
    )?;

    let vault_pda = Pubkey::create_program_address(
        &[b"vault-account", token_account_info.key().as_ref(), &[bump]],
        program_id,
    )
    .unwrap();

    assert_keys_equal(
        &vault_pda,
        vault_account_info.key,
        MyError::AssociatedTokenPublicKeyMismatch,
    )?;

    // NFTをちゃんと持っていることの検証
    let token_account: spl_token::state::Account = assert_initialized(token_account_info)?;

    assert_keys_equal(
        &token_account.mint,
        &vault_account.mint,
        MyError::MintPublicKeyMismatch,
    )?;
    assert_keys_equal(
        &vault_account.owner,
        vault_authority,
        MyError::AssociatedAuthorityMismatch,
    )?;
    require_eq!(token_account.amount, 0, MyError::NotFoundNft);
    require_eq!(vault_account.amount, 1, MyError::NotFoundNft);

    Ok(vault_account)
}

pub fn assert_initialized<T: Pack + IsInitialized>(account_info: &AccountInfo) -> Result<T> {
    let account: T = T::unpack_unchecked(&account_info.data.borrow())?;
    if !account.is_initialized() {
        return err!(MyError::UninitializedAssociatedToken);
    }
    Ok(account)
}

pub fn assert_owned_by(account_info: &AccountInfo, owner: &Pubkey) -> Result<()> {
    require_keys_eq!(
        *account_info.owner,
        *owner,
        MyError::IncorrectAccountInfoOwner
    );
    Ok(())
}

pub fn assert_keys_equal(key1: &Pubkey, key2: &Pubkey, error_code: MyError) -> Result<()> {
    require_keys_eq!(*key1, *key2, error_code);
    Ok(())
}
