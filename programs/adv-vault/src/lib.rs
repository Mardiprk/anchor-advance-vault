use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("ZB1BxyVhCwFECQoV7bjoun2pMk1yPvz3PGVoKu4d4m5");

#[program]
pub mod advanced_vault {
    use super::*;

    pub fn create_vault(ctx: Context<CreateVault>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;

        vault.initialize(ctx.accounts.admin.key(), ctx.bumps.vault);

        msg!("Vault created by admin: {}", vault.admin);
        Ok(())
    }

    pub fn stake_tokens(ctx: Context<StakeTokens>, amount: u64, stake_years: u8) -> Result<()> {
        require!(amount > 0, VaultError::InvalidAmount);
        require!(
            stake_years >= 1 && stake_years <= 2,
            VaultError::InvalidStakePeriod
        );

        let user_stake = &mut ctx.accounts.user_stake;
        let clock = Clock::get()?;

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        user_stake.create_stake(
            ctx.accounts.user.key(),
            amount,
            stake_years,
            clock.unix_timestamp,
            ctx.bumps.user_stake,
        )?;

        emit!(StakeCreatedEvent {
            user: ctx.accounts.user.key(),
            amount,
            stake_years,
            unlock_time: user_stake.unlock_time,
        });

        msg!("User staked {} tokens for {} years", amount, stake_years);
        Ok(())
    }

    pub fn withdraw_stake(ctx: Context<WithdrawStake>) -> Result<()> {
        let vault = &ctx.accounts.vault;
        let user_stake = &mut ctx.accounts.user_stake;
        let clock = Clock::get()?;

        user_stake.check_if_unlocked(clock.unix_timestamp)?;

        let total_return = user_stake.calculate_total_return()?;

        let admin_key = vault.admin;
        let vault_bump = vault.bump;
        let seeds = &[b"vault", admin_key.as_ref(), &[vault_bump]];
        let signer_seeds = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        token::transfer(cpi_ctx, total_return)?;

        user_stake.mark_as_withdrawn();

        emit!(StakeWithdrawnEvent {
            user: ctx.accounts.user.key(),
            original_amount: user_stake.amount,
            total_return,
            multiplier: user_stake.get_multiplier(),
        });

        msg!(
            "User withdrew {} tokens ({}x multiplier)",
            total_return,
            user_stake.get_multiplier()
        );
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateVault<'info> {
    #[account(
        init,                                       
        payer = admin,                                
        space = Vault::INIT_SPACE,                     
        seeds = [b"vault", admin.key().as_ref()],      
        bump                                           
    )]
    pub vault: Account<'info, Vault>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct StakeTokens<'info> {
    #[account(
        seeds = [b"vault", vault.admin.as_ref()],     
        bump = vault.bump                              
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        init,                                         
        payer = user,                                   
        space = UserStake::INIT_SPACE,           
        seeds = [b"user_stake", vault.key().as_ref(), user.key().as_ref()],  
        bump
    )]
    pub user_stake: Account<'info, UserStake>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,                                         
        associated_token::mint = mint,             
        associated_token::authority = user        
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,                                 
        payer = user,                                  
        associated_token::mint = mint,           
        associated_token::authority = vault      
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawStake<'info> {
    #[account(
        seeds = [b"vault", vault.admin.as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        mut,                                         
        seeds = [b"user_stake", vault.key().as_ref(), user.key().as_ref()],
        bump = user_stake.bump,
        has_one = user @ VaultError::UnauthorizedUser 
    )]
    pub user_stake: Account<'info, UserStake>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed,                                 
        payer = user,                                  
        associated_token::mint = mint,
        associated_token::authority = user
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,                                       
        associated_token::mint = mint,
        associated_token::authority = vault
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct Vault {
    pub admin: Pubkey,
    pub bump: u8,
}

impl Vault {
    pub fn initialize(&mut self, admin: Pubkey, bump: u8) {
        self.admin = admin;
        self.bump = bump;
    }
}

#[account]
#[derive(InitSpace)]
pub struct UserStake {
    pub user: Pubkey,
    pub amount: u64,
    pub stake_years: u8,
    pub stake_time: i64,
    pub unlock_time: i64,
    pub is_withdrawn: bool,
    pub bump: u8,
}

impl UserStake {
    pub fn create_stake(
        &mut self,
        user: Pubkey,
        amount: u64,
        stake_years: u8,
        current_time: i64,
        bump: u8,
    ) -> Result<()> {
        self.user = user;
        self.amount = amount;
        self.stake_years = stake_years;
        self.stake_time = current_time;

        let seconds_per_year = 365 * 24 * 60 * 60;
        let lock_duration = (stake_years as i64) * seconds_per_year;

        self.unlock_time = current_time
            .checked_add(lock_duration)
            .ok_or(VaultError::MathOverflow)?;

        self.is_withdrawn = false;
        self.bump = bump;

        Ok(())
    }

    pub fn check_if_unlocked(&self, current_time: i64) -> Result<()> {
        require!(!self.is_withdrawn, VaultError::AlreadyWithdrawn);
        require!(current_time >= self.unlock_time, VaultError::StillLocked);
        Ok(())
    }

    pub fn calculate_total_return(&self) -> Result<u64> {
        let multiplier = self.get_multiplier();

        self.amount
            .checked_mul(multiplier as u64)
            .ok_or(VaultError::MathOverflow)
    }

    pub fn get_multiplier(&self) -> u8 {
        match self.stake_years {
            1 => 1,
            2 => 2,
            _ => 1,
        }
    }

    pub fn mark_as_withdrawn(&mut self) {
        self.is_withdrawn = true;
    }
}

#[event]
pub struct StakeCreatedEvent {
    #[index]
    pub user: Pubkey,
    pub amount: u64,
    pub stake_years: u8,
    pub unlock_time: i64,
}

#[event]
pub struct StakeWithdrawnEvent {
    #[index]
    pub user: Pubkey,
    pub original_amount: u64,
    pub total_return: u64,
    pub multiplier: u8,
}

#[error_code]
pub enum VaultError {
    #[msg("Amount must be greater than 0")]
    InvalidAmount,

    #[msg("Stake period must be 1 or 2 years")]
    InvalidStakePeriod,

    #[msg("Tokens are still locked")]
    StillLocked,

    #[msg("Stake already withdrawn")]
    AlreadyWithdrawn,

    #[msg("Unauthorized user")]
    UnauthorizedUser,

    #[msg("Math overflow error")]
    MathOverflow,
}
