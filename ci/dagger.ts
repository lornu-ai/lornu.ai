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

import { dag, Container, Directory } from "@dagger.io/dagger";
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

async function runFeatPipeline(source: Directory): Promise<void> {
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("🔨 Feature Branch Pipeline: Build + Check");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

  // Create a base container with both Rust and Bun for flexibility
  // Use alpine-based Bun image and add Rust
  const baseContainer = dag
    .container()
    .from("oven/bun:alpine")
    .withExec(["apk", "add", "--no-cache", "rust", "cargo", "bash"]);

  // Install dependencies first
  console.log("📦 Installing dependencies...");
  const install = baseContainer
    .withMountedDirectory("/src", source)
    .withWorkdir("/src")
    .withExec(["bun", "install"]);

  try {
    await install.stdout();
    console.log("✅ Dependencies installed");
  } catch (error) {
    console.log("⚠️  Dependency installation had issues, but continuing...");
  }

  // Run baseline checks using Justfile
  // The Justfile already handles conditional checks (Rust/Bun/contracts)
  console.log("🔍 Running baseline checks (just check)...");
  const checks = baseContainer
    .withMountedDirectory("/src", source)
    .withWorkdir("/src")
    .withExec(["apk", "add", "--no-cache", "just"])
    .withExec(["just", "check"]);

  try {
    const checkOutput = await checks.stdout();
    console.log(checkOutput);
    console.log("✅ Baseline checks completed");
  } catch (error) {
    console.error("❌ Baseline checks failed!");
    console.error("   Error:", error instanceof Error ? error.message : String(error));
    throw error; // Fail the pipeline if checks fail
  }

  console.log("✅ Feature branch pipeline completed!");
}

async function runInfraPipeline(source: Directory): Promise<void> {
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("🏗️  Infrastructure Branch Pipeline: Dry-Run Validation");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

  // Run infrastructure dry-run
  console.log("📋 Running infrastructure dry-run...");
  const infraPlan = dag
    .container()
    .from("oven/bun:latest")
    .withMountedDirectory("/src", source)
    .withWorkdir("/src")
    .withExec(["bun", "install"])
    .withExec(["bun", "run", "infra:plan"]);

  const planOutput = await infraPlan.stdout();
  console.log(planOutput);

  console.log("✅ Infrastructure plan validated!");
}

async function runAgentPipeline(source: Directory, branch: string): Promise<void> {
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("🤖 Agent Branch Pipeline: Sandbox Environment");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

  // Extract agent name from branch (e.g., agent/researcher/exp-1 -> researcher)
  const agentName = branch.split("/")[1] || "unknown";
  const sandboxName = `agent-${agentName}-${Date.now()}`;

  console.log(`🏗️  Creating sandbox: ${sandboxName}`);

  // Create sandbox namespace via Crossplane (this would call your infra script)
  const sandboxCreate = dag
    .container()
    .from("oven/bun:latest")
    .withMountedDirectory("/src", source)
    .withWorkdir("/src")
    .withExec(["bun", "install"])
    .withExec([
      "bun",
      "run",
      "infra:agent:sandbox:create",
      "--name",
      sandboxName,
      "--branch",
      branch,
    ]);

  await sandboxCreate.stdout();

  // Build and deploy agent to sandbox
  console.log("📦 Building agent...");
  const agentBuild = dag
    .container()
    .from("rust:1.75-slim")
    .withMountedDirectory("/src", source)
    .withWorkdir("/src")
    .withExec(["cargo", "build", "--release", "--bin", agentName]);

  await agentBuild.stdout();

  // Deploy to Knative service in sandbox
  console.log("🚀 Deploying to Knative sandbox...");
  const deploy = dag
    .container()
    .from("oven/bun:latest")
    .withMountedDirectory("/src", source)
    .withWorkdir("/src")
    .withExec(["bun", "run", "infra:agent:deploy", "--name", agentName, "--sandbox", sandboxName]);

  await deploy.stdout();

  console.log(`✅ Agent deployed to sandbox: ${sandboxName}`);
  console.log(`💡 Sandbox will be automatically deleted when branch is deleted`);
}

async function runTrunkPipeline(source: Directory): Promise<void> {
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("🚀 Trunk (`ta`) Pipeline: Apply + Release Build");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

  // Apply infrastructure changes
  console.log("🏗️  Applying infrastructure changes...");
  const infraApply = dag
    .container()
    .from("oven/bun:latest")
    .withMountedDirectory("/src", source)
    .withWorkdir("/src")
    .withExec(["bun", "install"])
    .withExec(["bun", "run", "infra:apply"]);

  await infraApply.stdout();

  // Build release artifacts
  console.log("📦 Building release artifacts...");
  const rustRelease = dag
    .container()
    .from("rust:1.75-slim")
    .withMountedDirectory("/src", source)
    .withWorkdir("/src")
    .withExec(["cargo", "build", "--release"]);

  await rustRelease.stdout();

  const bunRelease = dag
    .container()
    .from("oven/bun:latest")
    .withMountedDirectory("/src", source)
    .withWorkdir("/src")
    .withExec(["bun", "install"])
    .withExec(["bun", "run", "build"]);

  await bunRelease.stdout();

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

  // Connect to Dagger (disable telemetry to avoid OpenTelemetry dependency issues)
  const client = await dag.connect({
    logOutput: process.stdout,
  });

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
        await runFeatPipeline(source);
        break;

      case "infra":
        await runInfraPipeline(source);
        break;

      case "agent":
        await runAgentPipeline(source, branch);
        break;

      case "trunk":
        if (event === "merge") {
          await runTrunkPipeline(source);
        } else {
          console.log("ℹ️  Trunk branch push - running feature pipeline");
          await runFeatPipeline(source);
        }
        break;

      default:
        console.log(`⚠️  Unknown branch pattern: ${pattern}`);
        console.log("   Running feature pipeline as fallback");
        await runFeatPipeline(source);
    }
  } catch (error) {
    console.error("❌ Pipeline failed:", error);
    process.exit(1);
  } finally {
    await client.close();
  }
}

main().catch((error) => {
  console.error("❌ Fatal error:", error);
  process.exit(1);
});
