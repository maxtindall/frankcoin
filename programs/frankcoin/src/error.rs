use anchor_lang::prelude::*;

#[error_code]
pub enum FrankError {
    #[msg("frankcoin is fully mined; the 100,000,000 cap has been reached")]
    FullyMined,
    #[msg("proof does not meet the required difficulty")]
    InsufficientDifficulty,
    #[msg("cooldown has not elapsed since your last claim")]
    Cooldown,
    #[msg("arithmetic overflow")]
    Overflow,
    #[msg("only the program's upgrade authority may set token metadata")]
    NotUpgradeAuthority,
    #[msg("that account is not a 0state proposal")]
    NotAZerostateProposal,
    #[msg("this proposal is not a spending proposal")]
    NotASpendProposal,
    #[msg("the proposal has not passed (still open, or more no than yes)")]
    ProposalNotPassed,
    #[msg("the recipient does not match the proposal")]
    RecipientMismatch,
    #[msg("the treasury does not hold enough for this withdrawal")]
    InsufficientTreasury,
}
