///home/alvin/hookswap_token_layer/hookswap_amm/programs/hook-registry/src/lib.rs
use anchor_lang::prelude::*;

declare_id!("11111111111111111111111111111114");

#[program]
pub mod hook_registry {
    use super::*;

    pub fn initialize_registry(
        ctx: Context<InitializeRegistry>,
        authority: Pubkey,
    ) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        registry.authority = authority;
        registry.approved_hooks = Vec::new();
        registry.hook_metadata = Vec::new();
        registry.created_at = Clock::get()?.unix_timestamp;
        registry.total_hooks = 0;
        
        msg!("Hook registry initialized with authority: {}", authority);
        Ok(())
    }
    
    pub fn add_approved_hook(
        ctx: Context<UpdateRegistry>,
        hook_program_id: Pubkey,
        hook_type: HookType,
        name: String,
        description: String,
    ) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        
        // Check if hook is already approved
        require!(
            !registry.approved_hooks.contains(&hook_program_id),
            RegistryError::HookAlreadyApproved
        );
        
        registry.approved_hooks.push(hook_program_id);
        
        // Add metadata
        let metadata = HookMetadata {
            program_id: hook_program_id,
            hook_type: hook_type.clone(),
            name: name.clone(),
            description: description.clone(),
            approved_at: Clock::get()?.unix_timestamp,
            is_active: true,
        };
        
        registry.hook_metadata.push(metadata);
        registry.total_hooks += 1;
        
        msg!("Hook approved: {} ({})", name, hook_program_id);
        Ok(())
    }
    
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
        
        msg!("Hook removed: {}", hook_program_id);
        Ok(())
    }
    
    pub fn is_hook_approved(
        ctx: Context<CheckHook>,
        hook_program_id: Pubkey,
    ) -> Result<bool> {
        let registry = &ctx.accounts.registry;
        Ok(registry.approved_hooks.contains(&hook_program_id))
    }
}

#[derive(Accounts)]
pub struct InitializeRegistry<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + HookRegistry::INIT_SPACE
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
}

#[derive(Accounts)]
pub struct CheckHook<'info> {
    pub registry: Account<'info, HookRegistry>,
}

#[account]
#[derive(InitSpace)]
pub struct HookRegistry {
    pub authority: Pubkey,
    #[max_len(50)] // Start with 50 hooks
    pub approved_hooks: Vec<Pubkey>,
    #[max_len(50)]
    pub hook_metadata: Vec<HookMetadata>,
    pub created_at: i64,
    pub total_hooks: u32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct HookMetadata {
    pub program_id: Pubkey,
    pub hook_type: HookType,
    #[max_len(50)]
    pub name: String,
    #[max_len(200)]
    pub description: String,
    pub approved_at: i64,
    pub is_active: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, PartialEq, Eq)]
pub enum HookType {
    KYC,
    Whitelist,
    Custom,
}

#[error_code]
pub enum RegistryError {
    #[msg("Hook program is already approved")]
    HookAlreadyApproved,
    #[msg("Hook program is not found")]
    HookNotFound,
}