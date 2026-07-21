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
}
