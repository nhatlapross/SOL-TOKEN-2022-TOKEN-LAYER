# HookSwap AMM - Smart Contracts

HookSwap AMM is an advanced Automated Market Maker (AMM) built on Solana with full Token-2022 and Transfer Hooks support. This project enables trading of tokens with special features like KYC compliance, whitelist validation, and custom transfer hooks.

## 🏗️ Architecture Overview

```
Smart Contract Layer
├── hookswap-amm/          # Core AMM program
├── hook-registry/         # Hook validation & management
├── kyc-hook/             # KYC compliance hook
├── whitelist-hook/       # Whitelist validation hook
└── token-layer/          # Token-2022 creation utilities
```

## 📁 Project Structure

```
hookswap_amm/
├── programs/
│   ├── hookswap-amm/         # Main AMM contract
│   │   ├── src/
│   │   │   ├── lib.rs        # Core AMM logic
│   │   │   ├── state.rs      # Data structures
│   │   │   └── errors.rs     # Error definitions
│   │   └── Cargo.toml
│   │
│   ├── hook-registry/        # Hook management system
│   │   ├── src/
│   │   │   ├── lib.rs        # Registry logic
│   │   │   └── state.rs      # Hook metadata
│   │   └── Cargo.toml
│   │
│   ├── kyc-hook/            # KYC compliance hook
│   │   ├── src/
│   │   │   ├── lib.rs        # KYC validation logic
│   │   │   └── state.rs      # KYC records
│   │   └── Cargo.toml
│   │
│   ├── whitelist-hook/      # Whitelist validation hook
│   │   └── src/lib.rs
│   │
│   └── token-layer/         # Token-2022 utilities
│       └── src/lib.rs
│
├── target/                  # Compiled programs
├── tests/                   # Integration tests
├── migrations/              # Deployment scripts
├── Anchor.toml             # Anchor configuration
└── Cargo.toml              # Rust workspace
```

## 🚀 Programs Overview

### 1. HookSwap AMM Program
**Program ID**: `4SCHMFNpFoHEbaMzgHHPpCKgtfHEuujbdwZsqNH2uC13`

Core AMM functionality with Token-2022 and Transfer Hooks support.

**Main Features:**
- Create and manage liquidity pools
- Token swaps with hook validation
- Add/remove liquidity
- Hook compatibility checking

**Instructions:**
```rust
// Initialize AMM system
initialize_amm(fee_rate: u64) -> Result<()>

// Create new trading pool
create_pool(initial_price: u64) -> Result<()>

// Execute token swap with hook validation
swap(amount_in: u64, minimum_amount_out: u64, a_to_b: bool) -> Result<()>

// Add liquidity to existing pool
add_liquidity(amount_a: u64, amount_b: u64) -> Result<()>
```

### 2. Hook Registry Program
**Program ID**: `6guQ6trdmPmnfqgZwgiBPW7wVzEZuzWKNRzagHxveC88`

Manages approved hook programs and their metadata.

**Main Features:**
- Approve/remove hook programs
- Validate hook compatibility
- Track hook usage statistics
- Risk assessment and classification

**Instructions:**
```rust
// Initialize hook registry
initialize_registry(authority: Pubkey, max_hooks: u16) -> Result<()>

// Add approved hook
add_approved_hook(
    hook_program_id: Pubkey,
    hook_type: HookType,
    name: String,
    risk_level: RiskLevel
) -> Result<()>

// Validate hook is approved
is_hook_approved(hook_program_id: Pubkey) -> Result<bool>
```

### 3. KYC Hook Program
**Program ID**: `76V7AeKynXT5e53XFzYXKZc5BoPAhSVqpyRbq1pAf4YC`

Transfer hook that enforces KYC compliance for token transfers.

**Main Features:**
- Validate user KYC status
- Enforce compliance rules
- Track verification levels
- Fallback support for Transfer Hook Interface

### 4. Whitelist Hook Program
**Program ID**: `7Q3jm9Wqnpgg6SfUn2tujhSAiNaW1NvW74Ai821FEP93`

Transfer hook for whitelist-based permission validation.

**Main Features:**
- Whitelist address validation
- Permission-based transfers
- Dynamic whitelist management

## 🔧 Development Setup

### Prerequisites
- Rust 1.70+
- Solana CLI 1.16+
- Anchor Framework 0.28+
- Node.js 18+ (for tests)

### Installation

1. **Clone repository**
```bash
git clone <repository-url>
cd hookswap_amm
```

2. **Install dependencies**
```bash
# Install Rust dependencies
cargo install --locked

# Install Anchor
npm install -g @coral-xyz/anchor-cli

# Install Node dependencies for tests
npm install
```

3. **Build programs**
```bash
# Build all programs
anchor build

# Build specific program
cargo build-bpf --manifest-path programs/hookswap-amm/Cargo.toml
```

## 🧪 Testing

### Unit Tests
```bash
# Run Rust unit tests
cargo test

# Run specific program tests
cargo test --manifest-path programs/hookswap-amm/Cargo.toml
```

### Integration Tests
```bash
# Run Anchor integration tests
anchor test

# Run specific test file
anchor test --skip-local-validator tests/hookswap-amm.ts
```

### Local Development
```bash
# Start local validator
solana-test-validator

# Deploy to local
anchor deploy --provider.cluster localnet

# Run tests against local
anchor test --skip-local-validator
```

## 🚀 Deployment

### Devnet Deployment
```bash
# Configure for devnet
solana config set --url devnet
anchor deploy --provider.cluster devnet
```

### Program Addresses
- **Devnet Addresses**: See `Anchor.toml`
- **Mainnet**: Not yet deployed

## 📖 Usage Examples

### 1. Creating a Token-2022 Pool
```typescript
import { HookSwapSDK } from './sdk';

const sdk = new HookSwapSDK(connection, wallet);

// Create pool with KYC token
const signature = await sdk.createPool({
  tokenA: kycTokenMint,
  tokenB: solMint,
  initialPrice: 1000000000, // 1.0 in lamports
});
```

### 2. Executing Swaps
```typescript
// Swap with hook validation
const swapResult = await sdk.executeSwap({
  tokenA: kycTokenMint.toString(),
  tokenB: solMint.toString(),
  amountIn: 1000000, // 1 token
  minimumAmountOut: 900000, // 0.9 tokens min
  aToB: true
});
```

### 3. Hook Validation
```typescript
// Check hook compatibility
const isValid = await sdk.validateHookCompatibility(
  tokenMint,
  recipientWallet
);
```

## 🔐 Security Features

### Hook Validation Strategy
- **Whitelist Approach**: Only allow verified hook programs
- **Simulation First**: Test hook execution before actual swap
- **Fallback Mechanism**: Backup when hooks fail
- **Rate Limiting**: Prevent spam and abuse

### Smart Contract Security
- **Comprehensive Testing**: Anchor framework testing
- **Audit-Ready Code**: Clean, documented code structure
- **Proper Error Handling**: Detailed error messages
- **Access Control**: Authority-based permissions

## 🎯 Key Features

### ✅ Token-2022 Support
- Native Token-2022 program integration
- Transfer Hook compatibility
- Extension support (hooks, metadata, etc.)

### ✅ Advanced Hook System
- Multi-hook support on single token
- Hook registry and validation
- Dynamic hook approval/removal

### ✅ Compliance Ready
- KYC/AML integration
- Whitelist management
- Audit trail and logging

### ✅ Developer Friendly
- TypeScript SDK
- Comprehensive documentation
- Example implementations

## 🛠️ Configuration

### Anchor.toml
```toml
[features]
seeds = false
skip-lint = false

[programs.devnet]
hookswap_amm = "4SCHMFNpFoHEbaMzgHHPpCKgtfHEuujbdwZsqNH2uC13"
hook_registry = "6guQ6trdmPmnfqgZwgiBPW7wVzEZuzWKNRzagHxveC88"
kyc_hook = "76V7AeKynXT5e53XFzYXKZc5BoPAhSVqpyRbq1pAf4YC"
whitelist_hook = "7Q3jm9Wqnpgg6SfUn2tujhSAiNaW1NvW74Ai821FEP93"

[provider]
cluster = "devnet"
wallet = "~/.config/solana/id.json"
```

## 📊 Program Data Structures

### Pool Account
```rust
pub struct Pool {
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub creator: Pubkey,
    pub fee_rate: u64,
    pub current_price: u64,
    pub total_liquidity_a: u64,
    pub total_liquidity_b: u64,
    pub hook_enabled: bool,
    pub bump: u8,
}
```

### Hook Registry
```rust
pub struct HookRegistry {
    pub authority: Pubkey,
    pub approved_hooks: Vec<Pubkey>,
    pub hook_metadata: Vec<HookMetadata>,
    pub total_validations: u64,
    pub is_enabled: bool,
}
```

## 🔍 Troubleshooting

### Common Issues

1. **Program not found**
   - Verify program IDs in `Anchor.toml`
   - Check network configuration

2. **Hook validation failed**
   - Ensure hook is approved in registry
   - Check KYC/whitelist status

3. **Token account issues**
   - Verify Token-2022 program usage
   - Check associated token accounts

### Debug Commands
```bash
# Check program logs
solana logs <program-id>

# Verify account data
solana account <account-address>

# Check transaction details
solana confirm <transaction-signature> -v
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Write tests for new functionality
4. Submit a pull request

### Development Guidelines
- Follow Rust best practices
- Add comprehensive tests
- Document all public functions
- Update README for new features

## 📈 Roadmap

### Phase 1 (Current)
- ✅ Core AMM functionality
- ✅ Hook registry system
- ✅ KYC and whitelist hooks
- ✅ Token-2022 integration

### Phase 2 (Coming Soon)
- 🔄 Advanced pricing algorithms
- 🔄 Multi-hop swaps
- 🔄 Governance token integration
- 🔄 Yield farming features

### Phase 3 (Future)
- 📋 Cross-chain bridge support
- 📋 Advanced analytics
- 📋 Mobile SDK
- 📋 Institutional features

## 📚 API Reference

### HookSwap AMM Instructions

| Instruction | Description | Parameters |
|-------------|-------------|------------|
| `initialize_amm` | Initialize AMM system | `fee_rate: u64` |
| `create_pool` | Create liquidity pool | `initial_price: u64` |
| `swap` | Execute token swap | `amount_in: u64, min_out: u64, a_to_b: bool` |
| `add_liquidity` | Add liquidity to pool | `amount_a: u64, amount_b: u64` |

### Hook Registry Instructions

| Instruction | Description | Parameters |
|-------------|-------------|------------|
| `initialize_registry` | Setup hook registry | `authority: Pubkey, max_hooks: u16` |
| `add_approved_hook` | Approve new hook | `hook_id: Pubkey, metadata: HookMetadata` |
| `is_hook_approved` | Check hook status | `hook_program_id: Pubkey` |

## 📄 License

MIT License - see the LICENSE file for details.

## 🔗 Links

- **Documentation**: [Link to comprehensive docs]
- **SDK Repository**: [TypeScript SDK repo]
- **Live Demo**: [Interactive demo]
- **Discord Community**: [Join our Discord]
- **Twitter**: [@HookSwapAMM]

---

**Built with ❤️ for the Solana DeFi ecosystem**

*Enabling compliant and feature-rich token trading on Solana*
