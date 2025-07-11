use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    program::invoke,
    program_pack::Pack,
};
use spl_transfer_hook_interface::{
    instruction::{ExecuteInstruction, TransferHookInstruction},
    get_extra_account_metas_address,
};
use spl_tlv_account_resolution::{
    account::ExtraAccountMeta,
    seeds::Seed,
    state::ExtraAccountMetaList,
};
use spl_token_2022::{
    state::Mint as Token2022Mint,
    extension::{StateWithExtensions, BaseStateWithExtensions, transfer_hook::TransferHook},
};

declare_id!("11111111111111111111111111111112");

#[program]
pub mod kyc_hook {
    use super::*;

    /// Initialize KYC system
    pub fn initialize_kyc_system(
        ctx: Context<InitializeKYCSystem>,
        authority: Pubkey,
    ) -> Result<()> {
        let kyc_system = &mut ctx.accounts.kyc_system;
        kyc_system.authority = authority;
        kyc_system.total_users = 0;
        kyc_system.created_at = Clock::get()?.unix_timestamp;
        kyc_system.total_transfers_validated = 0;
        kyc_system.total_transfers_blocked = 0;
        
        msg!("🏗️ KYC System initialized with authority: {}", authority);
        Ok(())
    }

    /// Create KYC record for a user
    pub fn create_kyc_record(
        ctx: Context<CreateKYCRecord>,
        user: Pubkey,
        is_verified: bool,
        kyc_level: u8, // 0 = None, 1 = Basic, 2 = Enhanced
    ) -> Result<()> {
        let kyc_record = &mut ctx.accounts.kyc_record;
        kyc_record.user = user;
        kyc_record.is_verified = is_verified;
        kyc_record.kyc_level = kyc_level;
        kyc_record.verified_at = if is_verified { Clock::get()?.unix_timestamp } else { 0 };
        kyc_record.updated_at = Clock::get()?.unix_timestamp;
        kyc_record.transfer_count = 0;
        kyc_record.last_transfer_at = 0;
        
        // Update system stats
        let kyc_system = &mut ctx.accounts.kyc_system;
        kyc_system.total_users += 1;
        
        msg!("📝 KYC record created for user: {} (verified: {}, level: {})", 
             user, is_verified, kyc_level);
        Ok(())
    }

    /// Update KYC verification status
    pub fn update_kyc_status(
        ctx: Context<UpdateKYCStatus>,
        is_verified: bool,
        kyc_level: u8,
    ) -> Result<()> {
        let kyc_record = &mut ctx.accounts.kyc_record;
        kyc_record.is_verified = is_verified;
        kyc_record.kyc_level = kyc_level;
        kyc_record.verified_at = if is_verified { Clock::get()?.unix_timestamp } else { 0 };
        kyc_record.updated_at = Clock::get()?.unix_timestamp;
        
        msg!("🔄 KYC status updated for user: {} -> verified: {}, level: {}", 
             kyc_record.user, is_verified, kyc_level);
        Ok(())
    }

    /// Check if user is KYC verified (view function)
    pub fn check_kyc_status(
        ctx: Context<CheckKYCStatus>,
        user: Pubkey,
    ) -> Result<bool> {
        let kyc_record = &ctx.accounts.kyc_record;
        require!(kyc_record.user == user, KYCError::InvalidKYCRecord);
        
        msg!("🔍 KYC status check for {}: verified={}, level={}", 
             user, kyc_record.is_verified, kyc_record.kyc_level);
        Ok(kyc_record.is_verified)
    }

    /// Initialize extra account meta list for REAL Transfer Hook
    pub fn initialize_extra_account_meta_list(
        ctx: Context<InitializeExtraAccountMetaList>,
    ) -> Result<()> {
        msg!("🚀 Initializing REAL ExtraAccountMetaList for mint: {}", ctx.accounts.mint.key());
        
        // Define the extra accounts needed for our KYC validation
        let account_metas = vec![
            // KYC System account (global config)
            ExtraAccountMeta::new_with_seeds(
                &[Seed::Literal {
                    bytes: b"kyc_system".to_vec(),
                }],
                false, // is_signer
                true,  // is_writable (to update stats)
            )?,
            // KYC record for the owner/authority performing the transfer
            ExtraAccountMeta::new_with_seeds(
                &[
                    Seed::Literal {
                        bytes: b"kyc_record".to_vec(),
                    },
                    Seed::AccountKey { index: 3 }, // Index 3 is the owner in transfer hook context
                ],
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
        msg!("🔥 REAL Transfer Hook Execute called!");
        msg!("💰 Transfer amount: {}", amount);
        msg!("👤 Owner: {}", ctx.accounts.owner.key());
        msg!("📦 Source: {}", ctx.accounts.source_token.key());
        msg!("📦 Destination: {}", ctx.accounts.destination_token.key());

        // Load KYC record for the owner
        let kyc_record = &ctx.accounts.kyc_record;
        let kyc_system = &mut ctx.accounts.kyc_system;

        // Validate KYC record belongs to the owner
        require!(
            kyc_record.user == ctx.accounts.owner.key(),
            KYCError::InvalidKYCRecord
        );

        // Check if user is KYC verified
        if !kyc_record.is_verified {
            kyc_system.total_transfers_blocked += 1;
            msg!("❌ Transfer BLOCKED: User {} not KYC verified", ctx.accounts.owner.key());
            return Err(KYCError::UserNotVerified.into());
        }

        // Additional validations based on KYC level and amount
        match kyc_record.kyc_level {
            0 => {
                // No KYC - block all transfers
                kyc_system.total_transfers_blocked += 1;
                return Err(KYCError::InsufficientKYCLevel.into());
            }
            1 => {
                // Basic KYC - limit to smaller amounts
                if amount > 1_000_000 { // 1M tokens (adjust based on decimals)
                    kyc_system.total_transfers_blocked += 1;
                    msg!("❌ Transfer BLOCKED: Amount {} exceeds Basic KYC limit", amount);
                    return Err(KYCError::TransferAmountExceedsLimit.into());
                }
            }
            2 => {
                // Enhanced KYC - allow larger amounts
                if amount > 100_000_000 { // 100M tokens
                    kyc_system.total_transfers_blocked += 1;
                    msg!("❌ Transfer BLOCKED: Amount {} exceeds Enhanced KYC limit", amount);
                    return Err(KYCError::TransferAmountExceedsLimit.into());
                }
            }
            _ => {
                kyc_system.total_transfers_blocked += 1;
                return Err(KYCError::InvalidKYCLevel.into());
            }
        }

        // Update statistics
        kyc_system.total_transfers_validated += 1;
        
        // Update user transfer stats (would need mutable KYC record for this)
        // kyc_record.transfer_count += 1;
        // kyc_record.last_transfer_at = Clock::get()?.unix_timestamp;

        msg!("✅ KYC validation PASSED!");
        msg!("👤 User: {} (Level {})", ctx.accounts.owner.key(), kyc_record.kyc_level);
        msg!("💰 Transfer amount: {} approved", amount);
        msg!("📊 Total validated: {}", kyc_system.total_transfers_validated);
        
        Ok(())
    }

    /// Fallback function - handles all transfer hook interface calls
    pub fn fallback<'info>(
        program_id: &Pubkey,
        accounts: &'info [AccountInfo<'info>],
        data: &[u8],
    ) -> Result<()> {
        msg!("🔀 KYC Hook Fallback called with {} accounts", accounts.len());
        
        // Parse the instruction
        let instruction = TransferHookInstruction::unpack(data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        // Dispatch based on instruction type
        match instruction {
            TransferHookInstruction::Execute { amount } => {
                msg!("🔥 Fallback: Executing transfer hook for amount: {}", amount);
                
                // Validate minimum accounts
                if accounts.len() < 7 {
                    msg!("❌ Not enough accounts provided: {} (need 7)", accounts.len());
                    return Err(ProgramError::NotEnoughAccountKeys.into());
                }

                // Parse accounts according to Transfer Hook Interface
                let source_token = &accounts[0];
                let mint = &accounts[1]; 
                let destination_token = &accounts[2];
                let owner = &accounts[3];
                let kyc_system = &accounts[4];
                let kyc_record = &accounts[5];
                // accounts[6] would be extra accounts if needed

                msg!("📋 Validating transfer:");
                msg!("🪙 Mint: {}", mint.key());
                msg!("👤 Owner: {}", owner.key());
                msg!("💰 Amount: {}", amount);

                // Load and validate KYC record
                let kyc_data = kyc_record.try_borrow_data()
                    .map_err(|_| KYCError::InvalidKYCRecord)?;
                
                // Basic validation - check if account has proper KYC data structure
                if kyc_data.len() < 8 + 32 + 1 + 1 + 8 + 8 + 8 + 8 { // discriminator + user + verified + level + timestamps
                    msg!("❌ Invalid KYC record structure for owner: {}", owner.key());
                    return Err(KYCError::InvalidKYCRecord.into());
                }

                // Extract verification status (after discriminator + pubkey)
                let is_verified = kyc_data[8 + 32] != 0;
                let kyc_level = kyc_data[8 + 32 + 1];
                
                if !is_verified {
                    msg!("❌ User {} not KYC verified - Transfer BLOCKED", owner.key());
                    return Err(KYCError::UserNotVerified.into());
                }

                // Basic amount validation based on KYC level
                let max_amount = match kyc_level {
                    0 => 0,
                    1 => 1_000_000,      // Basic KYC limit
                    2 => 100_000_000,    // Enhanced KYC limit  
                    _ => 0,
                };

                if amount > max_amount {
                    msg!("❌ Amount {} exceeds KYC level {} limit", amount, kyc_level);
                    return Err(KYCError::TransferAmountExceedsLimit.into());
                }

                msg!("✅ KYC validation PASSED in fallback");
                msg!("👤 User: {} (Level {})", owner.key(), kyc_level);
                msg!("💰 Transfer amount {} approved", amount);
                
                Ok(())
            }
            TransferHookInstruction::InitializeExtraAccountMetaList { .. } => {
                msg!("🚀 Fallback: Initializing ExtraAccountMetaList");
                // This should be handled by the dedicated instruction above
                Ok(())
            }
            TransferHookInstruction::UpdateExtraAccountMetaList { .. } => {
                msg!("🔄 Fallback: Updating ExtraAccountMetaList");
                // For now, just log and succeed
                Ok(())
            }
        }
    }

    /// Get KYC system statistics
    pub fn get_kyc_stats(ctx: Context<GetKYCStats>) -> Result<()> {
        let kyc_system = &ctx.accounts.kyc_system;
        
        msg!("📊 KYC System Statistics:");
        msg!("👥 Total users: {}", kyc_system.total_users);
        msg!("✅ Transfers validated: {}", kyc_system.total_transfers_validated);
        msg!("❌ Transfers blocked: {}", kyc_system.total_transfers_blocked);
        msg!("📅 Created at: {}", kyc_system.created_at);
        
        Ok(())
    }
}

// ========== ACCOUNT STRUCTURES ==========

/// Initialize KYC System
#[derive(Accounts)]
pub struct InitializeKYCSystem<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + KYCSystem::SPACE,
        seeds = [b"kyc_system"],
        bump
    )]
    pub kyc_system: Account<'info, KYCSystem>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// Create KYC Record
#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct CreateKYCRecord<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + KYCRecord::SPACE,
        seeds = [b"kyc_record", user.as_ref()],
        bump
    )]
    pub kyc_record: Account<'info, KYCRecord>,
    
    #[account(mut)]
    pub kyc_system: Account<'info, KYCSystem>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// Update KYC Status
#[derive(Accounts)]
pub struct UpdateKYCStatus<'info> {
    #[account(mut)]
    pub kyc_record: Account<'info, KYCRecord>,
    pub authority: Signer<'info>,
}

/// Check KYC Status
#[derive(Accounts)]
pub struct CheckKYCStatus<'info> {
    pub kyc_record: Account<'info, KYCRecord>,
}

/// Initialize ExtraAccountMetaList for Transfer Hook
#[derive(Accounts)]
pub struct InitializeExtraAccountMetaList<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// ExtraAccountMetaList account - MUST use these exact seeds per Transfer Hook Interface
    /// CHECK: ExtraAccountMetaList Account, must use these seeds
    #[account(
        init,
        payer = payer,
        space = 1000, // Enough space for ExtraAccountMetaList
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,
    
    /// The mint that this hook is for
    /// CHECK: Mint account validation
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
    
    /// KYC System for global stats
    #[account(
        mut,
        seeds = [b"kyc_system"],
        bump
    )]
    pub kyc_system: Account<'info, KYCSystem>,
    
    /// KYC Record for the owner
    #[account(
        seeds = [b"kyc_record", owner.key().as_ref()],
        bump
    )]
    pub kyc_record: Account<'info, KYCRecord>,
}

/// Get KYC Statistics
#[derive(Accounts)]
pub struct GetKYCStats<'info> {
    pub kyc_system: Account<'info, KYCSystem>,
}

// ========== DATA STRUCTURES ==========

/// KYC System State - Global configuration and statistics
#[account]
pub struct KYCSystem {
    pub authority: Pubkey,                   // 32 bytes
    pub total_users: u64,                    // 8 bytes
    pub created_at: i64,                     // 8 bytes
    pub total_transfers_validated: u64,      // 8 bytes
    pub total_transfers_blocked: u64,        // 8 bytes
}

impl KYCSystem {
    pub const SPACE: usize = 32 + 8 + 8 + 8 + 8; // 64 bytes
}

/// Individual KYC Record - Per-user verification status
#[account]
pub struct KYCRecord {
    pub user: Pubkey,              // 32 bytes
    pub is_verified: bool,         // 1 byte
    pub kyc_level: u8,            // 1 byte (0=None, 1=Basic, 2=Enhanced)
    pub verified_at: i64,         // 8 bytes
    pub updated_at: i64,          // 8 bytes
    pub transfer_count: u64,      // 8 bytes
    pub last_transfer_at: i64,    // 8 bytes
}

impl KYCRecord {
    pub const SPACE: usize = 32 + 1 + 1 + 8 + 8 + 8 + 8; // 66 bytes
}

/// Error codes
#[error_code]
pub enum KYCError {
    #[msg("User is not KYC verified - Transfer blocked")]
    UserNotVerified,
    #[msg("Invalid KYC record")]
    InvalidKYCRecord,
    #[msg("Transfer amount exceeds KYC level limit")]
    TransferAmountExceedsLimit,
    #[msg("Insufficient KYC level for this operation")]
    InsufficientKYCLevel,
    #[msg("Invalid KYC level")]
    InvalidKYCLevel,
    #[msg("KYC record not found")]
    KYCRecordNotFound,
}