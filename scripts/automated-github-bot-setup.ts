#!/usr/bin/env bun
/**
 * Automated GitHub Bot Setup
 *
 * This script orchestrates the complete GitHub App setup process:
 * 1. Generates the manifest URL for one-click app registration
 * 2. Guides you through the browser-based registration
 * 3. Uploads credentials to GCP Secret Manager
 * 4. Tests the setup by generating a token
 *
 * Usage:
 *   bun scripts/automated-github-bot-setup.ts
 *
 * Or with options:
 *   bun scripts/automated-github-bot-setup.ts \
 *     --org lornu-ai \
 *     --name lornu-ai-bot \
 *     --project <GCP_PROJECT_ID>
 */

import { spawn } from "child_process";
import { existsSync } from "fs";
import * as readline from "readline";

interface SetupArgs {
  org: string;
  name: string;
  project?: string;
}

function parseArgs(): SetupArgs {
  const args: Partial<SetupArgs> = {};

  for (let i = 2; i < process.argv.length; i++) {
    const arg = process.argv[i];
    if (arg === "--org" && i + 1 < process.argv.length) {
      args.org = process.argv[++i];
    } else if (arg === "--name" && i + 1 < process.argv.length) {
      args.name = process.argv[++i];
    } else if (arg === "--project" && i + 1 < process.argv.length) {
      args.project = process.argv[++i];
    }
  }

  args.org = args.org || process.env.GITHUB_ORG || "lornu-ai";
  args.name = args.name || process.env.GITHUB_APP_NAME || "lornu-ai-bot";
  args.project = args.project || process.env.GCP_PROJECT_ID;

  return args as SetupArgs;
}

function prompt(question: string): Promise<string> {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  return new Promise((resolve) => {
    rl.question(question, (answer) => {
      rl.close();
      resolve(answer.trim());
    });
  });
}

async function runTool(command: string, args: string[]): Promise<void> {
  return new Promise((resolve, reject) => {
    const proc = spawn(command, args, {
      stdio: "inherit",
      cwd: process.cwd(),
    });

    proc.on("close", (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`Process exited with code ${code}`));
      }
    });
  });
}

async function main() {
  const args = parseArgs();

  console.log("╔══════════════════════════════════════════════════════════════╗");
  console.log("║        🤖 Automated GitHub Bot Setup                         ║");
  console.log("╚══════════════════════════════════════════════════════════════╝");
  console.log("");

  // Step 1: Generate manifest URL
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("📋 STEP 1: Generate GitHub App Manifest URL");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("");

  await runTool("bun", [
    "ai-agents/github-bot-tools-ts/src/bin/create-manifest.ts",
    "--org", args.org,
    "--name", args.name,
  ]);

  console.log("");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("🌐 STEP 2: Register the App in Browser");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("");
  console.log("1. Copy the URL above and open it in your browser");
  console.log("2. Click 'Create GitHub App'");
  console.log("3. Install the app on your organization/repositories");
  console.log("4. Download the private key (.pem file)");
  console.log("5. Note the App ID and Installation ID from the app settings");
  console.log("");

  await prompt("Press Enter when you've completed the browser registration...");

  // Step 3: Collect credentials
  console.log("");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("🔑 STEP 3: Enter Credentials");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("");

  const appId = await prompt("Enter GitHub App ID: ");
  const installationId = await prompt("Enter Installation ID: ");
  const privateKeyPath = await prompt("Enter path to private key (.pem): ");

  if (!existsSync(privateKeyPath)) {
    console.error(`❌ Private key file not found: ${privateKeyPath}`);
    process.exit(1);
  }

  // Step 4: Upload to GCP Secret Manager (optional)
  if (args.project) {
    console.log("");
    console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    console.log("☁️  STEP 4: Upload to GCP Secret Manager");
    console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    console.log("");

    try {
      await runTool("bun", [
        "ai-agents/github-bot-tools-ts/src/bin/upload-secrets.ts",
        "--project", args.project,
        "--app-id", appId,
        "--installation-id", installationId,
        "--private-key-file", privateKeyPath,
      ]);
    } catch (error) {
      console.error("⚠️  Failed to upload to GCP. You can do this manually later.");
    }
  } else {
    console.log("");
    console.log("⏭️  Skipping GCP upload (no --project specified)");
    console.log("   Run upload-secrets.ts manually when ready:");
    console.log(`   bun ai-agents/github-bot-tools-ts/src/bin/upload-secrets.ts \\`);
    console.log(`     --project <GCP_PROJECT_ID> \\`);
    console.log(`     --app-id ${appId} \\`);
    console.log(`     --installation-id ${installationId} \\`);
    console.log(`     --private-key-file ${privateKeyPath}`);
  }

  // Step 5: Test the setup
  console.log("");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("🧪 STEP 5: Test Token Generation");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("");

  const testToken = await prompt("Would you like to test token generation? (y/n): ");

  if (testToken.toLowerCase() === "y") {
    try {
      await runTool("bun", [
        "ai-agents/github-bot-tools-ts/src/bin/generate-github-app-token.ts",
        "--app-id", appId,
        "--private-key", privateKeyPath,
        "--installation-id", installationId,
      ]);
      console.log("");
      console.log("✅ Token generation successful!");
    } catch (error) {
      console.error("❌ Token generation failed. Check your credentials.");
    }
  }

  // Summary
  console.log("");
  console.log("╔══════════════════════════════════════════════════════════════╗");
  console.log("║        ✅ Setup Complete!                                     ║");
  console.log("╚══════════════════════════════════════════════════════════════╝");
  console.log("");
  console.log("Your GitHub App is now configured. Here's a summary:");
  console.log("");
  console.log(`  App ID:          ${appId}`);
  console.log(`  Installation ID: ${installationId}`);
  console.log(`  Private Key:     ${privateKeyPath}`);
  if (args.project) {
    console.log(`  GCP Project:     ${args.project}`);
  }
  console.log("");
  console.log("To generate tokens:");
  console.log("  bun ai-agents/github-bot-tools-ts/src/bin/generate-github-app-token.ts");
  console.log("");
  console.log("To approve PRs:");
  console.log("  bun ai-agents/github-bot-tools-ts/src/bin/approve-prs-with-bot.ts \\");
  console.log("    --repo lornu-ai/lornu.ai --pr-number <PR>");
  console.log("");
  console.log("Or use the Rust tools:");
  console.log("  cargo run --bin get-token -p github-bot -- --app-id ...");
  console.log("  cargo run --bin approve-pr -p github-bot -- --repo ... --pr-number ...");
  console.log("");
}

main().catch((error) => {
  console.error("❌ Setup failed:", error.message);
  process.exit(1);
});
