use anchor_lang::prelude::*;

#[error_code]
pub enum FrankError {
    #[msg("proof does not meet the required difficulty")]
    InsufficientDifficulty,
    #[msg("cooldown has not elapsed since your last claim")]
    Cooldown,
    #[msg("arithmetic overflow")]
    Overflow,
    #[msg("frankcoin is fully mined — the 5,000,000,000 cap is reached, and there will be no more")]
    FullyMined,
    #[msg("only the program's upgrade authority may set token metadata")]
    NotUpgradeAuthority,
    #[msg("you cannot burn more than you hold")]
    InsufficientBalance,
}
