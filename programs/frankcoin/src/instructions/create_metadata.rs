use anchor_lang::prelude::*;
use anchor_spl::metadata::{
    create_metadata_accounts_v3, mpl_token_metadata::types::DataV2,
    CreateMetadataAccountsV3, Metadata,
};
use anchor_spl::token::Mint;

use crate::{constants::*, error::FrankError, state::Config};

/// Attach a Metaplex Token Metadata account to the mint so wallets and explorers
/// show "frankcoin" (and its logo) instead of a generic SPL token.
///
/// The catch this solves: the mint authority is the config PDA, so *no wallet*
/// can sign `CreateMetadataAccountV3` — only the program can, via CPI with the
/// PDA seeds. So this instruction exists purely to let the program sign as its
/// own mint authority.
///
/// It is a **curatorial** act of the General Secretary, not a monetary one: gated to the
/// sitting General Secretary, it writes only the token's name/symbol/URI and can never mint
/// or move a balance. The metadata is created mutable with its update authority
/// handed to a General Secretary-named wallet (normally the General Secretary itself), so the memecoin's
/// branding can keep evolving with standard Metaplex tooling — and be frozen
/// (`is_mutable = false`) whenever the membership wishes to fix the identity.
#[derive(Accounts)]
pub struct CreateMetadata<'info> {
    #[account(mut)]
    pub general_secretary: Signer<'info>,

    #[account(
        seeds = [CONFIG_SEED],
        bump = config.authority_bump,
        has_one = general_secretary @ FrankError::NotTheSecretary
    )]
    pub config: Account<'info, Config>,

    #[account(seeds = [MINT_SEED], bump = config.mint_bump)]
    pub mint: Account<'info, Mint>,

    /// CHECK: created and written by the Metaplex Token Metadata program; its
    /// address is the metadata PDA for this mint, which that program enforces.
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: the wallet that will own metadata updates (and later freeze it).
    /// Only its key is stored, as the metadata's update authority.
    pub update_authority: UncheckedAccount<'info>,

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
                payer: ctx.accounts.general_secretary.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            signer,
        ),
        data,
        true,  // is_mutable — refine now, freeze when the identity is settled
        false, // update_authority_is_signer — the update authority need not sign here
        None,  // collection_details — fungible token, none
    )
}
