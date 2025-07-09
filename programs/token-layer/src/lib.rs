use anchor_lang::prelude::*;
use anchor_spl::token_2022::*;
use anchor_spl::token_interface::{Mint, Token2022, TokenAccount};

declare_id!("11111111111111111111111111111111");

#[program]
pub mod token_layer {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Token Layer initialized");
        Ok(())
    }

    pub fn create_basic_token(
        ctx: Context<CreateBasicToken>,
        name: String,
        symbol: String,
        decimals: u8,
    ) -> Result<()> {
        msg!("Creating token: {} ({})", name, symbol);
        
        // Store token info
        let token_info = &mut ctx.accounts.token_info;
        token_info.name = name;
        token_info.symbol = symbol;
        token_info.decimals = decimals;
        token_info.created_at = Clock::get()?.unix_timestamp;
        token_info.creator = ctx.accounts.payer.key();
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct CreateBasicToken<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + TokenInfo::INIT_SPACE,
    )]
    pub token_info: Account<'info, TokenInfo>,
    
    /// CHECK: This is safe because we're just checking the account
    pub mint: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct TokenInfo {
    #[max_len(50)]
    pub name: String,
    #[max_len(10)]
    pub symbol: String,
    pub decimals: u8,
    pub created_at: i64,
    pub creator: Pubkey,
}