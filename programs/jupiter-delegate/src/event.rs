use anchor_lang::prelude::*;

#[event]
pub struct InitConfigEvent {
    pub admin: Pubkey,
    pub vault: Pubkey,
    pub cooldown_duration: i64,
}

#[event]
pub struct GrantAccessEvent {
    pub user: Pubkey,
    pub access: Pubkey,
}

#[event]
pub struct RevokeAccessEvent {
    pub user: Pubkey,
    pub access: Pubkey,
}

#[event]
pub struct AdminTransferProposedEvent {
    pub config: Pubkey,
    pub current_admin: Pubkey,
    pub proposed_admin: Pubkey,
}

#[event]
pub struct AdminTransferCompletedEvent {
    pub config: Pubkey,
    pub previous_admin: Pubkey,
    pub new_admin: Pubkey,
}

#[event]
pub struct ModifyCooldownDurationEvent {
    pub config: Pubkey,
    pub cooldown_duration: i64,
}

#[event]
pub struct ModifyOperatorEvent {
    pub config: Pubkey,
    pub operator: Pubkey,
}

#[event]
pub struct PauseEvent {
    pub config: Pubkey,
    pub toggle: bool,
}