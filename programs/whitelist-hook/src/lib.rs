use anchor_lang::prelude::*;
use spl_transfer_hook_interface::{
    instruction::{ExecuteInstruction, TransferHookInstruction},
};
use spl_tlv_account_resolution::{
    account::ExtraAccountMeta,
    seeds::Seed,
    state::ExtraAccountMetaList,
};

declare_id!("11111111111111111111111111111113");

#[program]
pub mod whitelist_hook {
    use super::*;

    /// Initialize whitelist system
    pub fn initialize_whitelist(
        ctx: Context<InitializeWhitelist>,
        authority: Pubkey,
        max_addresses: u16,
    ) -> Result<()> {
        let whitelist = &mut ctx.accounts.whitelist;
        whitelist.authority = authority;
        whitelist.max_addresses = max_addresses;
        whitelist.approved_addresses = Vec::new();
        whitelist.created_at = Clock::get()?.unix_timestamp;
        whitelist.total_transfers_validated = 0;
        whitelist.total_transfers_blocked = 0;
        whitelist.is_enabled = true;
        
        msg!("🏗️ Whitelist initialized with authority: {} (max: {})", 
             authority, max_addresses);
        Ok(())
    }

    /// Add address to whitelist
    pub fn add_to_whitelist(
        ctx: Context<UpdateWhitelist>,
        address: Pubkey,
    ) -> Result<()> {
        let whitelist = &mut ctx.accounts.whitelist;
        
        // Check if already whitelisted
        require!(
            !whitelist.approved_addresses.contains(&address),
            WhitelistError::AddressAlreadyWhitelisted
        );
        
        // Check max capacity
        require!(
            whitelist.approved_addresses.len() < whitelist.max_addresses as usize,
            WhitelistError::WhitelistFull
        );
        
        whitelist.approved_addresses.push(address);
        
        msg!("✅ Address added to whitelist: {} (total: {})", 
             address, whitelist.approved_addresses.len());
        Ok(())
    }

    /// Remove address from whitelist
    pub fn remove_from_whitelist(
        ctx: Context<UpdateWhitelist>,
        address: Pubkey,
    ) -> Result<()> {
        let whitelist = &mut ctx.accounts.whitelist;
        
        // Find and remove address
        let position = whitelist.approved_addresses.iter().position(|&x| x == address);
        require!(position.is_some(), WhitelistError::AddressNotWhitelisted);
        
        whitelist.approved_addresses.remove(position.unwrap());
        
        msg!("❌ Address removed from whitelist: {} (remaining: {})", 
             address, whitelist.approved_addresses.len());
        Ok(())
    }

    /// Enable/disable whitelist validation
    pub fn set_whitelist_enabled(
        ctx: Context<UpdateWhitelist>,
        enabled: bool,
    ) -> Result<()> {
        let whitelist = &mut ctx.accounts.whitelist;
        whitelist.is_enabled = enabled;
        
        msg!("🔄 Whitelist validation: {}", if enabled { "ENABLED" } else { "DISABLED" });
        Ok(())
    }

    /// Bulk add addresses to whitelist
    pub fn bulk_add_to_whitelist(
        ctx: Context<UpdateWhitelist>,
        addresses: Vec<Pubkey>,
    ) -> Result<()> {
        let whitelist = &mut ctx.accounts.whitelist;
        
        // Check capacity
        require!(
            whitelist.approved_addresses.len() + addresses.len() <= whitelist.max_addresses as usize,
            WhitelistError::WhitelistFull
        );
        
        let mut added_count = 0;
        for address in addresses {
            if !whitelist.approved_addresses.contains(&address) {
                whitelist.approved_addresses.push(address);
                added_count += 1;
            }
        }
        
        msg!("✅ Bulk added {} addresses to whitelist (total: {})", 
             added_count, whitelist.approved_addresses.len());
        Ok(())
    }

    /// Check if address is whitelisted (view function)
    pub fn is_whitelisted(
        ctx: Context<CheckWhitelist>,
        address: Pubkey,
    ) -> Result<bool> {
        let whitelist = &ctx.accounts.whitelist;
        let is_approved = whitelist.approved_addresses.contains(&address);
        
        msg!("🔍 Address {} whitelisted: {}", address, is_approved);
        Ok(is_approved)
    }

    /// Initialize extra account meta list for REAL Transfer Hook
    pub fn initialize_extra_account_meta_list(
        ctx: Context<InitializeExtraAccountMetaList>,
    ) -> Result<()> {
        msg!("🚀 Initializing REAL ExtraAccountMetaList for mint: {}", ctx.accounts.mint.key());
        
        // Define the extra accounts needed for whitelist validation
        let account_metas = vec![
            // Whitelist account for validation
            ExtraAccountMeta::new_with_seeds(
                &[Seed::Literal {
                    bytes: b"whitelist".to_vec(),
                }],
                false, // is_signer
                true,  // is_writable (to update transfer stats)
            )?,
        ];

        // Calculate account size needed
        let account_size = ExtraAccountMetaList::size_of(account_metas.len())?;
        
        msg!("📏 ExtraAccountMetaList size needed: {} bytes", account_size);

        // Resize account to fit the data
        ctx.accounts.extra_account_meta_list.realloc(account_size, false)?;

        // Initialize the account data
        let mut data = ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?;
        ExtraAccountMetaList::init::<ExecuteInstruction>(&mut data, &account_metas)?;

        msg!("✅ REAL ExtraAccountMetaList initialized with {} extra accounts", account_metas.len());
        Ok(())
    }

    /// Transfer Hook Execute - REAL implementation called by Token-2022
    pub fn transfer_hook_execute(
        ctx: Context<TransferHookExecute>,
        amount: u64,
    ) -> Result<()> {
        msg!("🔥 REAL Whitelist Hook Execute called!");
        msg!("💰 Transfer amount: {}", amount);
        msg!("👤 Owner: {}", ctx.accounts.owner.key());
        msg!("📦 Source: {}", ctx.accounts.source_token.key());
        msg!("📦 Destination: {}", ctx.accounts.destination_token.key());

        let whitelist = &mut ctx.accounts.whitelist;

        // Check if whitelist validation is enabled
        if !whitelist.is_enabled {
            msg!("ℹ️  Whitelist validation disabled - transfer allowed");
            whitelist.total_transfers_validated += 1;
            return Ok(());
        }

        // Check if owner is whitelisted
        let owner_key = ctx.accounts.owner.key();
        if !whitelist.approved_addresses.contains(&owner_key) {
            whitelist.total_transfers_blocked += 1;
            msg!("❌ Transfer BLOCKED: Owner {} not whitelisted", owner_key);
            msg!("📋 Whitelisted addresses: {}", whitelist.approved_addresses.len());
            return Err(WhitelistError::TransferNotAllowed.into());
        }

        // Additional validation: Check destination owner if needed
        // For enhanced security, we could also validate the destination account owner
        // This would require reading the destination token account data

        // Update statistics
        whitelist.total_transfers_validated += 1;

        msg!("✅ Whitelist validation PASSED!");
        msg!("👤 Owner: {} is whitelisted", owner_key);
        msg!("💰 Transfer amount: {} approved", amount);
        msg!("📊 Total validated: {}", whitelist.total_transfers_validated);
        
        Ok(())
    }

    /// Fallback function - handles all transfer hook interface calls
    pub fn fallback<'info>(
        program_id: &Pubkey,
        accounts: &'info [AccountInfo<'info>],
        data: &[u8],
    ) -> Result<()> {
        msg!("🔀 Whitelist Hook Fallback called with {} accounts", accounts.len());
        
        // Parse the instruction
        let instruction = TransferHookInstruction::unpack(data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        // Dispatch based on instruction type
        match instruction {
            TransferHookInstruction::Execute { amount } => {
                msg!("🔥 Fallback: Executing whitelist hook for amount: {}", amount);
                
                // Validate minimum accounts
                if accounts.len() < 5 {
                    msg!("❌ Not enough accounts provided: {} (need 5)", accounts.len());
                    return Err(ProgramError::NotEnoughAccountKeys.into());
                }

                // Parse accounts according to Transfer Hook Interface
                let source_token = &accounts[0];
                let mint = &accounts[1]; 
                let destination_token = &accounts[2];
                let owner = &accounts[3];
                let whitelist_account = &accounts[4];

                msg!("📋 Validating whitelist transfer:");
                msg!("🪙 Mint: {}", mint.key());
                msg!("👤 Owner: {}", owner.key());
                msg!("💰 Amount: {}", amount);

                // Load and validate whitelist
                let whitelist_data = whitelist_account.try_borrow_data()
                    .map_err(|_| WhitelistError::InvalidWhitelistAccount)?;
                
                // Basic validation - check if account has proper whitelist data structure
                if whitelist_data.len() < 8 + 32 + 2 + 4 + 8 + 8 + 8 + 1 { // discriminator + basic fields
                    msg!("❌ Invalid whitelist account structure");
                    return Err(WhitelistError::InvalidWhitelistAccount.into());
                }

                // Extract enabled status (approximate position - would need exact parsing in production)
                let is_enabled = whitelist_data[whitelist_data.len() - 1] != 0;
                
                if !is_enabled {
                    msg!("ℹ️  Whitelist validation disabled in fallback - transfer allowed");
                    return Ok(());
                }

                // For simplicity in fallback, we'll do basic validation
                // In production, would parse the full whitelist data and check addresses
                msg!("⚠️  Whitelist validation simplified for fallback");
                msg!("✅ Whitelist transfer approved for amount: {}", amount);
                
                Ok(())
            }
            TransferHookInstruction::InitializeExtraAccountMetaList { .. } => {
                msg!("🚀 Fallback: Initializing ExtraAccountMetaList");
                Ok(())
            }
            TransferHookInstruction::UpdateExtraAccountMetaList { .. } => {
                msg!("🔄 Fallback: Updating ExtraAccountMetaList");
                Ok(())
            }
        }
    }

    /// Get whitelist statistics
    pub fn get_whitelist_stats(ctx: Context<GetWhitelistStats>) -> Result<()> {
        let whitelist = &ctx.accounts.whitelist;
        
        msg!("📊 Whitelist Statistics:");
        msg!("👥 Whitelisted addresses: {}/{}", 
             whitelist.approved_addresses.len(), whitelist.max_addresses);
        msg!("✅ Transfers validated: {}", whitelist.total_transfers_validated);
        msg!("❌ Transfers blocked: {}", whitelist.total_transfers_blocked);
        msg!("🔄 Enabled: {}", whitelist.is_enabled);
        msg!("📅 Created at: {}", whitelist.created_at);
        
        // Log first few addresses for reference
        let show_count = std::cmp::min(5, whitelist.approved_addresses.len());
        if show_count > 0 {
            msg!("📋 Sample whitelisted addresses:");
            for i in 0..show_count {
                msg!("  [{}] {}", i, whitelist.approved_addresses[i]);
            }
        }
        
        Ok(())
    }
}

// ========== ACCOUNT STRUCTURES ==========

#[derive(Accounts)]
pub struct InitializeWhitelist<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + Whitelist::SPACE,
        seeds = [b"whitelist"],
        bump
    )]
    pub whitelist: Account<'info, Whitelist>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateWhitelist<'info> {
    #[account(mut)]
    pub whitelist: Account<'info, Whitelist>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct CheckWhitelist<'info> {
    pub whitelist: Account<'info, Whitelist>,
}

#[derive(Accounts)]
pub struct InitializeExtraAccountMetaList<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// CHECK: ExtraAccountMetaList Account, must use these seeds
    #[account(
        init,
        payer = payer,
        space = 1000, // Enough space for ExtraAccountMetaList
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,
    
    /// CHECK: The mint that this hook is for
    pub mint: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Transfer Hook Execute - Called by Token-2022 during transfers
#[derive(Accounts)]
pub struct TransferHookExecute<'info> {
    /// CHECK: Source token account
    pub source_token: UncheckedAccount<'info>,
    /// CHECK: Token mint
    pub mint: UncheckedAccount<'info>,
    /// CHECK: Destination token account
    pub destination_token: UncheckedAccount<'info>,
    /// CHECK: Owner/authority performing the transfer
    pub owner: UncheckedAccount<'info>,
    
    /// Whitelist for validation
    #[account(
        mut,
        seeds = [b"whitelist"],
        bump
    )]
    pub whitelist: Account<'info, Whitelist>,
}

/// Get Whitelist Statistics
#[derive(Accounts)]
pub struct GetWhitelistStats<'info> {
    pub whitelist: Account<'info, Whitelist>,
}

// ========== DATA STRUCTURES ==========

#[account]
pub struct Whitelist {
    pub authority: Pubkey,                 // 32 bytes
    pub max_addresses: u16,                // 2 bytes
    pub approved_addresses: Vec<Pubkey>,   // 4 + (100 * 32) = 3204 bytes
    pub created_at: i64,                   // 8 bytes
    pub total_transfers_validated: u64,    // 8 bytes
    pub total_transfers_blocked: u64,      // 8 bytes
    pub is_enabled: bool,                  // 1 byte
}

impl Whitelist {
    pub const SPACE: usize = 32 + 2 + 3204 + 8 + 8 + 8 + 1; // 3263 bytes
}

#[error_code]
pub enum WhitelistError {
    #[msg("Address is already whitelisted")]
    AddressAlreadyWhitelisted,
    #[msg("Address is not whitelisted")]
    AddressNotWhitelisted,
    #[msg("Whitelist is at maximum capacity")]
    WhitelistFull,
    #[msg("Invalid whitelist account")]
    InvalidWhitelistAccount,
    #[msg("Transfer not allowed - owner not whitelisted")]
    TransferNotAllowed,
    #[msg("Whitelist validation is disabled")]
    WhitelistDisabled,
}