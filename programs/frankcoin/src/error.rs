use anchor_lang::prelude::*;

#[error_code]
pub enum FrankError {
    #[msg("proof does not meet the required difficulty")]
    InsufficientDifficulty,
    #[msg("cooldown has not elapsed since your last claim")]
    Cooldown,
    #[msg("arithmetic overflow")]
    Overflow,
    #[msg("only the sitting General Secretary may perform this act")]
    NotTheSecretary,
    #[msg("the one-time genesis mint has already been performed; franks are now proof-of-work only")]
    GenesisAlreadyMinted,
    #[msg("mining is paused by the General Secretary")]
    MiningPaused,
    #[msg("that parameter is outside the bounds the code permits the General Secretary")]
    ParamOutOfBounds,
    #[msg("only the program's upgrade authority may migrate the config")]
    NotUpgradeAuthority,
    #[msg("the config account is already migrated to the current layout")]
    AlreadyMigrated,
}
