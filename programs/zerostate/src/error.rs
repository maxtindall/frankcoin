use anchor_lang::prelude::*;

#[error_code]
pub enum DaoError {
    #[msg("that Proof account is not owned by the frankcoin program")]
    NotAFrankcoinProof,
    #[msg("this wallet has not mined enough to become a citizen")]
    InsufficientLabour,
    #[msg("the proof account does not belong to this wallet")]
    ProofOwnerMismatch,
    #[msg("proposal title is too long")]
    TitleTooLong,
    #[msg("voting on this proposal has closed")]
    VotingClosed,
    #[msg("voting on this proposal is still open")]
    VotingOpen,
    #[msg("invalid vote choice")]
    BadChoice,
    #[msg("only the admit authority may do that")]
    NotTheAuthority,
    #[msg("only the nominated pending authority may accept the handover")]
    NotThePendingAuthority,
}
