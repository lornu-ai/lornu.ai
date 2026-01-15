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
  
  // Validate app ID format (should be numeric)
  if (args.appId && (isNaN(parseInt(args.appId, 10)) || parseInt(args.appId, 10) <= 0)) {
    console.error(`❌ Invalid app ID: ${args.appId}. Must be a positive number.`);
    process.exit(1);
  }
  
  // Validate installation ID format (should be numeric)
  if (args.installationId && (isNaN(parseInt(args.installationId, 10)) || parseInt(args.installationId, 10) <= 0)) {
    console.error(`❌ Invalid installation ID: ${args.installationId}. Must be a positive number.`);
    process.exit(1);
  }
  
  // Validate private key path exists
  if (args.privateKey) {
    try {
      const { accessSync, constants } = await import("fs");
      accessSync(args.privateKey, constants.F_OK | constants.R_OK);
    } catch (error) {
      console.error(`❌ Private key file not found or not readable: ${args.privateKey}`);
      process.exit(1);
    }
  }
  
  if (!args.appId || !args.privateKey || !args.installationId) {
    console.error("Usage: generate-github-app-token.ts --app-id <ID> --private-key <PATH> --installation-id <ID>");
    console.error("Or set: GITHUB_APP_ID, GITHUB_APP_PRIVATE_KEY_PATH, GITHUB_APP_INSTALLATION_ID");
    process.exit(1);
  }
  
  return args as Args;
}

function generateJWT(appId: string, privateKeyPath: string): string {
  try {
    const now = Math.floor(Date.now() / 1000);
    const payload = {
      iat: now - 60, // Issued at (60 seconds ago to account for clock skew)
      exp: now + 600, // Expires in 10 minutes
      iss: appId, // Issuer (GitHub App ID)
    };
    
    let privateKey: string;
    try {
      privateKey = readFileSync(privateKeyPath, "utf-8");
    } catch (error) {
      throw new Error(`Failed to read private key file: ${privateKeyPath}. ${error instanceof Error ? error.message : String(error)}`);
    }
    
    // Validate private key format (basic check)
    if (!privateKey.includes("BEGIN") || !privateKey.includes("PRIVATE KEY")) {
      throw new Error(`Invalid private key format. Expected PEM format with BEGIN/END markers.`);
    }
    
    const header = {
      alg: "RS256",
      typ: "JWT",
    };
    
    const encodedHeader = Buffer.from(JSON.stringify(header)).toString("base64url");
    const encodedPayload = Buffer.from(JSON.stringify(payload)).toString("base64url");
    
    let signature: string;
    try {
      signature = createSign("RSA-SHA256")
        .update(`${encodedHeader}.${encodedPayload}`)
        .sign(privateKey, "base64url");
    } catch (error) {
      throw new Error(`Failed to sign JWT. ${error instanceof Error ? error.message : String(error)}`);
    }
    
    return `${encodedHeader}.${encodedPayload}.${signature}`;
  } catch (error) {
    throw new Error(`JWT generation failed: ${error instanceof Error ? error.message : String(error)}`);
  }
}

async function getInstallationToken(
  jwt: string,
  installationId: string
): Promise<string> {
  const octokit = new Octokit({
    auth: jwt,
  });
  
  const installationIdNum = parseInt(installationId, 10);
  if (isNaN(installationIdNum) || installationIdNum <= 0) {
    throw new Error(`Invalid installation ID: ${installationId}`);
  }
  
  const maxRetries = 3;
  let lastError: any = null;
  
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      const response = await octokit.apps.createInstallationAccessToken({
        installation_id: installationIdNum,
      });
      
      if (!response.data.token) {
        throw new Error("Token not returned in response");
      }
      
      return response.data.token;
    } catch (error: any) {
      lastError = error;
      
      // Don't retry on client errors (4xx)
      if (error.status >= 400 && error.status < 500) {
        break;
      }
      
      // Retry on network/server errors (5xx, timeouts)
      if (attempt < maxRetries && (error.status >= 500 || error.code === "ETIMEDOUT" || error.code === "ECONNRESET")) {
        const delay = attempt * 1000; // Exponential backoff
        console.warn(`⚠️  Attempt ${attempt}/${maxRetries} failed, retrying in ${delay}ms...`);
        await new Promise(resolve => setTimeout(resolve, delay));
        continue;
      }
      
      throw error;
    }
  }
  
  if (lastError?.status === 401 || lastError?.status === 403) {
    throw new Error(`Authentication failed. Check app ID, installation ID, and private key. ${lastError.message || ""}`);
  }
  
  if (lastError?.status === 404) {
    throw new Error(`Installation not found. Check installation ID: ${installationId}`);
  }
  
  throw new Error(`Failed to get installation token: ${lastError?.message || String(lastError)}`);
}

async function main() {
  try {
    const args = parseArgs();
    
    console.log("🔐 Generating JWT for GitHub App...");
    let jwt: string;
    try {
      jwt = generateJWT(args.appId, args.privateKey);
    } catch (error) {
      console.error("❌ Failed to generate JWT:", error instanceof Error ? error.message : error);
      process.exit(1);
    }
    
    console.log("🔑 Exchanging JWT for installation token...");
    let token: string;
    try {
      token = await getInstallationToken(jwt, args.installationId);
    } catch (error) {
      console.error("❌ Failed to get installation token:", error instanceof Error ? error.message : error);
      process.exit(1);
    }
    
    if (args.output) {
      try {
        const { writeFileSync } = await import("fs");
        writeFileSync(args.output, token, { mode: 0o600 });
        console.log(`✅ Token saved to ${args.output}`);
      } catch (error) {
        console.error(`❌ Failed to write token to ${args.output}:`, error instanceof Error ? error.message : error);
        process.exit(1);
      }
    } else {
      console.log("✅ Installation token:");
      console.log(token);
    }
  } catch (error) {
    console.error("❌ Unexpected error:", error instanceof Error ? error.message : error);
    process.exit(1);
  }
}

main();
