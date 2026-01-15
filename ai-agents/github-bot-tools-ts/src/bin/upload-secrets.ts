#!/usr/bin/env bun
/**
 * GCP Secret Manager Upload Tool
 *
 * Uploads GitHub App credentials (App ID, Installation ID, Private Key)
 * to Google Cloud Secret Manager for secure storage and retrieval.
 *
 * Usage:
 *   bun src/bin/upload-secrets.ts \
 *     --project <GCP_PROJECT_ID> \
 *     --app-id <GITHUB_APP_ID> \
 *     --installation-id <INSTALLATION_ID> \
 *     --private-key-file <PATH_TO_PEM>
 *
 * Environment Variables:
 *   GCP_PROJECT_ID - Google Cloud project ID
 *   GITHUB_APP_ID - GitHub App ID
 *   GITHUB_APP_INSTALLATION_ID - Installation ID
 *   GITHUB_APP_PRIVATE_KEY_PATH - Path to .pem file
 */

import { SecretManagerServiceClient } from "@google-cloud/secret-manager";
import { readFileSync, existsSync } from "fs";

interface Args {
  project: string;
  appId: string;
  installationId: string;
  privateKeyFile: string;
  prefix?: string;
}

function parseArgs(): Args {
  const args: Partial<Args> = {};

  for (let i = 2; i < process.argv.length; i++) {
    const arg = process.argv[i];
    if (arg === "--project" && i + 1 < process.argv.length) {
      args.project = process.argv[++i];
    } else if (arg === "--app-id" && i + 1 < process.argv.length) {
      args.appId = process.argv[++i];
    } else if (arg === "--installation-id" && i + 1 < process.argv.length) {
      args.installationId = process.argv[++i];
    } else if (arg === "--private-key-file" && i + 1 < process.argv.length) {
      args.privateKeyFile = process.argv[++i];
    } else if (arg === "--prefix" && i + 1 < process.argv.length) {
      args.prefix = process.argv[++i];
    } else if (arg === "--help" || arg === "-h") {
      printUsage();
      process.exit(0);
    }
  }

  // Support environment variables
  args.project = args.project || process.env.GCP_PROJECT_ID || "";
  args.appId = args.appId || process.env.GITHUB_APP_ID || "";
  args.installationId = args.installationId || process.env.GITHUB_APP_INSTALLATION_ID || "";
  args.privateKeyFile = args.privateKeyFile || process.env.GITHUB_APP_PRIVATE_KEY_PATH || "";
  args.prefix = args.prefix || "lornu";

  if (!args.project || !args.appId || !args.installationId || !args.privateKeyFile) {
    console.error("❌ Missing required arguments.\n");
    printUsage();
    process.exit(1);
  }

  if (!existsSync(args.privateKeyFile)) {
    console.error(`❌ Private key file not found: ${args.privateKeyFile}`);
    process.exit(1);
  }

  return args as Args;
}

function printUsage(): void {
  console.log(`
GCP Secret Manager Upload Tool

Usage:
  bun src/bin/upload-secrets.ts [options]

Required Options:
  --project <ID>           GCP project ID
  --app-id <ID>            GitHub App ID
  --installation-id <ID>   GitHub App Installation ID
  --private-key-file <PATH> Path to private key .pem file

Optional:
  --prefix <PREFIX>        Secret name prefix (default: lornu)
  --help, -h               Show this help message

Environment Variables:
  GCP_PROJECT_ID                 GCP project ID
  GITHUB_APP_ID                  GitHub App ID
  GITHUB_APP_INSTALLATION_ID     Installation ID
  GITHUB_APP_PRIVATE_KEY_PATH    Path to .pem file

Example:
  bun src/bin/upload-secrets.ts \\
    --project my-gcp-project \\
    --app-id 123456 \\
    --installation-id 78901234 \\
    --private-key-file ./key.pem

Secrets Created:
  - {prefix}/github-app-id
  - {prefix}/github-app-installation-id
  - {prefix}/github-app-private-key
`);
}

async function createOrUpdateSecret(
  client: SecretManagerServiceClient,
  project: string,
  secretId: string,
  value: string
): Promise<void> {
  const parent = `projects/${project}`;
  const secretName = `${parent}/secrets/${secretId}`;

  try {
    // Try to get the secret first
    await client.getSecret({ name: secretName });
    console.log(`  📝 Secret ${secretId} exists, adding new version...`);
  } catch {
    // Secret doesn't exist, create it
    console.log(`  🆕 Creating secret ${secretId}...`);
    await client.createSecret({
      parent,
      secretId,
      secret: {
        replication: {
          automatic: {},
        },
        labels: {
          "managed-by": "github-bot-tools",
          "lornu-ai-environment": "production",
        },
      },
    });
  }

  // Add the secret version
  await client.addSecretVersion({
    parent: secretName,
    payload: {
      data: Buffer.from(value, "utf8"),
    },
  });

  console.log(`  ✅ ${secretId} uploaded successfully`);
}

async function main() {
  const args = parseArgs();
  const client = new SecretManagerServiceClient();

  console.log("🔐 GCP Secret Manager Upload Tool\n");
  console.log("Configuration:");
  console.log(`  Project:         ${args.project}`);
  console.log(`  App ID:          ${args.appId}`);
  console.log(`  Installation ID: ${args.installationId}`);
  console.log(`  Private Key:     ${args.privateKeyFile}`);
  console.log(`  Secret Prefix:   ${args.prefix}`);

  console.log("\n📤 Uploading secrets...\n");

  try {
    // Upload App ID
    await createOrUpdateSecret(
      client,
      args.project,
      `${args.prefix}-github-app-id`,
      args.appId
    );

    // Upload Installation ID
    await createOrUpdateSecret(
      client,
      args.project,
      `${args.prefix}-github-app-installation-id`,
      args.installationId
    );

    // Upload Private Key
    const privateKey = readFileSync(args.privateKeyFile, "utf-8");
    await createOrUpdateSecret(
      client,
      args.project,
      `${args.prefix}-github-app-private-key`,
      privateKey
    );

    console.log("\n✅ All secrets uploaded successfully!");
    console.log("\n📋 Secret names created:");
    console.log(`  - ${args.prefix}-github-app-id`);
    console.log(`  - ${args.prefix}-github-app-installation-id`);
    console.log(`  - ${args.prefix}-github-app-private-key`);

    console.log("\n💡 To retrieve secrets in your application:");
    console.log(`  gcloud secrets versions access latest --secret="${args.prefix}-github-app-private-key"`);
  } catch (error) {
    console.error("\n❌ Error uploading secrets:", error instanceof Error ? error.message : error);
    process.exit(1);
  }
}

main();
