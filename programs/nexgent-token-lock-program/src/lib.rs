use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;
use solana_security_txt::security_txt;

// Hardcoded addresses for the NEXGENT token - prevents fake mint/vault attacks
pub const NEXGENT_MINT: Pubkey = pubkey!("9JErKCoJX7nmMsYvicoZfEdTdW1qiF9oeocHWyKLmoon");
pub const NEXGENT_VAULT: Pubkey = pubkey!("AbGYx1a6j1KJAPwkePrDXPnMTBHQ2Yhx7n9qJVH4cwme");

security_txt! {
    name: "Nexgent Token Lock Program",
    project_url: "https://nexgent.ai/",
    contacts: "email:contact@nexgent.ai",
    policy: "https://nexgent.ai/security-policy",
    preferred_languages: "en"
}

declare_id!("47YGQvDJJzMAAq7Z6x7LegYhS5Dremk5sGRYGjkAM7c2");

// Account size: 8 (discriminator) + 32 (pubkey) + 16 (user_id) + 8 (amount) + 8 (lock_end) = 72 bytes
pub const LOCK_ACCOUNT_SIZE: usize = 8 + 32 + 16 + 8 + 8;

#[program]
pub mod nexgent_lock {
    use super::*;

    pub fn initialize_lock(ctx: Context<InitializeLock>, user_id: [u8; 16]) -> Result<()> {
        let amount = 250_000 * 10u64.pow(9); // 250,000 tokens with 9 decimals

        // Transfer tokens to the vault
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.vault_token_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
        )?;

        // Set up the lock data
        let lock = &mut ctx.accounts.lock_account;
        let clock = Clock::get()?;

        lock.user = ctx.accounts.user.key();
        lock.user_id = user_id;
        lock.amount = amount;
        lock.lock_end = clock.unix_timestamp + (30 * 24 * 60 * 60); // 30 days in seconds

        emit!(LockInitialized {
            user: lock.user,
            user_id,
            amount,
            lock_end: lock.lock_end,
        });

        Ok(())
    }

    pub fn unlock(ctx: Context<Unlock>) -> Result<()> {
        let clock = Clock::get()?;
        require!(
            clock.unix_timestamp >= ctx.accounts.lock_account.lock_end,
            LockError::LockPeriodNotOver
        );

        let amount = ctx.accounts.lock_account.amount;

        // Transfer tokens back to user
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.vault_token_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.vault_authority.to_account_info(),
                },
                &[&[b"vault", &[ctx.bumps.vault_authority]]],
            ),
            amount,
        )?;

        emit!(TokensUnlocked {
            user: ctx.accounts.lock_account.user,
            user_id: ctx.accounts.lock_account.user_id,
        });

        Ok(())
    }

    pub fn init_lock_account_metadata(_ctx: Context<MetaOnly>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(user_id: [u8; 16])]
pub struct InitializeLock<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        space = LOCK_ACCOUNT_SIZE,
        seeds = [b"lock", user.key().as_ref()],
        bump
    )]
    pub lock_account: Account<'info, LockAccount>,

    /// The NEXGENT token mint - must match hardcoded address
    #[account(
        constraint = token_mint.key() == NEXGENT_MINT @ LockError::InvalidMint
    )]
    pub token_mint: Account<'info, Mint>,

    /// User's token account - must be owned by user and hold NEXGENT tokens
    #[account(
        mut,
        constraint = user_token_account.owner == user.key() @ LockError::InvalidTokenAccountOwner,
        constraint = user_token_account.mint == NEXGENT_MINT @ LockError::MintMismatch
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    /// CHECK: PDA owned by the program - vault authority
    #[account(
        seeds = [b"vault"],
        bump
    )]
    pub vault_authority: AccountInfo<'info>,

    /// Vault token account - must match hardcoded address and be owned by vault_authority
    #[account(
        mut,
        constraint = vault_token_account.key() == NEXGENT_VAULT @ LockError::InvalidVault,
        constraint = vault_token_account.owner == vault_authority.key() @ LockError::InvalidVaultOwner,
        constraint = vault_token_account.mint == NEXGENT_MINT @ LockError::MintMismatch
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Unlock<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"lock", user.key().as_ref()],
        bump,
        close = user,
        has_one = user @ LockError::UnauthorizedUser
    )]
    pub lock_account: Account<'info, LockAccount>,

    /// The NEXGENT token mint - must match hardcoded address
    #[account(
        constraint = token_mint.key() == NEXGENT_MINT @ LockError::InvalidMint
    )]
    pub token_mint: Account<'info, Mint>,

    /// User's token account - must be owned by user and hold NEXGENT tokens
    #[account(
        mut,
        constraint = user_token_account.owner == user.key() @ LockError::InvalidTokenAccountOwner,
        constraint = user_token_account.mint == NEXGENT_MINT @ LockError::MintMismatch
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    /// CHECK: PDA owned by the program - vault authority
    #[account(
        seeds = [b"vault"],
        bump
    )]
    pub vault_authority: AccountInfo<'info>,

    /// Vault token account - must match hardcoded address and be owned by vault_authority
    #[account(
        mut,
        constraint = vault_token_account.key() == NEXGENT_VAULT @ LockError::InvalidVault,
        constraint = vault_token_account.owner == vault_authority.key() @ LockError::InvalidVaultOwner,
        constraint = vault_token_account.mint == NEXGENT_MINT @ LockError::MintMismatch
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[account]
#[derive(Debug)]
pub struct LockAccount {
    pub user: Pubkey,
    pub user_id: [u8; 16],
    pub amount: u64,
    pub lock_end: i64,
}

#[event]
pub struct LockInitialized {
    pub user: Pubkey,
    pub user_id: [u8; 16],
    pub amount: u64,
    pub lock_end: i64,
}

#[event]
pub struct TokensUnlocked {
    pub user: Pubkey,
    pub user_id: [u8; 16],
}

#[derive(Accounts)]
pub struct MetaOnly<'info> {
    #[account(
        init,
        payer = signer,
        space = LOCK_ACCOUNT_SIZE,
        seeds = [b"meta_only"],
        bump
    )]
    pub lock_account: Account<'info, LockAccount>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum LockError {
    #[msg("Lock period has not ended.")]
    LockPeriodNotOver,
    #[msg("Invalid token mint. Only NEXGENT token is supported.")]
    InvalidMint,
    #[msg("Invalid vault token account.")]
    InvalidVault,
    #[msg("Token mint does not match expected NEXGENT mint.")]
    MintMismatch,
    #[msg("Token account owner does not match the user.")]
    InvalidTokenAccountOwner,
    #[msg("Vault token account must be owned by vault authority PDA.")]
    InvalidVaultOwner,
    #[msg("Unauthorized user.")]
    UnauthorizedUser,
}
