// Minimal Whitelist Hook Program - Basic Foundation
use anchor_lang::prelude::*;

declare_id!("11111111111111111111111111111113");

#[program]
pub mod whitelist_hook {
    use super::*;

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
        
        msg!("Whitelist initialized with authority: {} (max: {})", 
             authority, max_addresses);
        Ok(())
    }

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
        
        msg!("Address added to whitelist: {}", address);
        Ok(())
    }

    pub fn remove_from_whitelist(
        ctx: Context<UpdateWhitelist>,
        address: Pubkey,
    ) -> Result<()> {
        let whitelist = &mut ctx.accounts.whitelist;
        
        // Find and remove address
        let position = whitelist.approved_addresses.iter().position(|&x| x == address);
        require!(position.is_some(), WhitelistError::AddressNotWhitelisted);
        
        whitelist.approved_addresses.remove(position.unwrap());
        
        msg!("Address removed from whitelist: {}", address);
        Ok(())
    }

    pub fn is_whitelisted(
        ctx: Context<CheckWhitelist>,
        address: Pubkey,
    ) -> Result<bool> {
        let whitelist = &ctx.accounts.whitelist;
        Ok(whitelist.approved_addresses.contains(&address))
    }

    pub fn execute_transfer_hook(
        _ctx: Context<ExecuteTransferHookLegacy>,
        amount: u64,
    ) -> Result<()> {
        msg!("Executing whitelist transfer hook for amount: {}", amount);
        
        // Basic whitelist validation would go here
        // For now, just log
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeWhitelist<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + Whitelist::INIT_SPACE,
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

// Legacy accounts
#[derive(Accounts)]
pub struct ExecuteTransferHookLegacy<'info> {
    /// CHECK: This is safe
    pub source_account: UncheckedAccount<'info>,
    /// CHECK: This is safe  
    pub mint: UncheckedAccount<'info>,
    /// CHECK: This is safe
    pub destination_account: UncheckedAccount<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct Whitelist {
    pub authority: Pubkey,
    pub max_addresses: u16,
    #[max_len(100)] // Start with 100 addresses
    pub approved_addresses: Vec<Pubkey>,
    pub created_at: i64,
}

#[error_code]
pub enum WhitelistError {
    #[msg("Address is already whitelisted")]
    AddressAlreadyWhitelisted,
    #[msg("Address is not whitelisted")]
    AddressNotWhitelisted,
    #[msg("Whitelist is at maximum capacity")]
    WhitelistFull,
}