#!/bin/bash

echo "🎬 HookSwap Devnet Demo - Full Workflow"
echo "======================================="

# Set devnet
solana config set --url https://api.devnet.solana.com

echo ""
echo "📋 Deployed Programs:"
echo "🪙 Token Layer: HJ4MosN8hG5qd6WFMKQcBmYVhHuX1EKdPZ1LyaPSdYLA"
echo "🔄 HookSwap AMM: 4SCHMFNpFoHEbaMzgHHPpCKgtfHEuujbdwZsqNH2uC13"  
echo "🔐 KYC Hook: 76V7AeKynXT5e53XFzYXKZc5BoPAhSVqpyRbq1pAf4YC"
echo "📝 Hook Registry: 6guQ6trdmPmnfqgZwgiBPW7wVzEZuzWKNRzagHxveC88"
echo "👥 Whitelist Hook: 7Q3jm9Wqnpgg6SfUn2tujhSAiNaW1NvW74Ai821FEP93"

echo ""
echo "🔗 Network: Solana Devnet"
echo "📡 RPC: https://api.devnet.solana.com"

echo ""
echo "💰 Checking wallet balance..."
solana balance

echo ""
echo "🚀 Next Steps to Test:"
echo "1. Run: node initialize_devnet_system.js"
echo "2. Test Token-2022 creation with hooks"
echo "3. Test KYC validation during transfers"
echo "4. Test whitelist enforcement"
echo "5. Test AMM pool creation & trading"

echo ""
echo "🎯 Demo Capabilities:"
echo "✅ Real Token-2022 with Transfer Hooks"
echo "✅ KYC validation blocking transfers"  
echo "✅ Whitelist address enforcement"
echo "✅ AMM trading with hook validation"
echo "✅ Hook management via registry"

echo ""
echo "🌐 Explorer Links:"
echo "Token Layer: https://explorer.solana.com/address/HJ4MosN8hG5qd6WFMKQcBmYVhHuX1EKdPZ1LyaPSdYLA?cluster=devnet"
echo "HookSwap AMM: https://explorer.solana.com/address/4SCHMFNpFoHEbaMzgHHPpCKgtfHEuujbdwZsqNH2uC13?cluster=devnet"
echo "KYC Hook: https://explorer.solana.com/address/76V7AeKynXT5e53XFzYXKZc5BoPAhSVqpyRbq1pAf4YC?cluster=devnet"
echo "Hook Registry: https://explorer.solana.com/address/6guQ6trdmPmnfqgZwgiBPW7wVzEZuzWKNRzagHxveC88?cluster=devnet"
echo "Whitelist Hook: https://explorer.solana.com/address/7Q3jm9Wqnpgg6SfUn2tujhSAiNaW1NvW74Ai821FEP93?cluster=devnet"

echo ""
echo "🎉 HookSwap Ecosystem Live on Devnet!"