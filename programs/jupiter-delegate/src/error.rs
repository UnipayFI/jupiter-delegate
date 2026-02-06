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
    #[msg("Config is paused")]
    ConfigPaused,

    // Swap
    #[msg("Swap too frequent")]
    SwapTooFrequent,
    #[msg("Swap failed")]
    SwapFailed,
    #[msg("Invalid operator")]
    InvalidOperator,

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
    #[msg("Only admin can modify operator")]
    OnlyAdminCanModifyOperator,
    #[msg("Only admin can pause")]
    OnlyAdminCanPause,

    // New variants
    #[msg("Swap amount is too small")]
    SwapAmountTooSmall,
    #[msg("Admin authority mismatch")]
    AdminAuthorityMismatch,
    #[msg("New admin proposed")]
    NewAdminProposed,
    #[msg("No new admin proposed")]
    NoNewAdminProposed,
    #[msg("Delegated amount is insufficient")]
    InsufficientDelegatedAmount,

    // Delegate
    #[msg("Vault has not been delegated authority")]
    DelegateNotApproved,
    #[msg("Invalid delegate token account")]
    InvalidDelegateTokenAccount,
    #[msg("Receiver token account not found in remaining accounts")]
    ReceiverTokenAccountNotFound,

    // Order Engine
    #[msg("Order engine failed")]
    OrderEngineFailed,
    #[msg("Invalid order engine data")]
    InvalidOrderEngineData,

    // Two Hop
    #[msg("Two hop insufficient input amount")]
    TwoHopInsufficientInputAmount,
    #[msg("Two hop max slippage output amount exceeded")]
    TwoHopMaxSlippageOutputAmountExceeded,
    #[msg("Two hop invalid intermediate token amount")]
    TwoHopInvalidIntermediateTokenAmount,

    // Fund Vault
    #[msg("Executor output token account is insufficient")]
    ExecutorOutputTokenAccountIsInsufficient,
    #[msg("Fund vault output token account not found")]
    FundVaultOutputTokenAccountNotFound,
    #[msg("Unsupported token program")]
    UnsupportedTokenProgram,

    // DelegateIsNotReceiver
    #[msg("Delegate is not receiver")]
    DelegateIsNotReceiver,
}
