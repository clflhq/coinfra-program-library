use crate::errors::MyError;
use {
    anchor_lang::{
        prelude::*,
        solana_program::program_pack::{IsInitialized, Pack},
    },
    spl_associated_token_account::get_associated_token_address,
};

pub fn assert_is_ata<'a>(
    ata: &AccountInfo,
    wallet: &Pubkey,
    mint: &AccountInfo,
) -> Result<spl_token::state::Account> {
    assert_owned_by(ata, &spl_token::id())?; // AccountInfoのownerは、Program that owns this account　https://docs.rs/solana-program/1.5.0/solana_program/account_info/struct.AccountInfo.html
    let ata_account: spl_token::state::Account = assert_initialized(ata)?;
    assert_keys_equal(&ata_account.owner, wallet)?; // Accountのownerは、The owner of this account. https://docs.rs/spl-token/latest/spl_token/state/struct.Account.html
    assert_keys_equal(&get_associated_token_address(wallet, mint.key), ata.key)?;
    Ok(ata_account)
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

pub fn assert_keys_equal(key1: &Pubkey, key2: &Pubkey) -> Result<()> {
    require_keys_eq!(*key1, *key2, MyError::AssociatedTokenPublicKeyMismatch);
    Ok(())
}
