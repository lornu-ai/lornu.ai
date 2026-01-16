#!/usr/bin/env bun
/**
 * GitHub Bot Account Setup - Interactive Helper
 * 
 * This script helps you set up a GitHub bot account for PR approvals.
 * Supports both GitHub App and Personal Access Token methods.
 * 
 * Usage:
 *   bun scripts/setup-github-bot.ts
 */

import { readFileSync, existsSync } from "fs";
import { createInterface } from "readline";
import { execSync } from "child_process";

const rl = createInterface({
  input: process.stdin,
  output: process.stdout,
});

function question(prompt: string): Promise<string> {
  return new Promise((resolve) => {
    rl.question(prompt, resolve);
  });
}

function questionSecret(prompt: string): Promise<string> {
  return new Promise((resolve) => {
    process.stdout.write(prompt);
    process.stdin.setRawMode(true);
    process.stdin.resume();
    process.stdin.setEncoding("utf8");
    
    let input = "";
    process.stdin.on("data", (char: string) => {
      char = char.toString();
      
      switch (char) {
        case "\n":
        case "\r":
        case "\u0004":
          process.stdin.setRawMode(false);
          process.stdin.pause();
          process.stdout.write("\n");
          resolve(input);
          break;
        case "\u0003":
          process.exit();
          break;
        case "\u007f":
          if (input.length > 0) {
            input = input.slice(0, -1);
            process.stdout.write("\b \b");
          }
          break;
        default:
          input += char;
          process.stdout.write("*");
          break;
      }
    });
  });
}

async function checkGitHubCLI(): Promise<boolean> {
  try {
    execSync("gh --version", { stdio: "ignore" });
    return true;
  } catch {
    return false;
  }
}

async function checkGitHubAuth(): Promise<boolean> {
  try {
    execSync("gh auth status", { stdio: "ignore" });
    return true;
  } catch {
    return false;
  }
}

async function setupGitHubApp(): Promise<void> {
  console.log("\n📱 GitHub App Setup");
  console.log("===================\n");
  
  const currentUser = execSync("gh api user", { encoding: "utf-8" });
  const userLogin = JSON.parse(currentUser).login;
  
  console.log("Follow these steps:\n");
  console.log(`1. Go to: https://github.com/organizations/${userLogin}/settings/apps/new`);
  console.log("   (Or: Settings → Developer settings → GitHub Apps → New GitHub App)\n");
  console.log("2. Fill in app details:");
  console.log("   - Name: lornu-ai-bot (or your choice)");
  console.log("   - Homepage: https://lornu.ai");
  console.log("   - Webhook: Leave unchecked\n");
  console.log("3. Set permissions:");
  console.log("   - Pull requests: Read and write");
  console.log("   - Contents: Read (optional)\n");
  console.log("4. Install the app on your repositories\n");
  console.log("5. Generate a private key (save it securely!)\n");
  
  await question("Press Enter when you've completed the steps above...");
  
  const appId = await question("\nEnter GitHub App ID: ");
  const installationId = await question("Enter Installation ID: ");
  const privateKeyPath = await question("Enter path to private key (.pem file): ");
  
  if (!existsSync(privateKeyPath)) {
    console.error(`❌ Private key file not found: ${privateKeyPath}`);
    process.exit(1);
  }
  
  console.log("\n🧪 Testing GitHub App setup...\n");
  
  // Test token generation
  try {
    // Use the actual script to generate token
    const { execFileSync } = await import("child_process");
    
    // Run the generate-github-app-token script
    execFileSync(
      "bun",
      [
        "ai-agents/github-bot-tools-ts/src/bin/generate-github-app-token.ts",
        "--output",
        "/tmp/github-bot-token-test",
      ],
      {
        encoding: "utf-8",
        env: {
          ...process.env,
          GITHUB_APP_ID: appId,
          GITHUB_APP_INSTALLATION_ID: installationId,
          GITHUB_APP_PRIVATE_KEY_PATH: privateKeyPath,
        },
        cwd: process.cwd(),
        stdio: "pipe",
      }
    );
    
    const tokenPath = "/tmp/github-bot-token-test";
    
    console.log("✅ GitHub App setup successful!\n");
    console.log(`Token saved to: ${tokenPath}\n`);
    console.log("To use in the future, set these environment variables:");
    console.log(`  export GITHUB_APP_ID="${appId}"`);
    console.log(`  export GITHUB_APP_INSTALLATION_ID="${installationId}"`);
    console.log(`  export GITHUB_APP_PRIVATE_KEY_PATH="${privateKeyPath}"\n`);
    console.log("Or store in GCP Secret Manager:");
    console.log(`  gcloud secrets create github-app-id --data-file=- <<< "${appId}"`);
    console.log(`  gcloud secrets create github-app-installation-id --data-file=- <<< "${installationId}"`);
    console.log(`  gcloud secrets create github-app-private-key --data-file=- < "${privateKeyPath}"`);
  } catch (error) {
    console.error("❌ GitHub App setup failed:", error instanceof Error ? error.message : error);
    console.error("\nCheck your credentials and try again.");
    process.exit(1);
  }
}

async function setupPersonalAccessToken(): Promise<void> {
  console.log("\n🔑 Personal Access Token Setup");
  console.log("==============================\n");
  
  console.log("Follow these steps:\n");
  console.log("1. Create a bot user account on GitHub (or use existing)\n");
  console.log("2. Go to: https://github.com/settings/tokens/new");
  console.log("   (Settings → Developer settings → Personal access tokens → Tokens (classic))\n");
  console.log("3. Generate a new token with:");
  console.log("   - repo (Full control of private repositories)");
  console.log("   - write:org (if needed for org operations)\n");
  console.log("4. Copy the token (you won't see it again!)\n");
  
  await question("Press Enter when you have the token...");
  
  const token = await questionSecret("\nEnter Personal Access Token: ");
  
  if (!token || token.trim().length === 0) {
    console.error("❌ Token cannot be empty");
    process.exit(1);
  }
  
  console.log("\n🧪 Testing token...\n");
  
  // Test token by making an API call
  try {
    process.env.GITHUB_TOKEN = token;
    const result = execSync("gh api user", { encoding: "utf-8", env: process.env });
    const user = JSON.parse(result);
    
    console.log(`✅ Token is valid! Authenticated as: ${user.login}\n`);
    console.log("To use in the future, set:");
    console.log(`  export GITHUB_TOKEN="${token}"\n`);
    console.log("Or store in GCP Secret Manager:");
    console.log(`  gcloud secrets create github-bot-token --data-file=- <<< "${token}"`);
  } catch (error) {
    console.error("❌ Token validation failed:", error instanceof Error ? error.message : error);
    console.error("\nCheck your token and try again.");
    process.exit(1);
  }
}

async function main() {
  console.log("🔧 GitHub Bot Account Setup");
  console.log("==========================\n");
  
  // Check prerequisites
  if (!(await checkGitHubCLI())) {
    console.error("❌ GitHub CLI (gh) is not installed");
    console.error("   Install it: https://cli.github.com/");
    process.exit(1);
  }
  
  if (!(await checkGitHubAuth())) {
    console.error("⚠️  Not authenticated with GitHub CLI");
    console.error("   Run: gh auth login");
    process.exit(1);
  }
  
  console.log("Choose setup method:");
  console.log("1) GitHub App (Recommended - more secure, fine-grained permissions)");
  console.log("2) Personal Access Token (Classic - simpler setup)");
  
  const choice = await question("\nEnter choice [1-2]: ");
  
  switch (choice.trim()) {
    case "1":
      await setupGitHubApp();
      break;
    case "2":
      await setupPersonalAccessToken();
      break;
    default:
      console.error("❌ Invalid choice");
      process.exit(1);
  }
  
  console.log("\n✅ Setup complete!\n");
  console.log("Test approving a PR:");
  console.log("  cd ai-agents/github-bot-tools-ts");
  console.log("  bun src/bin/approve-prs-with-bot.ts --pr-number <PR_NUMBER>");
  
  rl.close();
}

main().catch((error) => {
  console.error("❌ Error:", error);
  process.exit(1);
});
