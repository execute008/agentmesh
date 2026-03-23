#!/bin/bash
# submit-project.sh
# Helper script for AgentMesh Synthesis submission

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Read API key from .synthesis file
API_KEY=$(grep "API_KEY=" .synthesis | cut -d'=' -f2)
TEAM_ID=$(grep "TEAM_ID=" .synthesis | cut -d'=' -f2)

if [ -z "$API_KEY" ]; then
    echo -e "${RED}❌ API_KEY not found in .synthesis file${NC}"
    exit 1
fi

BASE_URL="https://synthesis.devfolio.co"

echo -e "${GREEN}🚀 AgentMesh Synthesis Submission Helper${NC}"
echo ""

# Function to make API calls
api_call() {
    local method=$1
    local endpoint=$2
    local data=$3
    
    if [ -z "$data" ]; then
        curl -s -X "$method" "$BASE_URL$endpoint" \
            -H "Authorization: Bearer $API_KEY" \
            -H "Content-Type: application/json"
    else
        curl -s -X "$method" "$BASE_URL$endpoint" \
            -H "Authorization: Bearer $API_KEY" \
            -H "Content-Type: application/json" \
            -d "$data"
    fi
}

# Menu
echo "Select an action:"
echo "1. Get team info"
echo "2. Browse tracks"
echo "3. Check self-custody status"
echo "4. Initiate self-custody transfer"
echo "5. Confirm self-custody transfer"
echo "6. Create project draft"
echo "7. Update project draft"
echo "8. Publish project"
echo "9. View project"
echo ""
read -p "Enter choice [1-9]: " choice

case $choice in
    1)
        echo -e "${YELLOW}📋 Fetching team info...${NC}"
        api_call "GET" "/teams/$TEAM_ID" | jq '.'
        ;;
    
    2)
        echo -e "${YELLOW}📋 Fetching available tracks...${NC}"
        api_call "GET" "/catalog?page=1&limit=50" | jq '.items[] | {uuid, name, company, description}'
        ;;
    
    3)
        echo -e "${YELLOW}🔍 Checking self-custody status...${NC}"
        api_call "GET" "/participants/me" | jq '{custodyType, ownerAddress, walletAddress, selfCustodyVerifiedAt}'
        ;;
    
    4)
        echo -e "${YELLOW}🔐 Initiating self-custody transfer...${NC}"
        read -p "Enter your wallet address (0x...): " wallet_address
        
        if [[ ! "$wallet_address" =~ ^0x[a-fA-F0-9]{40}$ ]]; then
            echo -e "${RED}❌ Invalid Ethereum address${NC}"
            exit 1
        fi
        
        response=$(api_call "POST" "/participants/me/transfer/init" "{\"targetOwnerAddress\":\"$wallet_address\"}")
        echo "$response" | jq '.'
        
        transfer_token=$(echo "$response" | jq -r '.transferToken')
        echo ""
        echo -e "${GREEN}✅ Transfer initiated${NC}"
        echo -e "${YELLOW}⚠️  VERIFY the targetOwnerAddress in the response above matches: $wallet_address${NC}"
        echo -e "${YELLOW}⚠️  Transfer token (save this): $transfer_token${NC}"
        echo ""
        echo "Next step: Run option 5 to confirm the transfer"
        ;;
    
    5)
        echo -e "${YELLOW}🔐 Confirming self-custody transfer...${NC}"
        read -p "Enter transfer token (from step 4): " transfer_token
        read -p "Enter wallet address (must match step 4): " wallet_address
        
        response=$(api_call "POST" "/participants/me/transfer/confirm" "{\"transferToken\":\"$transfer_token\",\"targetOwnerAddress\":\"$wallet_address\"}")
        echo "$response" | jq '.'
        
        if echo "$response" | jq -e '.status == "transfer_complete"' > /dev/null; then
            echo ""
            echo -e "${GREEN}✅ Self-custody transfer complete!${NC}"
            echo "Transaction: $(echo "$response" | jq -r '.txHash')"
            echo "You can now publish your project"
        else
            echo -e "${RED}❌ Transfer failed${NC}"
        fi
        ;;
    
    6)
        echo -e "${YELLOW}📝 Creating project draft...${NC}"
        
        # Check if required files exist
        if [ ! -f "CONVERSATION-LOG.md" ]; then
            echo -e "${YELLOW}⚠️  CONVERSATION-LOG.md not found. Run ./compile-conversation-log.sh first${NC}"
            read -p "Continue anyway? (y/n): " continue
            if [ "$continue" != "y" ]; then
                exit 0
            fi
        fi
        
        if [ ! -f "PROBLEM-STATEMENT.md" ]; then
            echo -e "${RED}❌ PROBLEM-STATEMENT.md not found${NC}"
            exit 1
        fi
        
        # Read problem statement
        problem_statement=$(cat PROBLEM-STATEMENT.md | jq -Rs .)
        
        # Read conversation log (or use placeholder)
        if [ -f "CONVERSATION-LOG.md" ]; then
            conversation_log=$(cat CONVERSATION-LOG.md | jq -Rs .)
        else
            conversation_log='"See CONVERSATION-LOG.md (to be compiled)"'
        fi
        
        echo ""
        echo "Enter track UUIDs (comma-separated, get from option 2):"
        read -p "Track UUIDs: " track_uuids
        
        # Convert comma-separated UUIDs to JSON array
        track_array=$(echo "$track_uuids" | jq -R 'split(",") | map(gsub("^\\s+|\\s+$";""))')
        
        echo ""
        read -p "Agent harness used (openclaw/claude-code/codex-cli/etc): " agent_harness
        read -p "AI model used (claude-sonnet-4-6/claude-opus-4-6/etc): " model
        read -p "Agent framework (elizaos/langchain/mastra/other): " agent_framework
        
        if [ "$agent_framework" = "other" ]; then
            read -p "Describe your framework: " agent_framework_other
            framework_field="\"agentFrameworkOther\":\"$agent_framework_other\","
        else
            framework_field=""
        fi
        
        echo ""
        read -p "Skills used (comma-separated, e.g., web-search,react-best-practices): " skills
        skills_array=$(echo "$skills" | jq -R 'split(",") | map(gsub("^\\s+|\\s+$";""))')
        
        read -p "Tools used (comma-separated, e.g., Foundry,Rust,ngrok): " tools
        tools_array=$(echo "$tools" | jq -R 'split(",") | map(gsub("^\\s+|\\s+$";""))')
        
        echo ""
        read -p "Video URL (YouTube/Loom, leave empty if not ready): " video_url
        read -p "Deployed URL (leave empty if none): " deployed_url
        read -p "Moltbook post URL (leave empty if not posted yet): " moltbook_url
        
        echo ""
        read -p "Post-hackathon intention (continuing/exploring/one-time): " intention
        
        # Build JSON payload
        video_field=""
        if [ -n "$video_url" ]; then
            video_field=",\"videoURL\":\"$video_url\""
        fi
        
        deployed_field=""
        if [ -n "$deployed_url" ]; then
            deployed_field=",\"deployedURL\":\"$deployed_url\""
        fi
        
        moltbook_field=""
        if [ -n "$moltbook_url" ]; then
            moltbook_field=",\"moltbookPostURL\":\"$moltbook_url\""
        fi
        
        payload=$(cat <<EOF
{
  "teamUUID": "$TEAM_ID",
  "name": "AgentMesh",
  "description": "Decentralized agent coordination protocol — deployable smart contract standard enabling any agent to create an on-chain mesh, discover peers via chain scanning, coordinate via x402 P2P WebSocket messaging, and settle payments with on-chain escrow and reputation.",
  "problemStatement": $problem_statement,
  "repoURL": "https://github.com/execute008/agentmesh",
  "trackUUIDs": $track_array,
  "conversationLog": $conversation_log,
  "submissionMetadata": {
    "agentFramework": "$agent_framework",
    $framework_field
    "agentHarness": "$agent_harness",
    "model": "$model",
    "skills": $skills_array,
    "tools": $tools_array,
    "helpfulResources": [
      "https://eips.ethereum.org/EIPS/eip-8004",
      "https://book.getfoundry.sh",
      "https://docs.rs/alloy"
    ],
    "intention": "$intention"
    $moltbook_field
  }
  $video_field
  $deployed_field
}
EOF
)
        
        echo ""
        echo -e "${YELLOW}Sending project creation request...${NC}"
        response=$(api_call "POST" "/projects" "$payload")
        echo "$response" | jq '.'
        
        project_uuid=$(echo "$response" | jq -r '.uuid')
        if [ "$project_uuid" != "null" ] && [ -n "$project_uuid" ]; then
            echo ""
            echo -e "${GREEN}✅ Project draft created!${NC}"
            echo "Project UUID: $project_uuid"
            echo "Status: $(echo "$response" | jq -r '.status')"
            echo ""
            echo "Next steps:"
            echo "1. Review the project at: $BASE_URL/projects/$project_uuid"
            echo "2. When ready, run option 8 to publish"
        else
            echo -e "${RED}❌ Project creation failed${NC}"
        fi
        ;;
    
    7)
        echo -e "${YELLOW}📝 Updating project draft...${NC}"
        read -p "Enter project UUID: " project_uuid
        
        echo "What do you want to update?"
        echo "1. Video URL"
        echo "2. Deployed URL"
        echo "3. Moltbook URL"
        echo "4. All submission metadata"
        read -p "Choice: " update_choice
        
        case $update_choice in
            1)
                read -p "New video URL: " video_url
                payload="{\"videoURL\":\"$video_url\"}"
                ;;
            2)
                read -p "New deployed URL: " deployed_url
                payload="{\"deployedURL\":\"$deployed_url\"}"
                ;;
            3)
                read -p "New Moltbook URL: " moltbook_url
                payload="{\"submissionMetadata\":{\"moltbookPostURL\":\"$moltbook_url\"}}"
                ;;
            *)
                echo "Manual update not implemented for this option yet"
                exit 0
                ;;
        esac
        
        response=$(api_call "POST" "/projects/$project_uuid" "$payload")
        echo "$response" | jq '.'
        ;;
    
    8)
        echo -e "${YELLOW}🚀 Publishing project...${NC}"
        read -p "Enter project UUID: " project_uuid
        
        echo ""
        echo -e "${RED}⚠️  WARNING: Only team admin can publish${NC}"
        echo -e "${RED}⚠️  All team members must have self-custody${NC}"
        echo -e "${RED}⚠️  Make sure video URL and all fields are final${NC}"
        echo ""
        read -p "Are you sure you want to publish? (yes/no): " confirm
        
        if [ "$confirm" != "yes" ]; then
            echo "Cancelled"
            exit 0
        fi
        
        response=$(api_call "POST" "/projects/$project_uuid/publish")
        echo "$response" | jq '.'
        
        if echo "$response" | jq -e '.status == "publish"' > /dev/null; then
            echo ""
            echo -e "${GREEN}✅ Project published!${NC}"
            echo "Slug: $(echo "$response" | jq -r '.slug')"
            echo ""
            echo -e "${YELLOW}Next step: Tweet about your project tagging @synthesis_md${NC}"
        else
            echo -e "${RED}❌ Publishing failed${NC}"
        fi
        ;;
    
    9)
        echo -e "${YELLOW}👁  Viewing project...${NC}"
        read -p "Enter project UUID: " project_uuid
        api_call "GET" "/projects/$project_uuid" | jq '.'
        ;;
    
    *)
        echo -e "${RED}Invalid choice${NC}"
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}Done!${NC}"
