use anchor_lang::error_code;

#[error_code]
pub enum MyError {
    // 6000
    #[msg("NftAmountMismatch")]
    NftAmountMismatch,
    #[msg("NotProvidedInitializerAssets")]
    NotProvidedInitializerAssets,
    #[msg("NotProvidedTakerAssets")]
    NotProvidedTakerAssets,
    #[msg("UninitializedAssociatedToken")]
    UninitializedAssociatedToken,
    #[msg("IncorrectAccountInfoOwner")]
    IncorrectAccountInfoOwner,
    // 6005
    #[msg("AssociatedTokenPublicKeyMismatch")]
    AssociatedTokenPublicKeyMismatch,
    #[msg("NotFoundNft")]
    NotFoundNft,
    #[msg("VaultAccountBumpsMismatch")]
    VaultAccountBumpsMismatch,
    #[msg("PdaPublicKeyMismatch")]
    PdaPublicKeyMismatch,
    #[msg("AssociatedAuthorityMismatch")]
    AssociatedAuthorityMismatch,
    // 6010
    #[msg("InitializerAdditionalSolAmountMismatch")]
    InitializerAdditionalSolAmountMismatch,
    #[msg("TakerAdditionalSolAmountMismatch")]
    TakerAdditionalSolAmountMismatch,
    #[msg("FoundNft")]
    FoundNft,
    #[msg("MintPublicKeyMismatch")]
    MintPublicKeyMismatch,
    #[msg("InitializerInsufficientFunds")]
    InitializerInsufficientFunds,
    // 6015
    #[msg("TakerInsufficientFunds")]
    TakerInsufficientFunds,
    #[msg("InitializerPublicKeyMismatch")]
    InitializerPublicKeyMismatch,
    #[msg("TakerPublicKeyMismatch")]
    TakerPublicKeyMismatch,
}
