use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

pub const GROUP_VAULT_SEED: &[u8] = b"group_vault";
pub const INSURANCE_VAULT_SEED: &[u8] = b"insurance_vault";
pub const MEMBER_SEED: &[u8] = b"member";

declare_id!("Ez7nS3RhjdeYknDMJSrunJE1wbACMg7yN4YTgFmkHkQz");

#[program]
pub mod ahorro {
	use super::*;

	pub fn create_group(
		ctx: Context<CreateGroup>,
		model_type: u8,
		insurance_bps: u16,
		cycle_order: Vec<Pubkey>,
		contribution_amount: u64,
	) -> Result<()> {
		let group = &mut ctx.accounts.thrift_group;
		require!(insurance_bps <= 1_000, AhorroError::InvalidInsuranceBps);

		group.authority = ctx.accounts.authority.key();
		group.model_type = model_type;
		group.insurance_bps = insurance_bps;
		group.cycle_order = cycle_order;
		group.current_cycle_index = 0;
		group.total_cycles = group.cycle_order.len() as u32;
		group.usdc_mint = ctx.accounts.usdc_mint.key();
		group.contribution_amount = contribution_amount;

		// Record PDA bumps (already validated by the account constraints)
		group.group_vault_bump = ctx.bumps.group_vault_authority;
		group.insurance_vault_bump = ctx.bumps.insurance_vault_authority;
		Ok(())
	}

	pub fn join_group(ctx: Context<JoinGroup>) -> Result<()> {
		let member = &mut ctx.accounts.member_account;
		let group = &mut ctx.accounts.thrift_group;
		require_keys_eq!(group.usdc_mint, ctx.accounts.usdc_mint.key(), AhorroError::InvalidMint);
		require_keys_eq!(ctx.accounts.member.key(), ctx.accounts.member_signer.key(), AhorroError::Unauthorized);

		member.member = ctx.accounts.member.key();
		member.group = group.key();
		member.total_contributed = 0;
		member.has_received_payout = false;
		member.bump = ctx.bumps.member_account;
		Ok(())
	}

	pub fn make_contribution(ctx: Context<MakeContribution>) -> Result<()> {
		let group = &ctx.accounts.thrift_group;
		let member = &mut ctx.accounts.member_account;
		require_keys_eq!(member.group, group.key(), AhorroError::Unauthorized);
		require_keys_eq!(member.member, ctx.accounts.member.key(), AhorroError::Unauthorized);
		require!(ctx.accounts.member_ata.mint == group.usdc_mint, AhorroError::InvalidMint);
		require!(ctx.accounts.group_vault.mint == group.usdc_mint, AhorroError::InvalidMint);
		require!(ctx.accounts.insurance_vault.mint == group.usdc_mint, AhorroError::InvalidMint);

		let total = group.contribution_amount;
		let insurance_cut = (total as u128 * group.insurance_bps as u128 / 10_000u128) as u64;
		let to_pool = insurance_cut;
		let to_group = total.checked_sub(insurance_cut).ok_or(AhorroError::MathOverflow)?;

		// Transfer contribution minus insurance to group vault
		let cpi_ctx_group = CpiContext::new(
			ctx.accounts.token_program.to_account_info(),
			Transfer {
				from: ctx.accounts.member_ata.to_account_info(),
				to: ctx.accounts.group_vault.to_account_info(),
				authority: ctx.accounts.member.to_account_info(),
			},
		);
		token::transfer(cpi_ctx_group, to_group)?;

		// Transfer insurance cut to insurance pool vault
		let cpi_ctx_pool = CpiContext::new(
			ctx.accounts.token_program.to_account_info(),
			Transfer {
				from: ctx.accounts.member_ata.to_account_info(),
				to: ctx.accounts.insurance_vault.to_account_info(),
				authority: ctx.accounts.member.to_account_info(),
			},
		);
		token::transfer(cpi_ctx_pool, to_pool)?;

		member.total_contributed = member
			.total_contributed
			.checked_add(total)
			.ok_or(AhorroError::MathOverflow)?;
		Ok(())
	}

	pub fn disburse_payout(ctx: Context<DisbursePayout>) -> Result<()> {
		let group = &mut ctx.accounts.thrift_group;
		require_keys_eq!(ctx.accounts.authority.key(), group.authority, AhorroError::Unauthorized);
		let current_recipient = group
			.cycle_order
			.get(group.current_cycle_index as usize)
			.ok_or(AhorroError::InvalidState)?;
		require_keys_eq!(*current_recipient, ctx.accounts.recipient.key(), AhorroError::Unauthorized);

		// Transfer group vault amount to recipient ATA
		let amount = ctx.accounts.group_vault.amount;
		let group_key = group.key();
		let seeds: &[&[u8]] = &[
			GROUP_VAULT_SEED,
			group_key.as_ref(),
			&[group.group_vault_bump],
		];
		let signer_seeds: &[&[&[u8]]] = &[seeds];
		let cpi_ctx = CpiContext::new_with_signer(
			ctx.accounts.token_program.to_account_info(),
			Transfer {
				from: ctx.accounts.group_vault.to_account_info(),
				to: ctx.accounts.recipient_ata.to_account_info(),
				authority: ctx.accounts.group_vault_authority.to_account_info(),
			},
			signer_seeds,
		);
		token::transfer(cpi_ctx, amount)?;

		group.current_cycle_index = group
			.current_cycle_index
			.checked_add(1)
			.ok_or(AhorroError::MathOverflow)?;
		Ok(())
	}

	pub fn fallback_insurance_payout(ctx: Context<FallbackInsurancePayout>, amount: u64) -> Result<()> {
		let group = &ctx.accounts.thrift_group;
		require_keys_eq!(ctx.accounts.authority.key(), group.authority, AhorroError::Unauthorized);
		// Transfer from insurance vault to recipient in case of default
		let group_key = group.key();
		let seeds: &[&[u8]] = &[
			INSURANCE_VAULT_SEED,
			group_key.as_ref(),
			&[group.insurance_vault_bump],
		];
		let signer_seeds: &[&[&[u8]]] = &[seeds];
		let cpi_ctx = CpiContext::new_with_signer(
			ctx.accounts.token_program.to_account_info(),
			Transfer {
				from: ctx.accounts.insurance_vault.to_account_info(),
				to: ctx.accounts.recipient_ata.to_account_info(),
				authority: ctx.accounts.insurance_vault_authority.to_account_info(),
			},
			signer_seeds,
		);
		token::transfer(cpi_ctx, amount)?;
		Ok(())
	}
}

const MEMBERS_MAX: usize = 32;

#[account]
pub struct ThriftGroupAccount {
	pub authority: Pubkey,
	pub model_type: u8,
	pub insurance_bps: u16,
	pub cycle_order: Vec<Pubkey>,
	pub current_cycle_index: u32,
	pub total_cycles: u32,
	pub usdc_mint: Pubkey,
	pub contribution_amount: u64,
	pub group_vault_bump: u8,
	pub insurance_vault_bump: u8,
}

impl Space for ThriftGroupAccount {
	// 8 discriminator + 32 authority + 1 model + 2 bps + 4 vec len + 32 * MEMBERS_MAX + 4 curr + 4 total + 32 mint + 8 amount + 1 + 1 bumps
	const INIT_SPACE: usize = 8 + 32 + 1 + 2 + 4 + (32 * MEMBERS_MAX) + 4 + 4 + 32 + 8 + 1 + 1;
}

#[account]
pub struct MemberAccount {
	pub group: Pubkey,
	pub member: Pubkey,
	pub total_contributed: u64,
	pub has_received_payout: bool,
	pub bump: u8,
}

impl Space for MemberAccount {
	// 8 + 32 + 32 + 8 + 1 + 1
	const INIT_SPACE: usize = 8 + 32 + 32 + 8 + 1 + 1;
}

#[derive(Accounts)]
pub struct CreateGroup<'info> {
	#[account(mut)]
	pub authority: Signer<'info>,
	pub usdc_mint: Account<'info, Mint>,
	#[account(
		init,
		payer = authority,
		space = ThriftGroupAccount::INIT_SPACE,
	)]
	pub thrift_group: Account<'info, ThriftGroupAccount>,
	/// CHECK: PDA authority for the group vault
	#[account(seeds = [GROUP_VAULT_SEED, thrift_group.key().as_ref()], bump)]
	pub group_vault_authority: UncheckedAccount<'info>,
	/// CHECK: PDA authority for the insurance vault
	#[account(seeds = [INSURANCE_VAULT_SEED, thrift_group.key().as_ref()], bump)]
	pub insurance_vault_authority: UncheckedAccount<'info>,
	#[account(
		init,
		payer = authority,
		associated_token::mint = usdc_mint,
		associated_token::authority = group_vault_authority,
	)]
	pub group_vault: Account<'info, TokenAccount>,
	#[account(
		init,
		payer = authority,
		associated_token::mint = usdc_mint,
		associated_token::authority = insurance_vault_authority,
	)]
	pub insurance_vault: Account<'info, TokenAccount>,
	pub system_program: Program<'info, System>,
	pub token_program: Program<'info, Token>,
	pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct JoinGroup<'info> {
	pub usdc_mint: Account<'info, Mint>,
	#[account(mut)]
	pub member_signer: Signer<'info>,
	/// CHECK: Member public key (useful when joining on behalf of someone else in future)
	pub member: UncheckedAccount<'info>,
	#[account(mut)]
	pub thrift_group: Account<'info, ThriftGroupAccount>,
	#[account(
		init,
		payer = member_signer,
		space = MemberAccount::INIT_SPACE,
		seeds = [MEMBER_SEED, thrift_group.key().as_ref(), member.key().as_ref()],
		bump,
	)]
	pub member_account: Account<'info, MemberAccount>,
	pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MakeContribution<'info> {
	#[account(mut)]
	pub member: Signer<'info>,
	#[account(mut)]
	pub thrift_group: Account<'info, ThriftGroupAccount>,
	#[account(mut)]
	pub member_ata: Account<'info, TokenAccount>,
	#[account(mut)]
	pub group_vault: Account<'info, TokenAccount>,
	#[account(mut)]
	pub insurance_vault: Account<'info, TokenAccount>,
	pub usdc_mint: Account<'info, Mint>,
	#[account(
		mut,
		seeds = [MEMBER_SEED, thrift_group.key().as_ref(), member.key().as_ref()],
		bump = member_account.bump,
	)]
	pub member_account: Account<'info, MemberAccount>,
	pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct DisbursePayout<'info> {
	#[account(mut)]
	pub authority: Signer<'info>,
	#[account(mut)]
	pub thrift_group: Account<'info, ThriftGroupAccount>,
	/// CHECK: PDA or authority account controlling group vault
	#[account(seeds = [GROUP_VAULT_SEED, thrift_group.key().as_ref()], bump = thrift_group.group_vault_bump)]
	pub group_vault_authority: UncheckedAccount<'info>,
	#[account(mut)]
	pub group_vault: Account<'info, TokenAccount>,
	#[account(mut)]
	pub recipient: SystemAccount<'info>,
	#[account(mut)]
	pub recipient_ata: Account<'info, TokenAccount>,
	pub usdc_mint: Account<'info, Mint>,
	pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct FallbackInsurancePayout<'info> {
	#[account(mut)]
	pub authority: Signer<'info>,
	pub thrift_group: Account<'info, ThriftGroupAccount>,
	/// CHECK: PDA or authority account controlling insurance vault
	#[account(seeds = [INSURANCE_VAULT_SEED, thrift_group.key().as_ref()], bump = thrift_group.insurance_vault_bump)]
	pub insurance_vault_authority: UncheckedAccount<'info>,
	#[account(mut)]
	pub insurance_vault: Account<'info, TokenAccount>,
	#[account(mut)]
	pub recipient: SystemAccount<'info>,
	#[account(mut)]
	pub recipient_ata: Account<'info, TokenAccount>,
	pub usdc_mint: Account<'info, Mint>,
	pub token_program: Program<'info, Token>,
}

#[error_code]
pub enum AhorroError {
	#[msg("Invalid insurance basis points")] 
	InvalidInsuranceBps,
	#[msg("Invalid mint for this group")] 
	InvalidMint,
	#[msg("Math overflow")] 
	MathOverflow,
	#[msg("Unauthorized")] 
	Unauthorized,
	#[msg("Invalid state")] 
	InvalidState,
}
