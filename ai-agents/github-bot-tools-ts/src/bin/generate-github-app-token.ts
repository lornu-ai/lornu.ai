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
    }
  }
  
  // Support environment variables
  args.appId = args.appId || process.env.GITHUB_APP_ID || "";
  args.privateKey = args.privateKey || process.env.GITHUB_APP_PRIVATE_KEY_PATH || "";
  args.installationId = args.installationId || process.env.GITHUB_APP_INSTALLATION_ID || "";
  
  if (!args.appId || !args.privateKey || !args.installationId) {
    console.error("Usage: generate-github-app-token.ts --app-id <ID> --private-key <PATH> --installation-id <ID>");
    console.error("Or set: GITHUB_APP_ID, GITHUB_APP_PRIVATE_KEY_PATH, GITHUB_APP_INSTALLATION_ID");
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

async function getInstallationToken(
  jwt: string,
  installationId: string
): Promise<string> {
  const octokit = new Octokit({
    auth: jwt,
  });
  
  const response = await octokit.apps.createInstallationAccessToken({
    installation_id: parseInt(installationId, 10),
  });
  
  return response.data.token;
}

async function main() {
  try {
    const args = parseArgs();
    
    console.log("🔐 Generating JWT for GitHub App...");
    const jwt = generateJWT(args.appId, args.privateKey);
    
    console.log("🔑 Exchanging JWT for installation token...");
    const token = await getInstallationToken(jwt, args.installationId);
    
    if (args.output) {
      const { writeFileSync } = await import("fs");
      writeFileSync(args.output, token, { mode: 0o600 });
      console.log(`✅ Token saved to ${args.output}`);
    } else {
      console.log("✅ Installation token:");
      console.log(token);
    }
  } catch (error) {
    console.error("❌ Error:", error instanceof Error ? error.message : error);
    process.exit(1);
  }
}

main();
