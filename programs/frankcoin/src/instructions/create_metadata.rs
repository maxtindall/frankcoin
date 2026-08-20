use anchor_lang::prelude::*;
use anchor_spl::metadata::{
    create_metadata_accounts_v3, mpl_token_metadata::types::DataV2,
    CreateMetadataAccountsV3, Metadata,
};
use anchor_spl::token::Mint;

use crate::{constants::*, error::FrankError, state::Config};

/// Attach a Metaplex Token Metadata account to the mint so wallets, explorers,
/// and DEX aggregators show frankcoin's name, ticker, and Frank's face instead of
/// a generic SPL token.
///
/// The mint authority is the config PDA, so *no wallet* can sign
/// `CreateMetadataAccountV3` — only the program can, via CPI. This instruction
/// exists solely to let the program sign as its own mint authority. It is gated
/// to the program's **upgrade authority** — the pre-launch role — so no one can
/// front-run the token's identity. Set the name/symbol/logo at launch, hand the
/// metadata's update authority to a normal wallet, then freeze it and renounce
/// the program's upgrade authority for a fully immutable, renounced memecoin.
#[derive(Accounts)]
pub struct CreateMetadata<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(seeds = [CONFIG_SEED], bump = config.authority_bump)]
    pub config: Account<'info, Config>,

    #[account(seeds = [MINT_SEED], bump = config.mint_bump)]
    pub mint: Account<'info, Mint>,

    /// CHECK: created and written by the Metaplex Token Metadata program; its
    /// address is the metadata PDA for this mint, which that program enforces.
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: the wallet that will own metadata updates (and later freeze it).
    pub update_authority: UncheckedAccount<'info>,

    // ---- upgrade-authority gate: only the program's upgrade authority may call.
    #[account(constraint = program.programdata_address()? == Some(program_data.key()))]
    pub program: Program<'info, crate::program::Frankcoin>,
    #[account(
        constraint = program_data.upgrade_authority_address == Some(payer.key())
            @ FrankError::NotUpgradeAuthority
    )]
    pub program_data: Account<'info, ProgramData>,

    pub token_metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<CreateMetadata>, name: String, symbol: String, uri: String) -> Result<()> {
    let signer: &[&[&[u8]]] = &[&[CONFIG_SEED, &[ctx.accounts.config.authority_bump]]];

    let data = DataV2 {
        name,
        symbol,
        uri,
        seller_fee_basis_points: 0,
        creators: None,
        collection: None,
        uses: None,
    };

    create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.key(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.metadata.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                mint_authority: ctx.accounts.config.to_account_info(),
                update_authority: ctx.accounts.update_authority.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            signer,
        ),
        data,
        true,  // is_mutable — refine now, freeze at launch
        false, // update_authority_is_signer
        None,  // collection_details — fungible token, none
    )
}
