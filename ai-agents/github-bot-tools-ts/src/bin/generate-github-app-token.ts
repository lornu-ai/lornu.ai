#!/usr/bin/env bun
/**
 * Generate GitHub App Installation Access Token
 * 
 * Creates a JWT for GitHub App authentication and exchanges it for an installation token.
 * This allows using a GitHub App (bot account) instead of a Personal Access Token.
 * 
 * Usage:
 *   bun src/bin/generate-github-app-token.ts \
 *     --app-id <APP_ID> \
 *     --private-key <PRIVATE_KEY_PATH> \
 *     --installation-id <INSTALLATION_ID>
 */

import { createSign } from "crypto";
import { readFileSync } from "fs";
import { Octokit } from "@octokit/rest";

interface Args {
  appId: string;
  privateKey: string;
  installationId: string;
  output?: string;
  quiet?: boolean;
}

function parseArgs(): Args {
  const args: Partial<Args> = {};
  
  for (let i = 2; i < process.argv.length; i++) {
    const arg = process.argv[i];
    if (arg === "--app-id" && i + 1 < process.argv.length) {
      args.appId = process.argv[++i];
    } else if (arg === "--private-key" && i + 1 < process.argv.length) {
      args.privateKey = process.argv[++i];
    } else if (arg === "--installation-id" && i + 1 < process.argv.length) {
      args.installationId = process.argv[++i];
    } else if (arg === "--output" && i + 1 < process.argv.length) {
      args.output = process.argv[++i];
    } else if (arg === "--quiet" || arg === "-q") {
      args.quiet = true;
    }
  }
  
  // Support environment variables
  args.appId = args.appId || process.env.GITHUB_APP_ID || "";
  args.privateKey = args.privateKey || process.env.GITHUB_APP_PRIVATE_KEY_PATH || "";
  args.installationId = args.installationId || process.env.GITHUB_APP_INSTALLATION_ID || "";
  
  if (!args.appId || !args.privateKey || !args.installationId) {
    console.error("❌ Missing required arguments");
    console.error("\nUsage:");
    console.error("  bun src/bin/generate-github-app-token.ts \\");
    console.error("    --app-id <APP_ID> \\");
    console.error("    --private-key <PATH_TO_PEM_FILE> \\");
    console.error("    --installation-id <INSTALLATION_ID>");
    console.error("\nOr set environment variables:");
    console.error("  export GITHUB_APP_ID=<APP_ID>");
    console.error("  export GITHUB_APP_PRIVATE_KEY_PATH=<PATH_TO_PEM_FILE>");
    console.error("  export GITHUB_APP_INSTALLATION_ID=<INSTALLATION_ID>");
    console.error("\nTo find your Installation ID:");
    console.error("  bun scripts/find-github-app-installation-id.ts --app-id <APP_ID> --private-key <KEY.pem>");
    process.exit(1);
  }
  
  return args as Args;
}

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

async function getInstallationIdFromSlug(
  jwt: string,
  installationSlug: string
): Promise<number> {
  const octokit = new Octokit({
    auth: jwt,
  });
  
  try {
    // First, try to get installation by slug directly via REST API
    // GitHub API accepts slugs in the URL path
    try {
      const response = await fetch(
        `https://api.github.com/app/installations/${installationSlug}`,
        {
          headers: {
            Authorization: `Bearer ${jwt}`,
            Accept: "application/vnd.github.v3+json",
            "User-Agent": "lornu-ai-bot",
          },
        }
      );
      
      if (response.ok) {
        const installation = await response.json();
        console.error(`✅ Found installation: ID=${installation.id}, Account=${installation.account?.login}`);
        return installation.id;
      } else if (response.status === 404) {
        // Installation not found by slug, try listing all
        console.error(`⚠️  Installation slug not found, listing all installations...`);
      }
    } catch (e) {
      // Continue to list all installations
    }
    
    // List all installations and find matching one
    let installations;
    try {
      const response = await octokit.apps.listInstallations();
      installations = response.data;
    } catch (error: any) {
      if (error.status === 401 || error.status === 403) {
        throw new Error(
          `Authentication failed (${error.status}). Please verify:\n` +
          `  1. App ID is correct: ${process.env.GITHUB_APP_ID || '<APP_ID>'}\n` +
          `  2. Private key matches the app\n` +
          `  3. Private key is in PKCS#8 format (downloaded from GitHub)\n` +
          `  4. JWT is being generated correctly`
        );
      }
      throw error;
    }
    
    if (installations.length === 0) {
      throw new Error(
        "No installations found. Make sure:\n" +
        "  1. The app is installed on at least one repository\n" +
        "  2. Go to: https://github.com/organizations/lornu-ai/settings/installations\n" +
        "  3. Install the app on 'lornu-ai/lornu.ai' repository"
      );
    }
    
    console.error(`📋 Found ${installations.length} installation(s):`);
    for (const inst of installations) {
      console.error(`   - ID: ${inst.id}, Account: ${inst.account?.login || 'N/A'}, Repos: ${inst.repository_selection}`);
    }
    
    // If only one installation, use it
    if (installations.length === 1) {
      console.error(`✅ Using the only installation: ID=${installations[0].id}`);
      return installations[0].id;
    }
    
    // Try to match by account name if slug looks like it could be an account
    for (const installation of installations) {
      if (installation.account?.login === installationSlug) {
        return installation.id;
      }
    }
    
    throw new Error(
      `Multiple installations found but couldn't match slug "${installationSlug}".\n` +
      `Please use the numeric Installation ID from the list above, or install the app on only one repository.`
    );
  } catch (error: any) {
    if (error.status === 401 || error.status === 403) {
      throw new Error(`Authentication failed. Please verify:\n  1. App ID is correct\n  2. Private key matches the app\n  3. Private key is in PKCS#8 format`);
    }
    throw new Error(`Failed to find installation: ${error.message}`);
  }
}

async function getInstallationToken(
  jwt: string,
  installationId: string
): Promise<string> {
  // First, try using the installation ID/slug directly via REST API
  // GitHub's REST API accepts installation slugs in the URL path
  try {
    const response = await fetch(
      `https://api.github.com/app/installations/${installationId}/access_tokens`,
      {
        method: "POST",
        headers: {
          Authorization: `Bearer ${jwt}`,
          Accept: "application/vnd.github.v3+json",
          "User-Agent": "lornu-ai-bot",
        },
      }
    );
    
    if (response.ok) {
      const data = await response.json();
      return data.token;
    } else if (response.status === 404) {
      // Installation not found, try to resolve slug to numeric ID
      console.error(`⚠️  Installation "${installationId}" not found, trying to resolve...`);
    } else {
      const errorText = await response.text();
      throw new Error(`GitHub API error (${response.status}): ${errorText}`);
    }
  } catch (error: any) {
    if (error.message && !error.message.includes("404")) {
      throw error;
    }
    // Fall through to try resolving slug
  }
  
  // If direct API call failed, try using Octokit with numeric ID resolution
  const octokit = new Octokit({
    auth: jwt,
  });
  
  try {
    // Check if installationId is numeric or a slug
    let installationIdNum: number;
    
    if (/^\d+$/.test(installationId)) {
      // It's a numeric ID
      installationIdNum = parseInt(installationId, 10);
    } else {
      // It's a slug, need to resolve to numeric ID
      // Use stderr for log messages
      console.error(`🔍 Resolving installation slug "${installationId}" to numeric ID...`);
      installationIdNum = await getInstallationIdFromSlug(jwt, installationId);
      console.error(`✅ Found numeric Installation ID: ${installationIdNum}`);
    }
    
    const response = await octokit.apps.createInstallationAccessToken({
      installation_id: installationIdNum,
    });
    
    return response.data.token;
  } catch (error: any) {
    if (error.status === 404 || error.message?.includes("not found")) {
      throw new Error(
        `Installation not found. Please verify:\n` +
        `  1. Installation ID/Slug is correct: ${installationId}\n` +
        `  2. App is installed on the repository/organization\n` +
        `  3. App ID is correct: ${process.env.GITHUB_APP_ID || '<APP_ID>'}\n` +
        `  4. Private key matches the app\n\n` +
        `To find your Installation ID, run:\n` +
        `  bun scripts/find-github-app-installation-id.ts --app-id ${process.env.GITHUB_APP_ID || '<APP_ID>'} --private-key <KEY.pem>`
      );
    }
    throw error;
  }
}

async function main() {
  try {
    const args = parseArgs();
    
    // Use stderr for log messages so stdout only contains the token
    const log = args.quiet ? () => {} : (msg: string) => console.error(msg);
    
    // All log messages go to stderr so stdout only contains the token
    console.error("🔐 Generating JWT for GitHub App...");
    const jwt = generateJWT(args.appId, args.privateKey);
    
    console.error("🔑 Exchanging JWT for installation token...");
    const token = await getInstallationToken(jwt, args.installationId);
    
    if (args.output) {
      const { writeFileSync } = await import("fs");
      writeFileSync(args.output, token, { mode: 0o600 });
      console.error(`✅ Token saved to ${args.output}`);
    } else {
      // Output only the token to stdout (for command substitution)
      // All log messages already go to stderr via console.error
      if (!args.quiet) {
        console.error("✅ Installation token:");
      }
      // Output token to stdout only - this is what gets captured by $()
      console.log(token);
    }
  } catch (error) {
    // Errors go to stderr
    console.error("❌ Error:", error instanceof Error ? error.message : error);
    process.exit(1);
  }
}

main();
