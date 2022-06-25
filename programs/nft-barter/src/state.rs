use anchor_lang::prelude::*;

pub const VAULT_AUTHORITY_PDA_SEED: &[u8] = b"vault-authority";

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

#[account]
pub struct VaultAuthority {
    pub bump: u8,
}
