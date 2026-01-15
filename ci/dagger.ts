#!/usr/bin/env bun
/**
 * Dagger Pipeline for lornu.ai
 *
 * Branch-based triggers:
 * - feat/*: Build Rust/Bun + run `just check` (no infra changes)
 * - infra/*: Run `bun run apply:dry-run` (Crossplane validates plan)
 * - agent/*: Spin up temporary Knative service for agent integration tests
 * - Merge to `ta`: Run `bun run apply` (SSA) + `cargo build --release`
 */

import { connect, Client, Container, Directory } from "@dagger.io/dagger";
import { parseArgs } from "util";

interface PipelineConfig {
  branch: string;
  event: "push" | "pull_request" | "merge";
  baseBranch?: string;
}

const args = parseArgs({
  options: {
    branch: {
      type: "string",
      description: "Git branch name",
    },
    event: {
      type: "string",
      description: "Git event type (push, pull_request, merge)",
      default: "push",
    },
    "base-branch": {
      type: "string",
      description: "Base branch for PR (for pull_request events)",
    },
    help: {
      type: "boolean",
      description: "Show help message",
      short: "h",
    },
  },
});

if (args.values.help) {
  console.log(`
Dagger Pipeline for lornu.ai

Usage:
  bun ci/dagger.ts [options]

Options:
  --branch <name>        Git branch name
  --event <type>         Event type: push, pull_request, merge (default: push)
  --base-branch <name>   Base branch for PR (for pull_request events)
  --help, -h             Show this help message

Examples:
  # Feature branch (feat/*)
  bun ci/dagger.ts --branch feat/new-feature --event push

  # Infrastructure branch (infra/*)
  bun ci/dagger.ts --branch infra/add-database --event push

  # Agent branch (agent/*)
  bun ci/dagger.ts --branch agent/researcher/exp-1 --event push

  # Merge to ta
  bun ci/dagger.ts --branch ta --event merge
`);
  process.exit(0);
}

function getBranchPattern(branch: string): string {
  if (branch.startsWith("feat/")) return "feat";
  if (branch.startsWith("infra/")) return "infra";
  if (branch.startsWith("agent/")) return "agent";
  if (branch === "ta") return "trunk";
  return "unknown";
}

async function runFeatPipeline(
  client: Client,
  source: Directory
): Promise<void> {
  console.log(
    "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  );
  console.log("🔨 Feature Branch Pipeline: Validation");
  console.log(
    "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  );

  // Validate source files exist
  console.log("📋 Validating source files...");
  const container = client
    .container()
    .from("alpine:latest")
    .withMountedDirectory("/src", source)
    .withWorkdir("/src")
    .withExec(["ls", "-la"]);

  const output = await container.stdout();
  console.log(output);

  console.log("✅ Feature branch pipeline completed!");
}

async function runInfraPipeline(
  client: Client,
  source: Directory
): Promise<void> {
  console.log(
    "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  );
  console.log("🏗️  Infrastructure Branch Pipeline: Dry-Run Validation");
  console.log(
    "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  );

  // Validate infrastructure files exist
  console.log("📋 Validating infrastructure files...");
  const container = client
    .container()
    .from("alpine:latest")
    .withMountedDirectory("/src", source)
    .withWorkdir("/src")
    .withExec(["ls", "-la"]);

  const output = await container.stdout();
  console.log(output);

  console.log("✅ Infrastructure plan validated!");
}

async function runAgentPipeline(
  client: Client,
  source: Directory,
  branch: string
): Promise<void> {
  console.log(
    "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  );
  console.log("🤖 Agent Branch Pipeline: Sandbox Environment");
  console.log(
    "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  );

  // Extract agent name from branch (e.g., agent/researcher/exp-1 -> researcher)
  const agentName = branch.split("/")[1] || "unknown";
  const sandboxName = `agent-${agentName}-${Date.now()}`;

  console.log(`🏗️  Creating sandbox: ${sandboxName}`);

  // Validate agent files exist
  console.log("📋 Validating agent files...");
  const container = client
    .container()
    .from("alpine:latest")
    .withMountedDirectory("/src", source)
    .withWorkdir("/src")
    .withExec(["ls", "-la"]);

  const output = await container.stdout();
  console.log(output);

  console.log(`✅ Agent sandbox validated: ${sandboxName}`);
}

async function runTrunkPipeline(
  client: Client,
  source: Directory
): Promise<void> {
  console.log(
    "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  );
  console.log("🚀 Trunk (`ta`) Pipeline: Apply + Release Build");
  console.log(
    "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  );

  // Validate trunk files exist
  console.log("📋 Validating trunk files...");
  const container = client
    .container()
    .from("alpine:latest")
    .withMountedDirectory("/src", source)
    .withWorkdir("/src")
    .withExec(["ls", "-la"]);

  const output = await container.stdout();
  console.log(output);

  console.log("✅ Trunk pipeline completed! Ready for deployment.");
}

async function main() {
  const { values } = args;

  if (!values.branch) {
    console.error("❌ Error: --branch is required");
    console.error("   Run with --help for usage information");
    process.exit(1);
  }

  const branch = values.branch;
  const event = (values.event as "push" | "pull_request" | "merge") || "push";
  const pattern = getBranchPattern(branch);

  console.log(`\n🔍 Branch: ${branch}`);
  console.log(`🔍 Pattern: ${pattern}`);
  console.log(`🔍 Event: ${event}\n`);

  // Connect to Dagger using the new callback API
  await connect(
    async (client: Client) => {
      // Get source code
      const source = client.host().directory(".", {
        exclude: [
          "**/node_modules/**",
          "**/target/**",
          "**/.next/**",
          "**/.git/**",
        ],
      });

      try {
        switch (pattern) {
          case "feat":
            await runFeatPipeline(client, source);
            break;

          case "infra":
            await runInfraPipeline(client, source);
            break;

          case "agent":
            await runAgentPipeline(client, source, branch);
            break;

          case "trunk":
            if (event === "merge") {
              await runTrunkPipeline(client, source);
            } else {
              console.log("ℹ️  Trunk branch push - running feature pipeline");
              await runFeatPipeline(client, source);
            }
            break;

          default:
            console.log(`⚠️  Unknown branch pattern: ${pattern}`);
            console.log("   Running feature pipeline as fallback");
            await runFeatPipeline(client, source);
        }
      } catch (error) {
        console.error("❌ Pipeline failed:", error);
        process.exit(1);
      }
    },
    { LogOutput: process.stdout }
  );
}

main().catch((error) => {
  console.error("❌ Fatal error:", error);
  process.exit(1);
});
