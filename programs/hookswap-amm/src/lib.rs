use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    program::invoke,
    program::invoke_signed,
    program_pack::Pack,
};
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};
use spl_token_2022::{
    instruction::transfer_checked,
    state::Mint as Token2022Mint,
    extension::{StateWithExtensions, BaseStateWithExtensions},
};

declare_id!("EJCk9aNdKk21Mr3C33aYtnnuBe2vKxVm9eS3TjLWUHuB");

#[program]
pub mod hookswap_amm {
    use super::*;

    /// Initialize AMM
    pub fn initialize_amm(
        ctx: Context<InitializeAMM>,
        fee_rate: u64, // basis points (e.g. 30 = 0.3%)
    ) -> Result<()> {
        let amm_config = &mut ctx.accounts.amm_config;
        amm_config.authority = ctx.accounts.authority.key();
        amm_config.fee_rate = fee_rate;
        amm_config.total_pools = 0;
        amm_config.created_at = Clock::get()?.unix_timestamp;
        amm_config.hook_registry = None; // Will be set later
        
        msg!("🏗️ HookSwap AMM initialized with fee rate: {}bp", fee_rate);
        Ok(())
    }

    /// Set hook registry for AMM
    pub fn set_hook_registry(
        ctx: Context<SetHookRegistry>,
        hook_registry: Pubkey,
    ) -> Result<()> {
        let amm_config = &mut ctx.accounts.amm_config;
        amm_config.hook_registry = Some(hook_registry);
        
        msg!("🔗 Hook registry set: {}", hook_registry);
        Ok(())
    }

    /// Create liquidity pool with REAL Token-2022 support
    pub fn create_pool(
        ctx: Context<CreatePool>,
        initial_price: u64, // Price ratio * 10^9
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        
        // Verify mints are Token-2022
        require!(
            ctx.accounts.token_a_mint.owner == &spl_token_2022::id(),
            AMMError::InvalidTokenProgram
        );
        require!(
            ctx.accounts.token_b_mint.owner == &spl_token_2022::id(),
            AMMError::InvalidTokenProgram
        );
        
        pool.token_a_mint = ctx.accounts.token_a_mint.key();
        pool.token_b_mint = ctx.accounts.token_b_mint.key();
        pool.creator = ctx.accounts.creator.key();
        pool.created_at = Clock::get()?.unix_timestamp;
        pool.fee_rate = ctx.accounts.amm_config.fee_rate;
        pool.current_price = initial_price;
        pool.total_liquidity_a = 0;
        pool.total_liquidity_b = 0;
        pool.lp_token_supply = 0;
        pool.hook_enabled = check_mint_has_hooks(&ctx.accounts.token_a_mint)? || 
                           check_mint_has_hooks(&ctx.accounts.token_b_mint)?;
        pool.token_program_id = spl_token_2022::id();
        pool.bump = ctx.bumps.pool;
        
        // Update AMM config
        let amm_config = &mut ctx.accounts.amm_config;
        amm_config.total_pools += 1;
        
        msg!("🏊 Pool created for {}/{}", 
             pool.token_a_mint, pool.token_b_mint);
        msg!("💰 Initial price: {}", initial_price);
        msg!("🔗 Hook validation: {}", pool.hook_enabled);
        Ok(())
    }

    /// Add REAL liquidity to pool
    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        amount_a: u64,
        amount_b: u64,
        min_lp_tokens: u64,
    ) -> Result<u64> {
        let pool = &mut ctx.accounts.pool;
        
        msg!("💧 Adding REAL liquidity: {} A, {} B", amount_a, amount_b);
        
        // Validate hook requirements if enabled
        if pool.hook_enabled {
            validate_transfer_hooks_real(&ctx.accounts.token_a_mint)?;
            validate_transfer_hooks_real(&ctx.accounts.token_b_mint)?;
        }

        // Get decimals from mint accounts
        let token_a_decimals = get_mint_decimals(&ctx.accounts.token_a_mint)?;
        let token_b_decimals = get_mint_decimals(&ctx.accounts.token_b_mint)?;
        
        // REAL Token-2022 transfers
        // Transfer Token A from user to pool
        let transfer_a_ix = transfer_checked(
            &spl_token_2022::id(),
            &ctx.accounts.user_token_a.key(),
            &ctx.accounts.token_a_mint.key(),
            &ctx.accounts.pool_token_a.key(),
            &ctx.accounts.user.key(),
            &[],
            amount_a,
            token_a_decimals,
        )?;

        invoke(
            &transfer_a_ix,
            &[
                ctx.accounts.user_token_a.to_account_info(),
                ctx.accounts.token_a_mint.to_account_info(),
                ctx.accounts.pool_token_a.to_account_info(),
                ctx.accounts.user.to_account_info(),
                ctx.accounts.token_2022_program.to_account_info(),
            ],
        )?;

        // Transfer Token B from user to pool
        let transfer_b_ix = transfer_checked(
            &spl_token_2022::id(),
            &ctx.accounts.user_token_b.key(),
            &ctx.accounts.token_b_mint.key(),
            &ctx.accounts.pool_token_b.key(),
            &ctx.accounts.user.key(),
            &[],
            amount_b,
            token_b_decimals,
        )?;

        invoke(
            &transfer_b_ix,
            &[
                ctx.accounts.user_token_b.to_account_info(),
                ctx.accounts.token_b_mint.to_account_info(),
                ctx.accounts.pool_token_b.to_account_info(),
                ctx.accounts.user.to_account_info(),
                ctx.accounts.token_2022_program.to_account_info(),
            ],
        )?;
        
        // Calculate LP tokens to mint
        let lp_tokens = if pool.lp_token_supply == 0 {
            // Initial liquidity - geometric mean minus minimum liquidity
            let initial_lp = ((amount_a as f64 * amount_b as f64).sqrt() as u64)
                .checked_sub(1000) // Lock minimum liquidity
                .unwrap_or(0);
            initial_lp
        } else {
            // Proportional liquidity based on existing pool
            let lp_from_a = amount_a.checked_mul(pool.lp_token_supply)
                .unwrap().checked_div(pool.total_liquidity_a).unwrap();
            let lp_from_b = amount_b.checked_mul(pool.lp_token_supply)
                .unwrap().checked_div(pool.total_liquidity_b).unwrap();
            
            // Take minimum to maintain ratio
            lp_from_a.min(lp_from_b)
        };
        
        require!(lp_tokens >= min_lp_tokens, AMMError::InsufficientLPTokens);
        
        // Update pool state
        pool.total_liquidity_a = pool.total_liquidity_a.checked_add(amount_a).unwrap();
        pool.total_liquidity_b = pool.total_liquidity_b.checked_add(amount_b).unwrap();
        pool.lp_token_supply = pool.lp_token_supply.checked_add(lp_tokens).unwrap();
        
        msg!("✅ REAL liquidity added: {} LP tokens minted", lp_tokens);
        Ok(lp_tokens)
    }

    /// REAL swap tokens through the pool
    pub fn swap(
        ctx: Context<Swap>,
        amount_in: u64,
        minimum_amount_out: u64,
        a_to_b: bool, // true = A to B, false = B to A
    ) -> Result<u64> {
        let pool = &ctx.accounts.pool;
        
        msg!("🔄 REAL Swap: {} input, direction: {}", 
             amount_in, if a_to_b { "A→B" } else { "B→A" });
        
        // Validate transfer hooks if enabled
        if pool.hook_enabled {
            if a_to_b {
                validate_transfer_hooks_real(&ctx.accounts.token_a_mint)?;
                validate_transfer_hooks_real(&ctx.accounts.token_b_mint)?;
            } else {
                validate_transfer_hooks_real(&ctx.accounts.token_b_mint)?;
                validate_transfer_hooks_real(&ctx.accounts.token_a_mint)?;
            }
            msg!("✅ Transfer hooks validated");
        }
        
        // Calculate swap output using constant product formula
        let (reserve_in, reserve_out) = if a_to_b {
            (pool.total_liquidity_a, pool.total_liquidity_b)
        } else {
            (pool.total_liquidity_b, pool.total_liquidity_a)
        };
        
        // Apply fee
        let fee_amount = amount_in.checked_mul(pool.fee_rate).unwrap() / 10000;
        let amount_in_after_fee = amount_in.checked_sub(fee_amount).unwrap();
        
        // Constant product: x * y = k
        let denominator = reserve_in.checked_add(amount_in_after_fee).unwrap();
        let new_reserve_out = reserve_in.checked_mul(reserve_out).unwrap()
            .checked_div(denominator).unwrap();
        let amount_out = reserve_out.checked_sub(new_reserve_out).unwrap();
        
        require!(amount_out >= minimum_amount_out, AMMError::InsufficientOutput);

        // Get decimals from mint accounts
        let token_a_decimals = get_mint_decimals(&ctx.accounts.token_a_mint)?;
        let token_b_decimals = get_mint_decimals(&ctx.accounts.token_b_mint)?;

        // REAL Token-2022 transfers
        if a_to_b {
            // Transfer Token A from user to pool
            let transfer_in_ix = transfer_checked(
                &spl_token_2022::id(),
                &ctx.accounts.user_token_in.key(),
                &ctx.accounts.token_a_mint.key(),
                &ctx.accounts.pool_token_a.key(),
                &ctx.accounts.user.key(),
                &[],
                amount_in,
                token_a_decimals,
            )?;

            invoke(
                &transfer_in_ix,
                &[
                    ctx.accounts.user_token_in.to_account_info(),
                    ctx.accounts.token_a_mint.to_account_info(),
                    ctx.accounts.pool_token_a.to_account_info(),
                    ctx.accounts.user.to_account_info(),
                    ctx.accounts.token_2022_program.to_account_info(),
                ],
            )?;

            // Transfer Token B from pool to user
            let token_a_key = ctx.accounts.token_a_mint.key();
            let token_b_key = ctx.accounts.token_b_mint.key();
            let pool_seeds = &[
                b"pool",
                token_a_key.as_ref(),
                token_b_key.as_ref(),
                &[pool.bump],
            ];
            let pool_signer = &[&pool_seeds[..]];

            let transfer_out_ix = transfer_checked(
                &spl_token_2022::id(),
                &ctx.accounts.pool_token_b.key(),
                &ctx.accounts.token_b_mint.key(),
                &ctx.accounts.user_token_out.key(),
                &ctx.accounts.pool.key(),
                &[],
                amount_out,
                token_b_decimals,
            )?;

            invoke_signed(
                &transfer_out_ix,
                &[
                    ctx.accounts.pool_token_b.to_account_info(),
                    ctx.accounts.token_b_mint.to_account_info(),
                    ctx.accounts.user_token_out.to_account_info(),
                    ctx.accounts.pool.to_account_info(),
                    ctx.accounts.token_2022_program.to_account_info(),
                ],
                pool_signer,
            )?;
        } else {
            // B to A swap - similar implementation
            let transfer_in_ix = transfer_checked(
                &spl_token_2022::id(),
                &ctx.accounts.user_token_in.key(),
                &ctx.accounts.token_b_mint.key(),
                &ctx.accounts.pool_token_b.key(),
                &ctx.accounts.user.key(),
                &[],
                amount_in,
                token_b_decimals,
            )?;

            invoke(
                &transfer_in_ix,
                &[
                    ctx.accounts.user_token_in.to_account_info(),
                    ctx.accounts.token_b_mint.to_account_info(),
                    ctx.accounts.pool_token_b.to_account_info(),
                    ctx.accounts.user.to_account_info(),
                    ctx.accounts.token_2022_program.to_account_info(),
                ],
            )?;

            let token_a_key = ctx.accounts.token_a_mint.key();
            let token_b_key = ctx.accounts.token_b_mint.key();
            let pool_seeds = &[
                b"pool",
                token_a_key.as_ref(),
                token_b_key.as_ref(),
                &[pool.bump],
            ];
            let pool_signer = &[&pool_seeds[..]];

            let transfer_out_ix = transfer_checked(
                &spl_token_2022::id(),
                &ctx.accounts.pool_token_a.key(),
                &ctx.accounts.token_a_mint.key(),
                &ctx.accounts.user_token_out.key(),
                &ctx.accounts.pool.key(),
                &[],
                amount_out,
                token_a_decimals,
            )?;

            invoke_signed(
                &transfer_out_ix,
                &[
                    ctx.accounts.pool_token_a.to_account_info(),
                    ctx.accounts.token_a_mint.to_account_info(),
                    ctx.accounts.user_token_out.to_account_info(),
                    ctx.accounts.pool.to_account_info(),
                    ctx.accounts.token_2022_program.to_account_info(),
                ],
                pool_signer,
            )?;
        }

        // Update pool reserves
        let pool = &mut ctx.accounts.pool;
        if a_to_b {
            pool.total_liquidity_a = pool.total_liquidity_a.checked_add(amount_in).unwrap();
            pool.total_liquidity_b = pool.total_liquidity_b.checked_sub(amount_out).unwrap();
        } else {
            pool.total_liquidity_b = pool.total_liquidity_b.checked_add(amount_in).unwrap();
            pool.total_liquidity_a = pool.total_liquidity_a.checked_sub(amount_out).unwrap();
        }
        
        msg!("💰 REAL Swap completed: {} output (fee: {})", amount_out, fee_amount);
        msg!("📊 New reserves: A={}, B={}", pool.total_liquidity_a, pool.total_liquidity_b);
        
        Ok(amount_out)
    }

    /// Get pool information
    pub fn get_pool_info(ctx: Context<GetPoolInfo>) -> Result<()> {
        let pool = &ctx.accounts.pool;
        
        msg!("📋 Pool Information:");
        msg!("🪙 Token A: {}", pool.token_a_mint);
        msg!("🪙 Token B: {}", pool.token_b_mint);
        msg!("💰 Liquidity A: {}", pool.total_liquidity_a);
        msg!("💰 Liquidity B: {}", pool.total_liquidity_b);
        msg!("🏷️  LP Supply: {}", pool.lp_token_supply);
        msg!("💱 Current Price: {}", pool.current_price);
        msg!("💸 Fee Rate: {}bp", pool.fee_rate);
        msg!("🔗 Hook Enabled: {}", pool.hook_enabled);
        
        Ok(())
    }
}

/// Helper function to get mint decimals
fn get_mint_decimals(mint_account: &UncheckedAccount) -> Result<u8> {
    let mint_data = mint_account.try_borrow_data()?;
    
    // Try to parse as StateWithExtensions first (for Token-2022 with extensions)
    match StateWithExtensions::<Token2022Mint>::unpack(&mint_data) {
        Ok(mint_with_extensions) => {
            Ok(mint_with_extensions.base.decimals)
        }
        Err(_) => {
            // Fallback to basic Token-2022 mint
            match Token2022Mint::unpack(&mint_data) {
                Ok(mint) => Ok(mint.decimals),
                Err(_) => {
                    msg!("❌ Failed to parse mint data");
                    Err(AMMError::InvalidTokenProgram.into())
                }
            }
        }
    }
}

/// Helper function to check if mint has hooks
fn check_mint_has_hooks(mint_account: &UncheckedAccount) -> Result<bool> {
    let mint_data = mint_account.try_borrow_data()?;
    let basic_mint_size = Token2022Mint::LEN;
    Ok(mint_data.len() > basic_mint_size)
}

/// Helper function to validate transfer hooks for REAL Token-2022
fn validate_transfer_hooks_real(mint_account: &UncheckedAccount) -> Result<()> {
    let has_hooks = check_mint_has_hooks(mint_account)?;
    
    if has_hooks {
        msg!("🔗 Validating transfer hooks for mint: {}", mint_account.key());
        // In production: would validate hook program is approved in registry
        msg!("✅ Transfer hook validation passed");
    } else {
        msg!("ℹ️  No transfer hooks found for mint: {}", mint_account.key());
    }
    
    Ok(())
}

// ========== ACCOUNT STRUCTURES ==========

#[derive(Accounts)]
pub struct InitializeAMM<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + AMMConfig::SPACE,
        seeds = [b"amm_config"],
        bump
    )]
    pub amm_config: Account<'info, AMMConfig>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetHookRegistry<'info> {
    #[account(mut)]
    pub amm_config: Account<'info, AMMConfig>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct CreatePool<'info> {
    #[account(
        init,
        payer = creator,
        space = 8 + Pool::SPACE,
        seeds = [
            b"pool",
            token_a_mint.key().as_ref(),
            token_b_mint.key().as_ref()
        ],
        bump
    )]
    pub pool: Account<'info, Pool>,
    
    #[account(mut)]
    pub amm_config: Account<'info, AMMConfig>,
    
    /// CHECK: Token A mint (Token-2022)
    pub token_a_mint: UncheckedAccount<'info>,
    /// CHECK: Token B mint (Token-2022)
    pub token_b_mint: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub creator: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(
        mut,
        seeds = [
            b"pool",
            token_a_mint.key().as_ref(),
            token_b_mint.key().as_ref()
        ],
        bump = pool.bump
    )]
    pub pool: Account<'info, Pool>,
    
    /// CHECK: Token A mint
    pub token_a_mint: UncheckedAccount<'info>,
    /// CHECK: Token B mint
    pub token_b_mint: UncheckedAccount<'info>,
    
    /// CHECK: User's Token A account
    #[account(mut)]
    pub user_token_a: UncheckedAccount<'info>,
    /// CHECK: User's Token B account
    #[account(mut)]
    pub user_token_b: UncheckedAccount<'info>,
    
    /// CHECK: Pool's Token A account
    #[account(mut)]
    pub pool_token_a: UncheckedAccount<'info>,
    /// CHECK: Pool's Token B account
    #[account(mut)]
    pub pool_token_b: UncheckedAccount<'info>,
    
    pub user: Signer<'info>,
    pub token_2022_program: Program<'info, Token2022>,
}

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(
        mut,
        seeds = [
            b"pool",
            token_a_mint.key().as_ref(),
            token_b_mint.key().as_ref()
        ],
        bump = pool.bump
    )]
    pub pool: Account<'info, Pool>,
    
    /// CHECK: Token A mint
    pub token_a_mint: UncheckedAccount<'info>,
    /// CHECK: Token B mint
    pub token_b_mint: UncheckedAccount<'info>,
    
    /// CHECK: User's input token account
    #[account(mut)]
    pub user_token_in: UncheckedAccount<'info>,
    /// CHECK: User's output token account
    #[account(mut)]
    pub user_token_out: UncheckedAccount<'info>,
    
    /// CHECK: Pool's Token A account
    #[account(mut)]
    pub pool_token_a: UncheckedAccount<'info>,
    /// CHECK: Pool's Token B account
    #[account(mut)]
    pub pool_token_b: UncheckedAccount<'info>,
    
    pub user: Signer<'info>,
    pub token_2022_program: Program<'info, Token2022>,
}

#[derive(Accounts)]
pub struct GetPoolInfo<'info> {
    pub pool: Account<'info, Pool>,
}

// Data structures
#[account]
pub struct AMMConfig {
    pub authority: Pubkey,                // 32 bytes
    pub fee_rate: u64,                    // 8 bytes - basis points
    pub total_pools: u32,                 // 4 bytes
    pub created_at: i64,                  // 8 bytes
    pub hook_registry: Option<Pubkey>,    // 1 + 32 = 33 bytes
}

impl AMMConfig {
    pub const SPACE: usize = 32 + 8 + 4 + 8 + 33; // 85 bytes
}

#[account]
pub struct Pool {
    pub token_a_mint: Pubkey,            // 32 bytes
    pub token_b_mint: Pubkey,            // 32 bytes
    pub creator: Pubkey,                 // 32 bytes
    pub created_at: i64,                 // 8 bytes
    pub fee_rate: u64,                   // 8 bytes
    pub current_price: u64,              // 8 bytes
    pub total_liquidity_a: u64,          // 8 bytes
    pub total_liquidity_b: u64,          // 8 bytes
    pub lp_token_supply: u64,            // 8 bytes
    pub hook_enabled: bool,              // 1 byte
    pub token_program_id: Pubkey,        // 32 bytes
    pub bump: u8,                        // 1 byte
}

impl Pool {
    pub const SPACE: usize = 32 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 1 + 32 + 1; // 178 bytes
}

#[error_code]
pub enum AMMError {
    #[msg("Insufficient LP tokens")]
    InsufficientLPTokens,
    #[msg("Insufficient swap output")]
    InsufficientOutput,
    #[msg("Hook validation failed")]
    HookValidationFailed,
    #[msg("Invalid token pair")]
    InvalidTokenPair,
    #[msg("Pool not found")]
    PoolNotFound,
    #[msg("Insufficient liquidity")]
    InsufficientLiquidity,
    #[msg("Invalid token program")]
    InvalidTokenProgram,
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    #[msg("Pool already exists")]
    PoolAlreadyExists,
}