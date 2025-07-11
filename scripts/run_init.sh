#!/bin/bash

echo "🚀 Running HookSwap Devnet Initialization..."

# Set environment
export ANCHOR_PROVIDER_URL="https://api.devnet.solana.com"
export ANCHOR_WALLET="$HOME/.config/solana/id.json"

# Set Solana config
solana config set --url https://api.devnet.solana.com

echo "📡 Environment set:"
echo "  RPC: $ANCHOR_PROVIDER_URL"
echo "  Wallet: $ANCHOR_WALLET"
echo "  Balance: $(solana balance)"

echo ""
echo "🔨 Compiling and running initialization..."

# Compile and run
node initialize_devnet_system.js

echo ""
echo "✅ Initialization completed!"