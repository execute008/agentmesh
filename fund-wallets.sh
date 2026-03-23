#!/bin/bash
# Fund the 3 agent wallets with Sepolia ETH

set -e

RPC="https://eth-sepolia.g.alchemy.com/v2/LfeDCiDvCFTxjI4RkPToA"
AMOUNT="0.005ether"

echo "💰 Funding agent wallets with $AMOUNT each..."

# Scraper
echo "  📤 Funding scraper (0xBEEa...535e)..."
cast send 0xBEEac102BeB0a805aAAef7Fa0C147023442E535e \
  --value $AMOUNT \
  --rpc-url $RPC \
  --account deployer

# Analyzer
echo "  📤 Funding analyzer (0x56F4...fbA2)..."
cast send 0x56F4462c17711403E7a04C015fF5b92841b7fbA2 \
  --value $AMOUNT \
  --rpc-url $RPC \
  --account deployer

# Publisher
echo "  📤 Funding publisher (0xaDa0...d8bA)..."
cast send 0xaDa0dfF4522D26573c6DFd6E0F65095764FCd8bA \
  --value $AMOUNT \
  --rpc-url $RPC \
  --account deployer

echo ""
echo "✅ All wallets funded! Waiting 15s for confirmations..."
sleep 15

echo ""
echo "📊 Final balances:"
echo "  Scraper:   $(cast balance 0xBEEac102BeB0a805aAAef7Fa0C147023442E535e --rpc-url $RPC --ether) ETH"
echo "  Analyzer:  $(cast balance 0x56F4462c17711403E7a04C015fF5b92841b7fbA2 --rpc-url $RPC --ether) ETH"
echo "  Publisher: $(cast balance 0xaDa0dfF4522D26573c6DFd6E0F65095764FCd8bA --rpc-url $RPC --ether) ETH"
