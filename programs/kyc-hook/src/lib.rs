use anchor_lang::prelude::*;

declare_id!("11111111111111111111111111111112");

#[program]
pub mod kyc_hook {
    use super::*;

    pub fn initialize_kyc_system(
        ctx: Context<InitializeKYCSystem>,
        authority: Pubkey,
    ) -> Result<()> {
        let kyc_system = &mut ctx.accounts.kyc_system;
        kyc_system.authority = authority;
        kyc_system.total_users = 0;
        kyc_system.created_at = Clock::get()?.unix_timestamp;
        
        msg!("KYC System initialized with authority: {}", authority);
        Ok(())
    }

    pub fn create_kyc_record(
        ctx: Context<CreateKYCRecord>,
        user: Pubkey,
        is_verified: bool,
    ) -> Result<()> {
        let kyc_record = &mut ctx.accounts.kyc_record;
        kyc_record.user = user;
        kyc_record.is_verified = is_verified;
        kyc_record.updated_at = Clock::get()?.unix_timestamp;
        
        msg!("KYC record created for user: {}", user);
        Ok(())
    }

    pub fn update_kyc_status(
        ctx: Context<UpdateKYCStatus>,
        is_verified: bool,
    ) -> Result<()> {
        let kyc_record = &mut ctx.accounts.kyc_record;
        kyc_record.is_verified = is_verified;
        kyc_record.updated_at = Clock::get()?.unix_timestamp;
        
        msg!("KYC status updated for user: {}", kyc_record.user);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeKYCSystem<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + KYCSystem::INIT_SPACE
    )]
    pub kyc_system: Account<'info, KYCSystem>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct CreateKYCRecord<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + KYCRecord::INIT_SPACE
    )]
    pub kyc_record: Account<'info, KYCRecord>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateKYCStatus<'info> {
    #[account(mut)]
    pub kyc_record: Account<'info, KYCRecord>,
    pub authority: Signer<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct KYCSystem {
    pub authority: Pubkey,
    pub total_users: u64,
    pub created_at: i64,
}

#[account]
#[derive(InitSpace)]
pub struct KYCRecord {
    pub user: Pubkey,
    pub is_verified: bool,
    pub updated_at: i64,
}