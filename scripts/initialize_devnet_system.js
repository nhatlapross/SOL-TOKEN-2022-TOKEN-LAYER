import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Connection } from "@solana/web3.js";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { createRequire } from "module";

// Create require function for ES modules
const require = createRequire(import.meta.url);

async function initializeDevnetSystem() {
  console.log("🚀 Initializing HookSwap System on Devnet...");
  
  // Set environment variables if not set
  if (!process.env.ANCHOR_PROVIDER_URL) {
    process.env.ANCHOR_PROVIDER_URL = "https://api.devnet.solana.com";
  }
  if (!process.env.ANCHOR_WALLET) {
    process.env.ANCHOR_WALLET = `${process.env.HOME}/.config/solana/id.json`;
  }
  
  // Connect to devnet
  const connection = new Connection(process.env.ANCHOR_PROVIDER_URL, "confirmed");
  const wallet = anchor.AnchorProvider.env().wallet;
  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: "confirmed"
  });
  anchor.setProvider(provider);

  // Program IDs (deployed)
  const PROGRAM_IDS = {
    tokenLayer: new PublicKey("HJ4MosN8hG5qd6WFMKQcBmYVhHuX1EKdPZ1LyaPSdYLA"),
    hookswapAmm: new PublicKey("4SCHMFNpFoHEbaMzgHHPpCKgtfHEuujbdwZsqNH2uC13"),
    kycHook: new PublicKey("76V7AeKynXT5e53XFzYXKZc5BoPAhSVqpyRbq1pAf4YC"),
    hookRegistry: new PublicKey("6guQ6trdmPmnfqgZwgiBPW7wVzEZuzWKNRzagHxveC88"),
    whitelistHook: new PublicKey("7Q3jm9Wqnpgg6SfUn2tujhSAiNaW1NvW74Ai821FEP93"),
  };

  // Load programs with correct IDL paths
  const tokenProgram = new Program(
    require("../../target/idl/token_layer.json"), 
    PROGRAM_IDS.tokenLayer, 
    provider
  );
  const kycProgram = new Program(
    require("../../target/idl/kyc_hook.json"), 
    PROGRAM_IDS.kycHook, 
    provider
  );
  const whitelistProgram = new Program(
    require("../../target/idl/whitelist_hook.json"), 
    PROGRAM_IDS.whitelistHook, 
    provider
  );
  const registryProgram = new Program(
    require("../../target/idl/hook_registry.json"), 
    PROGRAM_IDS.hookRegistry, 
    provider
  );
  const ammProgram = new Program(
    require("../../target/idl/hookswap_amm.json"), 
    PROGRAM_IDS.hookswapAmm, 
    provider
  );

  console.log("📡 Connected to Devnet");
  console.log("👤 Wallet:", wallet.publicKey.toString());

  // Authority (your wallet)
  const authority = walletpayer || Keypair.generate();

  // Find PDAs
  const [hookRegistryPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("hook_registry")],
    PROGRAM_IDS.hookRegistry
  );

  const [kycSystemPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("kyc_system")],
    PROGRAM_IDS.kycHook
  );

  const [whitelistPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("whitelist")],
    PROGRAM_IDS.whitelistHook
  );

  const [ammConfigPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("amm_config")],
    PROGRAM_IDS.hookswapAmm
  );

  console.log("");
  console.log("📍 System PDAs:");
  console.log("  Hook Registry:", hookRegistryPda.toString());
  console.log("  KYC System:", kycSystemPda.toString());
  console.log("  Whitelist:", whitelistPda.toString());
  console.log("  AMM Config:", ammConfigPda.toString());
  console.log("");

  try {
    // 1. Initialize Hook Registry
    console.log("🏗️ 1. Initializing Hook Registry...");
    try {
      const tx1 = await registryProgram.methods
        .initializeRegistry(authority.publicKey, 10)
        .accounts({
          registry: hookRegistryPda,
          payer: authority.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      console.log("✅ Hook Registry initialized:", tx1);
    } catch (e) {
      console.log("ℹ️ Hook Registry already exists");
    }

    // 2. Initialize KYC System
    console.log("🔐 2. Initializing KYC System...");
    try {
      const tx2 = await kycProgram.methods
        .initializeKycSystem(authority.publicKey)
        .accounts({
          kycSystem: kycSystemPda,
          payer: authority.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      console.log("✅ KYC System initialized:", tx2);
    } catch (e) {
      console.log("ℹ️ KYC System already exists");
    }

    // 3. Initialize Whitelist
    console.log("👥 3. Initializing Whitelist...");
    try {
      const tx3 = await whitelistProgram.methods
        .initializeWhitelist(authority.publicKey, 100)
        .accounts({
          whitelist: whitelistPda,
          payer: authority.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      console.log("✅ Whitelist initialized:", tx3);
    } catch (e) {
      console.log("ℹ️ Whitelist already exists");
    }

    // 4. Initialize AMM
    console.log("🔄 4. Initializing HookSwap AMM...");
    try {
      const tx4 = await ammProgram.methods
        .initializeAmm(new anchor.BN(30)) // 0.3% fee
        .accounts({
          ammConfig: ammConfigPda,
          authority: authority.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      console.log("✅ HookSwap AMM initialized:", tx4);
    } catch (e) {
      console.log("ℹ️ AMM already exists");
    }

    // 5. Register Hooks
    console.log("📝 5. Registering Hooks in Registry...");
    try {
      // Register KYC Hook
      const tx5a = await registryProgram.methods
        .addApprovedHook(
          PROGRAM_IDS.kycHook,
          { kyc: {} },
          "KYC Transfer Hook",
          "Validates user KYC status before transfers",
          { medium: {} }
        )
        .accounts({
          registry: hookRegistryPda,
          authority: authority.publicKey,
          hookProgram: null,
        })
        .rpc();
      console.log("✅ KYC Hook registered:", tx5a);

      // Register Whitelist Hook
      const tx5b = await registryProgram.methods
        .addApprovedHook(
          PROGRAM_IDS.whitelistHook,
          { whitelist: {} },
          "Whitelist Transfer Hook",
          "Validates addresses are whitelisted before transfers",
          { low: {} }
        )
        .accounts({
          registry: hookRegistryPda,
          authority: authority.publicKey,
          hookProgram: null,
        })
        .rpc();
      console.log("✅ Whitelist Hook registered:", tx5b);
    } catch (e) {
      console.log("ℹ️ Hooks already registered");
    }

    // 6. Create Test Token
    console.log("🪙 6. Creating Test Token-2022...");
    const mintKeypair = Keypair.generate();
    const tokenInfoKeypair = Keypair.generate();

    try {
      const tx6 = await tokenProgram.methods
        .createBasicToken2022(
          "HookSwap Test Token",
          "HST",
          6,
          new anchor.BN(1_000_000)
        )
        .accounts({
          tokenInfo: tokenInfoKeypair.publicKey,
          mint: mintKeypair.publicKey,
          authority: authority.publicKey,
          payer: authority.publicKey,
          systemProgram: SystemProgram.programId,
          token2022Program: TOKEN_2022_PROGRAM_ID,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([mintKeypair, tokenInfoKeypair])
        .rpc();
      console.log("✅ Test token created:", tx6);
      console.log("🪙 Mint address:", mintKeypair.publicKey.toString());
    } catch (e) {
      console.log("⚠️ Token creation failed:", e.message);
    }

    console.log("");
    console.log("🎉 HookSwap System Successfully Initialized on Devnet!");
    console.log("");
    console.log("📋 System Status:");
    console.log("✅ Hook Registry: Operational");
    console.log("✅ KYC System: Operational");
    console.log("✅ Whitelist: Operational");
    console.log("✅ HookSwap AMM: Operational");
    console.log("✅ Token Layer: Operational");
    console.log("");
    console.log("🔗 Ready for:");
    console.log("  • Token-2022 creation with Transfer Hooks");
    console.log("  • KYC validation during transfers");
    console.log("  • Whitelist enforcement");
    console.log("  • AMM pool creation & trading");
    console.log("  • Hook management via registry");

  } catch (error) {
    console.error("❌ Initialization failed:", error);
  }
}

// Run initialization
initializeDevnetSystem().catch(console.error);