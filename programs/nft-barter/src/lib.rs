use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;
pub mod traits;
pub mod utils;

use instructions::*;

declare_id!("FRd6p3td6akTgfhHgJZHyhVeyYUhGWiM9dApVucDGer2");

// Context<'_, '_, '_, 'infoとContext<'info, 'info, 'info, 'infoが混じっているとthese two types are declared with different lifetimes　but data from `accounts` flows into `accounts` here
#[program]
pub mod nft_barter {
    use super::*;

    pub fn initialize<'info>(
        ctx: Context<'_, '_, '_, 'info, Initialize<'info>>,
        initializer_additional_sol_amount: u64, // こいつはstateで使っているから変数の先にもってきている
        taker_additional_sol_amount: u64, // こいつはstateで使っているから変数の先にもってきている
        initializer_nft_amount: u8,
        taker_nft_amount: u8,
        vault_account_bumps: Vec<u8>,
    ) -> Result<()> {
        instructions::initialize::handler(
            ctx,
            initializer_additional_sol_amount,
            taker_additional_sol_amount,
            initializer_nft_amount,
            taker_nft_amount,
            vault_account_bumps,
        )
    }

    pub fn exchange<'info>(
        ctx: Context<'_, '_, '_, 'info, Exchange<'info>>,
        initializer_additional_sol_amount: u64,
        _taker_additional_sol_amount: u64,
    ) -> Result<()> {
        instructions::exchange::handler(
            ctx,
            initializer_additional_sol_amount,
            _taker_additional_sol_amount,
        )
    }

    pub fn cancel_by_initializer<'info>(
        ctx: Context<'_, '_, '_, 'info, CancelByInitializer<'info>>,
    ) -> Result<()> {
        instructions::cancel_by_initializer::handler(ctx)
    }

    pub fn cancel_by_taker<'info>(
        ctx: Context<'_, '_, '_, 'info, CancelByTaker<'info>>,
    ) -> Result<()> {
        instructions::cancel_by_taker::handler(ctx)
    }
}
