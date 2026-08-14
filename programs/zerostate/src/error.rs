use anchor_lang::prelude::*;

#[error_code]
pub enum DaoError {
    #[msg("that Proof account is not owned by the frankcoin program")]
    NotAFrankcoinProof,
    #[msg("this wallet has not mined enough to join")]
    InsufficientLabour,
    #[msg("the proof account does not belong to this wallet")]
    ProofOwnerMismatch,
    #[msg("proposal title is too long")]
    TitleTooLong,
    #[msg("voting on this proposal has closed")]
    VotingClosed,
    #[msg("invalid vote choice")]
    BadChoice,
}
