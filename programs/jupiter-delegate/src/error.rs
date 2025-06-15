use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    // Common
    #[msg("Unauthorized")]
    Unauthorized,

    // Access
    #[msg("Access is not granted")]
    AccessNotGranted,
    #[msg("Access is already granted")]
    AccessAlreadyGranted,

    // Config
    #[msg("Config is not initialized")]
    ConfigNotInitialized,
    #[msg("Config is already initialized")]
    ConfigAlreadyInitialized,
    #[msg("Invalid cooldown duration")]
    InvalidCooldownDuration,

    // Swap
    #[msg("Swap too frequent")]
    SwapTooFrequent,
    #[msg("Swap failed")]
    SwapFailed,

    // Admin
    #[msg("Only admin can propose new admin")]
    OnlyAdminCanProposeNewAdmin,
    #[msg("Only proposed admin can accept")]
    OnlyProposedAdminCanAccept,
    #[msg("Proposed admin is already set")]
    ProposedAdminAlreadySet,
    #[msg("Proposed admin is current admin")]
    ProposedAdminIsCurrentAdmin,
    #[msg("No pending admin transfer")]
    NoPendingAdminTransfer,

    // Modify config
    #[msg("Only admin can modify cooldown duration")]
    OnlyAdminCanModifyCooldownDuration,

    // New variants
    #[msg("Swap amount is too small")]
    SwapAmountTooSmall,
    #[msg("Admin authority mismatch")]
    AdminAuthorityMismatch,
    #[msg("New admin proposed")]
    NewAdminProposed,
    #[msg("No new admin proposed")]
    NoNewAdminProposed,
    #[msg("Vault has not been delegated authority")]
    DelegateNotApproved,
    #[msg("Delegated amount is insufficient")]
    InsufficientDelegatedAmount,
}
