# Advanced Vault - Solana Anchor Program

A sophisticated token staking vault built on Solana using the Anchor framework. This program allows users to stake tokens for time-based rewards with different multiplier rates.

## 🚀 Features

### Core Functionality
- **Token Staking**: Users can stake tokens for 1 or 2 years
- **Time-based Rewards**: Different multiplier rates based on staking duration
  - 1 year: 1x multiplier (no additional rewards)
  - 2 years: 2x multiplier (100% additional tokens)
- **Secure Withdrawal**: Tokens are locked until the staking period ends
- **Event Emission**: Comprehensive event logging for tracking stakes and withdrawals

### Program Structure
- **Vault Management**: Admin-controlled vault with PDA (Program Derived Address)
- **User Stakes**: Individual stake accounts for each user-vault combination
- **Token Integration**: Full SPL token support with associated token accounts

## 📋 Instructions

### 1. Create Vault
```rust
create_vault(ctx: Context<CreateVault>) -> Result<()>
```
- Creates a new vault controlled by an admin
- Uses PDA with admin's public key as seed
- Only the admin can create the vault

### 2. Stake Tokens
```rust
stake_tokens(ctx: Context<StakeTokens>, amount: u64, stake_years: u8) -> Result<()>
```
- Allows users to stake tokens for 1 or 2 years
- Transfers tokens from user to vault
- Creates a user stake account with lock period
- Emits `StakeCreatedEvent`

### 3. Withdraw Stake
```rust
withdraw_stake(ctx: &mut Context<WithdrawStake>) -> Result<()>
```
- Allows users to withdraw their staked tokens after lock period
- Calculates total return based on multiplier
- Transfers tokens back to user
- Marks stake as withdrawn
- Emits `StakeWithdrawnEvent`

## 🏗️ Account Structures

### Vault Account
```rust
pub struct Vault {
    pub admin: Pubkey,    // Vault administrator
    pub bump: u8,         // PDA bump seed
}
```

### UserStake Account
```rust
pub struct UserStake {
    pub user: Pubkey,         // User's public key
    pub amount: u64,          // Staked amount
    pub stake_years: u8,      // Staking duration (1 or 2 years)
    pub stake_time: i64,      // Timestamp when staked
    pub unlock_time: i64,     // Timestamp when tokens unlock
    pub is_withdrawn: bool,   // Withdrawal status
    pub bump: u8,            // PDA bump seed
}
```

## ⚠️ Potential Issues & Limitations

### 1. **Limited Reward Structure**
- Only 1x and 2x multipliers available
- No intermediate reward rates (e.g., 1.5x for 18 months)
- No compound interest or dynamic rates

### 2. **Fixed Time Periods**
- Only supports 1 or 2 year staking periods
- No flexibility for shorter or longer periods
- No early withdrawal with penalties

### 3. **No Emergency Mechanisms**
- No admin ability to pause vault operations
- No emergency withdrawal functionality
- No circuit breaker for security issues

### 4. **Limited Vault Features**
- Single vault per admin (no multiple vaults)
- No vault configuration options
- No fee collection mechanism

### 5. **Security Considerations**
- No reentrancy protection explicitly implemented
- No maximum stake limits
- No minimum stake requirements

### 6. **Token Limitations**
- Only supports single token type per vault
- No multi-token vault support
- No token whitelist/blacklist functionality

## 🔧 Development Setup

### Prerequisites
- Rust 1.70+
- Solana CLI
- Anchor CLI
- Node.js and Yarn

### Installation
```bash
# Clone the repository
git clone <repository-url>
cd adv-vault

# Install dependencies
yarn install

# Build the program
anchor build

# Run tests
anchor test
```

### Configuration
The program is configured for localnet by default. Update `Anchor.toml` for different networks:

```toml
[provider]
cluster = "localnet"  # Change to devnet, mainnet-beta, etc.
wallet = "~/.config/solana/id.json"
```

## 🧪 Testing

Run the test suite:
```bash
anchor test
```

Current tests are minimal and need expansion to cover:
- Vault creation scenarios
- Token staking with different amounts and periods
- Withdrawal timing and validation
- Error conditions and edge cases
- Event emission verification

## 📊 Usage Examples

### Creating a Vault
```typescript
const vault = await program.methods.createVault().accounts({
  admin: admin.publicKey,
  vault: vaultPda,
  systemProgram: SystemProgram.programId,
}).rpc();
```

### Staking Tokens
```typescript
const stake = await program.methods.stakeTokens(
  new BN(1000000), // 1 token (assuming 6 decimals)
  1 // 1 year stake
).accounts({
  vault: vaultPda,
  userStake: userStakePda,
  user: user.publicKey,
  userTokenAccount: userTokenAccount,
  vaultTokenAccount: vaultTokenAccount,
  mint: tokenMint,
  tokenProgram: TOKEN_PROGRAM_ID,
  associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  systemProgram: SystemProgram.programId,
}).rpc();
```

### Withdrawing Stakes
```typescript
const withdraw = await program.methods.withdrawStake().accounts({
  vault: vaultPda,
  userStake: userStakePda,
  user: user.publicKey,
  userTokenAccount: userTokenAccount,
  vaultTokenAccount: vaultTokenAccount,
  mint: tokenMint,
  tokenProgram: TOKEN_PROGRAM_ID,
  associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  systemProgram: SystemProgram.programId,
}).rpc();
```

## 🔍 Program ID

```
ZB1BxyVhCwFECQoV7bjoun2pMk1yPvz3PGVoKu4d4m5
```

## 📝 License

This project is licensed under the MIT License.

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## ⚠️ Disclaimer

This is a demonstration program and should not be used in production without thorough security audits and additional testing. The current implementation has several limitations and potential security considerations that should be addressed before production deployment.
