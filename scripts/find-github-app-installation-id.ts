#!/usr/bin/env bun
/**
 * Find GitHub App Installation ID
 * 
 * This script helps you find the installation ID for a GitHub App.
 * 
 * Usage:
 *   bun scripts/find-github-app-installation-id.ts <APP_ID> <PRIVATE_KEY_PATH>
 * 
 * Example:
 *   bun scripts/find-github-app-installation-id.ts 123456 ./private-key.pem
 */

import { readFileSync, existsSync } from "fs";
import { createSign } from "crypto";

function generateJWT(appId: string, privateKeyPath: string): string {
  const now = Math.floor(Date.now() / 1000);
  const payload = {
    iat: now - 60, // Issued at (60 seconds ago to account for clock skew)
    exp: now + 600, // Expires in 10 minutes
    iss: appId, // Issuer (GitHub App ID)
  };
  
  const privateKey = readFileSync(privateKeyPath, "utf-8");
  
  const header = {
    alg: "RS256",
    typ: "JWT",
  };
  
  const encodedHeader = Buffer.from(JSON.stringify(header)).toString("base64url");
  const encodedPayload = Buffer.from(JSON.stringify(payload)).toString("base64url");
  const signature = createSign("RSA-SHA256")
    .update(`${encodedHeader}.${encodedPayload}`)
    .sign(privateKey, "base64url");
  
  return `${encodedHeader}.${encodedPayload}.${signature}`;
}

async function findInstallations(appId: string, privateKeyPath: string): Promise<void> {
  console.log("🔍 Finding GitHub App installations...\n");
  
  try {
    const jwt = generateJWT(appId, privateKeyPath);
    
    // Get installations via GitHub API
    const response = await fetch("https://api.github.com/app/installations", {
      headers: {
        Authorization: `Bearer ${jwt}`,
        Accept: "application/vnd.github.v3+json",
        "User-Agent": "lornu-ai-bot-setup",
      },
    });
    
    if (!response.ok) {
      const error = await response.text();
      throw new Error(`GitHub API error: ${response.status} ${error}`);
    }
    
    const installations = await response.json();
    
    if (installations.length === 0) {
      console.log("❌ No installations found for this GitHub App.");
      console.log("   Make sure the app is installed on at least one repository.\n");
      return;
    }
    
    console.log("✅ Found installations:\n");
    
    for (const installation of installations) {
      console.log(`Installation ID: ${installation.id}`);
      console.log(`  Account: ${installation.account?.login || "N/A"}`);
      console.log(`  Repository Selection: ${installation.repository_selection}`);
      console.log(`  Permissions: ${JSON.stringify(installation.permissions, null, 2)}`);
      
      // Get repositories if available
      if (installation.repository_selection === "selected") {
        try {
          const reposResponse = await fetch(
            `https://api.github.com/app/installations/${installation.id}/repositories`,
            {
              headers: {
                Authorization: `Bearer ${jwt}`,
                Accept: "application/vnd.github.v3+json",
                "User-Agent": "lornu-ai-bot-setup",
              },
            }
          );
          
          if (reposResponse.ok) {
            const repos = await reposResponse.json();
            console.log(`  Repositories (${repos.repositories?.length || 0}):`);
            for (const repo of repos.repositories || []) {
              console.log(`    - ${repo.full_name}`);
            }
          }
        } catch (error) {
          // Ignore errors fetching repositories
        }
      }
      
      console.log("");
    }
    
    console.log("To use an installation, set:");
    console.log(`  export GITHUB_APP_INSTALLATION_ID="<ID_FROM_ABOVE>"`);
  } catch (error) {
    console.error("❌ Error finding installations:", error instanceof Error ? error.message : error);
    process.exit(1);
  }
}

async function main() {
  const args = process.argv.slice(2);
  
  if (args.length < 2) {
    console.error("Usage: bun scripts/find-github-app-installation-id.ts <APP_ID> <PRIVATE_KEY_PATH>");
    console.error("");
    console.error("Example:");
    console.error("  bun scripts/find-github-app-installation-id.ts 123456 ./private-key.pem");
    process.exit(1);
  }
  
  const [appId, privateKeyPath] = args;
  
  if (!existsSync(privateKeyPath)) {
    console.error(`❌ Private key file not found: ${privateKeyPath}`);
    process.exit(1);
  }
  
  await findInstallations(appId, privateKeyPath);
}

main().catch((error) => {
  console.error("❌ Error:", error);
  process.exit(1);
});
