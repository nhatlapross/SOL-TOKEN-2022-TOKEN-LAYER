use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    program::invoke,
    program_pack::Pack,
};

declare_id!("GhQsGRQN9yGibH6F4jvnFXy7Ejbe25PWGENSPAmKGQrB");

#[program]
pub mod hook_registry {
    use super::*;

    /// Initialize hook registry system
    pub fn initialize_registry(
        ctx: Context<InitializeRegistry>,
        authority: Pubkey,
        max_hooks: u16,
    ) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        registry.authority = authority;
        registry.max_hooks = max_hooks;
        registry.approved_hooks = Vec::new();
        registry.hook_metadata = Vec::new();
        registry.created_at = Clock::get()?.unix_timestamp;
        registry.total_hooks = 0;
        registry.total_validations = 0;
        registry.total_rejections = 0;
        registry.is_enabled = true;
        
        msg!("🏗️ Hook registry initialized with authority: {} (max: {})", 
             authority, max_hooks);
        Ok(())
    }
    
    /// Add approved hook to registry
    pub fn add_approved_hook(
        ctx: Context<UpdateRegistry>,
        hook_program_id: Pubkey,
        hook_type: HookType,
        name: String,
        description: String,
        risk_level: RiskLevel,
    ) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        
        // Check if hook is already approved
        require!(
            !registry.approved_hooks.contains(&hook_program_id),
            RegistryError::HookAlreadyApproved
        );
        
        // Check max capacity
        require!(
            registry.approved_hooks.len() < registry.max_hooks as usize,
            RegistryError::RegistryFull
        );
        
        // Validate hook program exists (if provided)
        if let Some(hook_program) = &ctx.accounts.hook_program {
            require!(
                hook_program.executable,
                RegistryError::InvalidHookProgram
            );
        }
        
        registry.approved_hooks.push(hook_program_id);
        
        // Add metadata
        let metadata = HookMetadata {
            program_id: hook_program_id,
            hook_type: hook_type.clone(),
            name: name.clone(),
            description: description.clone(),
            risk_level,
            approved_at: Clock::get()?.unix_timestamp,
            last_validated_at: 0,
            total_validations: 0,
            total_failures: 0,
            is_active: true,
        };
        
        registry.hook_metadata.push(metadata);
        registry.total_hooks += 1;
        
        msg!("✅ Hook approved: {} ({}) - Type: {:?}, Risk: {:?}", 
             name, hook_program_id, hook_type, risk_level);
        Ok(())
    }
    
    /// Remove hook from registry
    pub fn remove_hook(
        ctx: Context<UpdateRegistry>,
        hook_program_id: Pubkey,
    ) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        
        // Check if hook exists
        require!(
            registry.approved_hooks.contains(&hook_program_id),
            RegistryError::HookNotFound
        );
        
        // Remove from approved list
        registry.approved_hooks.retain(|&x| x != hook_program_id);
        
        // Mark metadata as inactive
        if let Some(metadata) = registry.hook_metadata.iter_mut()
            .find(|m| m.program_id == hook_program_id) {
            metadata.is_active = false;
        }
        
        registry.total_hooks = registry.total_hooks.saturating_sub(1);
        
        msg!("❌ Hook removed: {}", hook_program_id);
        Ok(())
    }
    
    /// Enable/disable specific hook
    pub fn set_hook_active(
        ctx: Context<UpdateRegistry>,
        hook_program_id: Pubkey,
        is_active: bool,
    ) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        
        // Find and update hook metadata
        if let Some(metadata) = registry.hook_metadata.iter_mut()
            .find(|m| m.program_id == hook_program_id) {
            metadata.is_active = is_active;
            msg!("🔄 Hook {} status: {}", hook_program_id, 
                 if is_active { "ACTIVE" } else { "INACTIVE" });
        } else {
            return Err(RegistryError::HookNotFound.into());
        }
        
        Ok(())
    }
    
    /// Enable/disable entire registry
    pub fn set_registry_enabled(
        ctx: Context<UpdateRegistry>,
        enabled: bool,
    ) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        registry.is_enabled = enabled;
        
        msg!("🔄 Registry validation: {}", if enabled { "ENABLED" } else { "DISABLED" });
        Ok(())
    }
    
    /// Validate if hook is approved and active
    pub fn is_hook_approved(
        ctx: Context<CheckHook>,
        hook_program_id: Pubkey,
    ) -> Result<bool> {
        let registry = &ctx.accounts.registry;
        
        // Check if registry is enabled
        if !registry.is_enabled {
            msg!("ℹ️  Registry disabled - all hooks considered valid");
            return Ok(true);
        }
        
        // Check if hook is in approved list
        let is_approved = registry.approved_hooks.contains(&hook_program_id);
        
        // Check if hook is active
        let is_active = if let Some(metadata) = registry.hook_metadata.iter()
            .find(|m| m.program_id == hook_program_id) {
            metadata.is_active
        } else {
            false
        };
        
        let result = is_approved && is_active;
        
        msg!("🔍 Hook {} validation: approved={}, active={}, result={}", 
             hook_program_id, is_approved, is_active, result);
        
        Ok(result)
    }
    
    /// Validate hook with statistics update
    pub fn validate_hook_with_stats(
        ctx: Context<ValidateHook>,
        hook_program_id: Pubkey,
        validation_successful: bool,
    ) -> Result<bool> {
        let registry = &mut ctx.accounts.registry;
        
        // Update global stats
        if validation_successful {
            registry.total_validations += 1;
        } else {
            registry.total_rejections += 1;
        }
        
        // Update hook-specific stats
        if let Some(metadata) = registry.hook_metadata.iter_mut()
            .find(|m| m.program_id == hook_program_id) {
            metadata.last_validated_at = Clock::get()?.unix_timestamp;
            if validation_successful {
                metadata.total_validations += 1;
            } else {
                metadata.total_failures += 1;
            }
        }
        
        // Check if registry is enabled
        if !registry.is_enabled {
            msg!("ℹ️  Registry disabled - validation bypassed");
            return Ok(true);
        }
        
        // Check if hook is in approved list
        let is_approved = registry.approved_hooks.contains(&hook_program_id);
        
        // Check if hook is active
        let is_active = if let Some(metadata) = registry.hook_metadata.iter()
            .find(|m| m.program_id == hook_program_id) {
            metadata.is_active
        } else {
            false
        };
        
        let is_valid = is_approved && is_active;
        
        msg!("📊 Hook validation completed: success={}, approved={}, active={}, valid={}", 
             validation_successful, is_approved, is_active, is_valid);
        
        Ok(is_valid && validation_successful)
    }

    /// Get hook metadata
    pub fn get_hook_metadata(
        ctx: Context<CheckHook>,
        hook_program_id: Pubkey,
    ) -> Result<()> {
        let registry = &ctx.accounts.registry;
        
        if let Some(metadata) = registry.hook_metadata.iter()
            .find(|m| m.program_id == hook_program_id) {
            
            msg!("📋 Hook Metadata:");
            msg!("🏷️  Name: {}", metadata.name);
            msg!("🔤 Type: {:?}", metadata.hook_type);
            msg!("📝 Description: {}", metadata.description);
            msg!("⚠️  Risk Level: {:?}", metadata.risk_level);
            msg!("📅 Approved: {}", metadata.approved_at);
            msg!("🔄 Last Validated: {}", metadata.last_validated_at);
            msg!("✅ Validations: {}", metadata.total_validations);
            msg!("❌ Failures: {}", metadata.total_failures);
            msg!("🟢 Active: {}", metadata.is_active);
        } else {
            msg!("❌ Hook metadata not found for: {}", hook_program_id);
        }
        
        Ok(())
    }
    
    /// Get registry statistics
    pub fn get_registry_stats(ctx: Context<CheckHook>) -> Result<()> {
        let registry = &ctx.accounts.registry;
        
        msg!("📊 Hook Registry Statistics:");
        msg!("👥 Total hooks: {}/{}", registry.total_hooks, registry.max_hooks);
        msg!("✅ Total validations: {}", registry.total_validations);
        msg!("❌ Total rejections: {}", registry.total_rejections);
        msg!("🔄 Registry enabled: {}", registry.is_enabled);
        msg!("📅 Created at: {}", registry.created_at);
        
        // Show hook breakdown by type
        let mut kyc_count = 0;
        let mut whitelist_count = 0;
        let mut custom_count = 0;
        let mut active_count = 0;
        
        for metadata in &registry.hook_metadata {
            if metadata.is_active {
                active_count += 1;
                match metadata.hook_type {
                    HookType::KYC => kyc_count += 1,
                    HookType::Whitelist => whitelist_count += 1,
                    _ => custom_count += 1,
                }
            }
        }
        
        msg!("📋 Active hooks by type:");
        msg!("  🔐 KYC: {}", kyc_count);
        msg!("  👥 Whitelist: {}", whitelist_count);
        msg!("  🔧 Other: {}", custom_count);
        msg!("  🟢 Total Active: {}", active_count);
        
        Ok(())
    }
    
    /// Bulk approve hooks (for initial setup)
    pub fn bulk_approve_hooks(
        ctx: Context<BulkUpdate>,
        hooks_data: Vec<BulkHookData>,
    ) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        
        // Check capacity
        require!(
            registry.approved_hooks.len() + hooks_data.len() <= registry.max_hooks as usize,
            RegistryError::RegistryFull
        );
        
        let mut added_count = 0;
        let current_time = Clock::get()?.unix_timestamp;
        
        for hook_data in hooks_data {
            // Check if already approved
            if !registry.approved_hooks.contains(&hook_data.program_id) {
                registry.approved_hooks.push(hook_data.program_id);
                
                let metadata = HookMetadata {
                    program_id: hook_data.program_id,
                    hook_type: hook_data.hook_type.clone(),
                    name: hook_data.name.clone(),
                    description: hook_data.description.clone(),
                    risk_level: hook_data.risk_level,
                    approved_at: current_time,
                    last_validated_at: 0,
                    total_validations: 0,
                    total_failures: 0,
                    is_active: true,
                };
                
                registry.hook_metadata.push(metadata);
                registry.total_hooks += 1;
                added_count += 1;
                
                msg!("✅ Bulk approved: {} ({})", hook_data.name, hook_data.program_id);
            }
        }
        
        msg!("🔄 Bulk approval completed: {} hooks added", added_count);
        Ok(())
    }
}

// ========== ACCOUNT STRUCTURES ==========

#[derive(Accounts)]
pub struct InitializeRegistry<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + HookRegistry::SPACE,
        seeds = [b"hook_registry"],
        bump
    )]
    pub registry: Account<'info, HookRegistry>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateRegistry<'info> {
    #[account(mut)]
    pub registry: Account<'info, HookRegistry>,
    pub authority: Signer<'info>,
    /// CHECK: Hook program to validate (optional)
    pub hook_program: Option<UncheckedAccount<'info>>,
}

#[derive(Accounts)]
pub struct CheckHook<'info> {
    pub registry: Account<'info, HookRegistry>,
}

#[derive(Accounts)]
pub struct ValidateHook<'info> {
    #[account(mut)]
    pub registry: Account<'info, HookRegistry>,
}

#[derive(Accounts)]
pub struct BulkUpdate<'info> {
    #[account(mut)]
    pub registry: Account<'info, HookRegistry>,
    pub authority: Signer<'info>,
}

// ========== DATA STRUCTURES ==========

#[account]
pub struct HookRegistry {
    pub authority: Pubkey,                    // 32 bytes
    pub max_hooks: u16,                       // 2 bytes
    pub approved_hooks: Vec<Pubkey>,          // 4 + (50 * 32) = 1604 bytes
    pub hook_metadata: Vec<HookMetadata>,     // 4 + (50 * 200) = 10004 bytes  
    pub created_at: i64,                      // 8 bytes
    pub total_hooks: u32,                     // 4 bytes
    pub total_validations: u64,               // 8 bytes
    pub total_rejections: u64,                // 8 bytes
    pub is_enabled: bool,                     // 1 byte
}

impl HookRegistry {
    pub const SPACE: usize = 32 + 2 + 1604 + 10004 + 8 + 4 + 8 + 8 + 1; // 11671 bytes
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct HookMetadata {
    pub program_id: Pubkey,              // 32 bytes
    pub hook_type: HookType,             // 1 byte
    pub name: String,                    // 4 + 50 = 54 bytes
    pub description: String,             // 4 + 100 = 104 bytes
    pub risk_level: RiskLevel,           // 1 byte
    pub approved_at: i64,                // 8 bytes
    pub last_validated_at: i64,          // 8 bytes
    pub total_validations: u64,          // 8 bytes
    pub total_failures: u64,             // 8 bytes
    pub is_active: bool,                 // 1 byte
}

impl HookMetadata {
    pub const SPACE: usize = 32 + 1 + 54 + 104 + 1 + 8 + 8 + 8 + 8 + 1; // 225 bytes
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BulkHookData {
    pub program_id: Pubkey,
    pub hook_type: HookType,
    pub name: String,
    pub description: String,
    pub risk_level: RiskLevel,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum HookType {
    KYC,
    Whitelist,
    RateLimit,
    Royalty,
    Custom,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum RiskLevel {
    Low,       // Basic validation, minimal risk
    Medium,    // Standard compliance checks
    High,      // Complex validation, potential for failures
    Critical,  // Mission-critical, extensive validation required
}

#[error_code]
pub enum RegistryError {
    #[msg("Hook program is already approved")]
    HookAlreadyApproved,
    #[msg("Hook program is not found")]
    HookNotFound,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Registry is at maximum capacity")]
    RegistryFull,
    #[msg("Invalid hook program")]
    InvalidHookProgram,
    #[msg("Registry is disabled")]
    RegistryDisabled,
    #[msg("Hook validation failed")]
    HookValidationFailed,
}