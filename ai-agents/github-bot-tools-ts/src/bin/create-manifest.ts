#!/usr/bin/env bun
/**
 * GitHub App Manifest Link Generator
 *
 * Generates a URL that, when clicked, creates a GitHub App with the exact
 * permissions needed for Lornu AI bot operations. This uses GitHub's
 * Manifest flow for "one-click" app creation.
 *
 * Usage:
 *   bun src/bin/create-manifest.ts [--org <ORG>] [--name <APP_NAME>]
 *
 * Environment Variables:
 *   GITHUB_ORG - Organization name (default: lornu-ai)
 *   GITHUB_APP_NAME - App name (default: lornu-ai-bot)
 */

interface ManifestConfig {
  org: string;
  name: string;
  url: string;
  description: string;
  public: boolean;
  webhookActive: boolean;
}

interface Args {
  org: string;
  name: string;
  url?: string;
  description?: string;
  public?: boolean;
}

function parseArgs(): Args {
  const args: Partial<Args> = {};

  for (let i = 2; i < process.argv.length; i++) {
    const arg = process.argv[i];
    if (arg === "--org" && i + 1 < process.argv.length) {
      args.org = process.argv[++i];
    } else if (arg === "--name" && i + 1 < process.argv.length) {
      args.name = process.argv[++i];
    } else if (arg === "--url" && i + 1 < process.argv.length) {
      args.url = process.argv[++i];
    } else if (arg === "--description" && i + 1 < process.argv.length) {
      args.description = process.argv[++i];
    } else if (arg === "--public") {
      args.public = true;
    } else if (arg === "--help" || arg === "-h") {
      printUsage();
      process.exit(0);
    }
  }

  // Support environment variables
  args.org = args.org || process.env.GITHUB_ORG || "lornu-ai";
  args.name = args.name || process.env.GITHUB_APP_NAME || "lornu-ai-bot";

  return args as Args;
}

function printUsage(): void {
  console.log(`
GitHub App Manifest Link Generator

Usage:
  bun src/bin/create-manifest.ts [options]

Options:
  --org <ORG>           GitHub organization name (default: lornu-ai)
  --name <NAME>         App name (default: lornu-ai-bot)
  --url <URL>           App homepage URL (default: https://lornu.ai)
  --description <DESC>  App description
  --public              Make the app public (default: private)
  --help, -h            Show this help message

Environment Variables:
  GITHUB_ORG            Organization name
  GITHUB_APP_NAME       App name

Example:
  bun src/bin/create-manifest.ts --org lornu-ai --name my-bot
`);
}

function generateManifest(config: ManifestConfig): object {
  return {
    name: config.name,
    url: config.url,
    description: config.description,
    hook_attributes: {
      active: config.webhookActive,
    },
    public: config.public,
    // Permissions required for PR review bot
    default_permissions: {
      pull_requests: "write",
      contents: "read",
      metadata: "read",
      issues: "write",
      checks: "write",
      statuses: "write",
    },
    // Events to subscribe to
    default_events: [
      "pull_request",
      "pull_request_review",
      "pull_request_review_comment",
      "check_run",
      "check_suite",
    ],
  };
}

function main() {
  const args = parseArgs();

  const config: ManifestConfig = {
    org: args.org,
    name: args.name,
    url: args.url || "https://lornu.ai",
    description: args.description || "Lornu AI Bot - Automated PR approval and CI operations",
    public: args.public || false,
    webhookActive: false, // We don't need webhooks for token-based operations
  };

  const manifest = generateManifest(config);
  const encodedManifest = encodeURIComponent(JSON.stringify(manifest));
  const registrationUrl = `https://github.com/organizations/${config.org}/settings/apps/new?manifest=${encodedManifest}`;

  console.log("🤖 GitHub App Manifest Generator\n");
  console.log("Configuration:");
  console.log(`  Organization: ${config.org}`);
  console.log(`  App Name:     ${config.name}`);
  console.log(`  Homepage:     ${config.url}`);
  console.log(`  Public:       ${config.public}`);
  console.log(`  Webhooks:     ${config.webhookActive ? "enabled" : "disabled"}`);

  console.log("\n📋 Permissions requested:");
  console.log("  - pull_requests: write (approve/review PRs)");
  console.log("  - contents: read (read repository files)");
  console.log("  - metadata: read (read repository metadata)");
  console.log("  - issues: write (comment on issues)");
  console.log("  - checks: write (create check runs)");
  console.log("  - statuses: write (set commit statuses)");

  console.log("\n🔗 Registration URL:\n");
  console.log(registrationUrl);

  console.log("\n\n📝 Next Steps:");
  console.log("  1. Click the URL above (or copy to browser)");
  console.log("  2. Review and confirm the app name");
  console.log("  3. Click 'Create GitHub App'");
  console.log("  4. Download the private key (.pem file)");
  console.log("  5. Note the App ID and Installation ID");
  console.log("  6. Use upload-secrets.ts to store credentials in GCP Secret Manager");
}

main();
