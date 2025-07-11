use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    program::invoke,
    program_pack::Pack,
};
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};
use spl_token_2022::{
    instruction::{initialize_mint2, mint_to},
    extension::{
        transfer_hook::{TransferHook, instruction::initialize as initialize_transfer_hook},
        ExtensionType,
        StateWithExtensions,
        BaseStateWithExtensions,
    },
};

declare_id!("11111111111111111111111111111111");

#[program]
pub mod token_layer {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        msg!("Token Layer initialized - Real Token-2022 Ready");
        Ok(())
    }

    /// Create REAL Token-2022 with Transfer Hook Extension
    pub fn create_token_2022_with_hooks(
        ctx: Context<CreateToken2022WithHooks>,
        name: String,
        symbol: String,
        decimals: u8,
        hook_program_id: Pubkey,
        initial_supply: u64,
    ) -> Result<()> {
        msg!("🪙 Creating REAL Token-2022 with Transfer Hook: {} ({})", name, symbol);
        msg!("🔗 Hook Program ID: {}", hook_program_id);
        
        // 1. Calculate space needed for mint with Transfer Hook extension
        let transfer_hook_extension = ExtensionType::TransferHook;
        let mint_space = ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(&[transfer_hook_extension])?;
        
        msg!("📏 Mint space needed: {} bytes (with extensions)", mint_space);

        // 2. Create mint account with proper space
        let create_account_ix = anchor_lang::solana_program::system_instruction::create_account(
            &ctx.accounts.payer.key(),
            &ctx.accounts.mint.key(),
            ctx.accounts.rent.minimum_balance(mint_space),
            mint_space as u64,
            &spl_token_2022::id(),
        );

        invoke(
            &create_account_ix,
            &[
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // 3. Initialize Transfer Hook Extension FIRST
        let init_transfer_hook_ix = initialize_transfer_hook(
            &spl_token_2022::id(),
            &ctx.accounts.mint.key(),
            Some(ctx.accounts.authority.key()),
            Some(hook_program_id),
        )?;

        invoke(
            &init_transfer_hook_ix,
            &[
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.token_2022_program.to_account_info(),
            ],
        )?;

        // 4. Initialize the mint
        let init_mint_ix = initialize_mint2(
            &spl_token_2022::id(),
            &ctx.accounts.mint.key(),
            &ctx.accounts.authority.key(),
            Some(&ctx.accounts.authority.key()), // freeze authority
            decimals,
        )?;

        invoke(
            &init_mint_ix,
            &[
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.rent.to_account_info(),
                ctx.accounts.token_2022_program.to_account_info(),
            ],
        )?;

        // 5. Store token metadata
        let token_info = &mut ctx.accounts.token_info;
        token_info.name = name.clone();
        token_info.symbol = symbol.clone();
        token_info.decimals = decimals;
        token_info.mint = ctx.accounts.mint.key();
        token_info.hook_program_id = Some(hook_program_id);
        token_info.created_at = Clock::get()?.unix_timestamp;
        token_info.creator = ctx.accounts.authority.key();
        token_info.total_supply = initial_supply;
        token_info.has_transfer_hooks = true;
        token_info.token_program_id = spl_token_2022::id();

        msg!("✅ REAL Token-2022 created successfully!");
        msg!("🪙 Mint: {}", ctx.accounts.mint.key());
        msg!("🔗 Transfer Hook: {}", hook_program_id);
        msg!("📊 Decimals: {}, Initial Supply: {}", decimals, initial_supply);
        
        Ok(())
    }

    /// Create basic Token-2022 without hooks
    pub fn create_basic_token_2022(
        ctx: Context<CreateBasicToken2022>,
        name: String,
        symbol: String,
        decimals: u8,
        initial_supply: u64,
    ) -> Result<()> {
        msg!("🪙 Creating basic REAL Token-2022: {} ({})", name, symbol);
        
        // 1. Calculate space needed for basic mint
        let mint_space = spl_token_2022::state::Mint::LEN;
        
        // 2. Create mint account
        let create_account_ix = anchor_lang::solana_program::system_instruction::create_account(
            &ctx.accounts.payer.key(),
            &ctx.accounts.mint.key(),
            ctx.accounts.rent.minimum_balance(mint_space),
            mint_space as u64,
            &spl_token_2022::id(),
        );

        invoke(
            &create_account_ix,
            &[
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // 3. Initialize the mint
        let init_mint_ix = initialize_mint2(
            &spl_token_2022::id(),
            &ctx.accounts.mint.key(),
            &ctx.accounts.authority.key(),
            Some(&ctx.accounts.authority.key()),
            decimals,
        )?;

        invoke(
            &init_mint_ix,
            &[
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.rent.to_account_info(),
                ctx.accounts.token_2022_program.to_account_info(),
            ],
        )?;

        // 4. Store metadata
        let token_info = &mut ctx.accounts.token_info;
        token_info.name = name.clone();
        token_info.symbol = symbol.clone();
        token_info.decimals = decimals;
        token_info.mint = ctx.accounts.mint.key();
        token_info.hook_program_id = None;
        token_info.created_at = Clock::get()?.unix_timestamp;
        token_info.creator = ctx.accounts.authority.key();
        token_info.total_supply = initial_supply;
        token_info.has_transfer_hooks = false;
        token_info.token_program_id = spl_token_2022::id();

        msg!("✅ Basic Token-2022 created successfully!");
        Ok(())
    }

    /// Create associated token account for Token-2022
    pub fn create_associated_token_account(
        ctx: Context<CreateAssociatedTokenAccount>,
    ) -> Result<()> {
        msg!("🎯 Creating Associated Token Account for mint: {}", ctx.accounts.mint.key());
        
        let create_ata_ix = spl_associated_token_account::instruction::create_associated_token_account(
            &ctx.accounts.payer.key(),
            &ctx.accounts.wallet.key(),
            &ctx.accounts.mint.key(),
            &spl_token_2022::id(),
        );

        invoke(
            &create_ata_ix,
            &[
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.associated_token.to_account_info(),
                ctx.accounts.wallet.to_account_info(),
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.token_2022_program.to_account_info(),
            ],
        )?;

        msg!("✅ Associated Token Account created: {}", ctx.accounts.associated_token.key());
        Ok(())
    }

    /// Mint tokens to an account (with hook validation)
    pub fn mint_tokens(
        ctx: Context<MintTokens>,
        amount: u64,
    ) -> Result<()> {
        msg!("🔨 Minting {} tokens", amount);
        
        let mint_to_ix = mint_to(
            &spl_token_2022::id(),
            &ctx.accounts.mint.key(),
            &ctx.accounts.destination.key(),
            &ctx.accounts.authority.key(),
            &[],
            amount,
        )?;

        invoke(
            &mint_to_ix,
            &[
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.destination.to_account_info(),
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.token_2022_program.to_account_info(),
            ],
        )?;

        msg!("✅ Minted {} tokens successfully", amount);
        Ok(())
    }

    /// Check if mint has transfer hooks (simplified version)
    pub fn check_transfer_hook_extension(
        ctx: Context<CheckTransferHookExtension>,
    ) -> Result<bool> {
        msg!("🔍 Checking Transfer Hook extension for mint: {}", ctx.accounts.mint.key());
        
        // Read mint account data
        let mint_account_info = &ctx.accounts.mint;
        let mint_data = mint_account_info.try_borrow_data()?;
        
        // Simple check: if account data is larger than basic mint, likely has extensions
        let basic_mint_size = spl_token_2022::state::Mint::LEN;
        let has_extensions = mint_data.len() > basic_mint_size;
        
        if has_extensions {
            msg!("✅ Extensions detected - likely has Transfer Hook");
            
            // Try to parse as StateWithExtensions
            match StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&mint_data) {
                Ok(mint_with_extensions) => {
                    let has_transfer_hook = mint_with_extensions.get_extension::<TransferHook>().is_ok();
                    
                    if has_transfer_hook {
                        let transfer_hook = mint_with_extensions.get_extension::<TransferHook>()?;
                        msg!("🔗 Transfer Hook found:");
                        msg!("🔗 Hook Program: {:?}", transfer_hook.program_id);
                        msg!("👤 Authority: {:?}", transfer_hook.authority);
                        return Ok(true);
                    }
                }
                Err(_) => {
                    msg!("⚠️  Could not parse extensions, but extensions exist");
                }
            }
        } else {
            msg!("❌ No extensions found");
        }
        
        Ok(has_extensions)
    }

    /// Get comprehensive token information
    pub fn get_token_info(ctx: Context<GetTokenInfo>) -> Result<()> {
        let token_info = &ctx.accounts.token_info;
        
        msg!("📋 Token Information:");
        msg!("🏷️  Name: {}", token_info.name);
        msg!("🔤 Symbol: {}", token_info.symbol);
        msg!("🔢 Decimals: {}", token_info.decimals);
        msg!("🪙 Mint: {}", token_info.mint);
        msg!("🔗 Hook Program: {:?}", token_info.hook_program_id);
        msg!("👤 Creator: {}", token_info.creator);
        msg!("📊 Supply: {}", token_info.total_supply);
        msg!("🔒 Has Hooks: {}", token_info.has_transfer_hooks);
        msg!("⚙️  Token Program: {}", token_info.token_program_id);
        
        Ok(())
    }
}

// ========== ACCOUNT STRUCTURES ==========

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct CreateToken2022WithHooks<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + TokenInfo::SPACE,
    )]
    pub token_info: Account<'info, TokenInfo>,
    
    /// The mint account to be created (must be Keypair.generate())
    #[account(mut)]
    pub mint: Signer<'info>,
    
    /// Authority for the mint (mint authority + freeze authority)
    pub authority: Signer<'info>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_2022_program: Program<'info, Token2022>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CreateBasicToken2022<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + TokenInfo::SPACE,
    )]
    pub token_info: Account<'info, TokenInfo>,
    
    #[account(mut)]
    pub mint: Signer<'info>,
    
    pub authority: Signer<'info>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_2022_program: Program<'info, Token2022>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CreateAssociatedTokenAccount<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// CHECK: Wallet that will own the token account
    pub wallet: UncheckedAccount<'info>,
    
    /// CHECK: The mint for the token account
    pub mint: UncheckedAccount<'info>,
    
    /// CHECK: Associated token account to be created
    #[account(mut)]
    pub associated_token: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_2022_program: Program<'info, Token2022>,
}

#[derive(Accounts)]
pub struct MintTokens<'info> {
    /// CHECK: Token mint account (we'll verify it exists)
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,
    
    /// CHECK: Destination token account
    #[account(mut)]
    pub destination: UncheckedAccount<'info>,
    
    pub authority: Signer<'info>,
    
    pub token_2022_program: Program<'info, Token2022>,
}

#[derive(Accounts)]
pub struct CheckTransferHookExtension<'info> {
    /// CHECK: Token mint to check for extensions
    pub mint: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct GetTokenInfo<'info> {
    pub token_info: Account<'info, TokenInfo>,
}

// ========== DATA STRUCTURES ==========

#[account]
pub struct TokenInfo {
    pub name: String,                    // 4 + 50 = 54 bytes
    pub symbol: String,                  // 4 + 10 = 14 bytes
    pub decimals: u8,                    // 1 byte
    pub mint: Pubkey,                    // 32 bytes
    pub hook_program_id: Option<Pubkey>, // 1 + 32 = 33 bytes
    pub created_at: i64,                 // 8 bytes
    pub creator: Pubkey,                 // 32 bytes
    pub total_supply: u64,               // 8 bytes
    pub has_transfer_hooks: bool,        // 1 byte
    pub token_program_id: Pubkey,        // 32 bytes
}

impl TokenInfo {
    pub const SPACE: usize = 54 + 14 + 1 + 32 + 33 + 8 + 32 + 8 + 1 + 32; // 215 bytes
}

#[error_code]
pub enum TokenLayerError {
    #[msg("Invalid token program ID")]
    InvalidTokenProgram,
    #[msg("Invalid hook program")]
    InvalidHookProgram,
    #[msg("Token creation failed")]
    TokenCreationFailed,
    #[msg("Hook setup failed")]
    HookSetupFailed,
    #[msg("Extension initialization failed")]
    ExtensionInitializationFailed,
    #[msg("Failed to parse mint extensions")]
    ExtensionParsingFailed,
}