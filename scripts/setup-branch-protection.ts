#!/usr/bin/env bun
/**
 * Setup Branch Protection and Repository Settings for lornu.ai
 * 
 * Configures:
 * - Branch protection rules for `ta` branch
 * - Default branch to `ta`
 * - Repository merge settings (Squash & Merge only)
 * 
 * Usage:
 *   GITHUB_TOKEN=your_token bun scripts/setup-branch-protection.ts
 *   GITHUB_TOKEN=your_token bun scripts/setup-branch-protection.ts --dry-run
 */

import { Octokit } from "@octokit/rest";
import { parseArgs } from "util";

const args = parseArgs({
  options: {
    "dry-run": {
      type: "boolean",
      description: "Show what would be done without making changes",
      default: false,
    },
    "repo": {
      type: "string",
      description: "Repository in format 'owner/repo' (default: lornu-ai/lornu.ai)",
      default: "lornu-ai/lornu.ai",
    },
    "branch": {
      type: "string",
      description: "Branch to protect (default: ta)",
      default: "ta",
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
Setup Branch Protection and Repository Settings

Usage:
  GITHUB_TOKEN=your_token bun scripts/setup-branch-protection.ts [options]

Options:
  --dry-run              Show what would be done without making changes
  --repo <owner/repo>    Repository (default: lornu-ai/lornu.ai)
  --branch <name>        Branch to protect (default: ta)
  --help, -h             Show this help message

Environment Variables:
  GITHUB_TOKEN           GitHub Personal Access Token (required)
                         Needs: repo, admin:repo (for branch protection)

Examples:
  # Dry run to see what would be configured
  GITHUB_TOKEN=token bun scripts/setup-branch-protection.ts --dry-run

  # Configure branch protection for ta branch
  GITHUB_TOKEN=token bun scripts/setup-branch-protection.ts

  # Configure for different repository
  GITHUB_TOKEN=token bun scripts/setup-branch-protection.ts --repo owner/repo
`);
  process.exit(0);
}

const GITHUB_TOKEN = process.env.GITHUB_TOKEN;
if (!GITHUB_TOKEN) {
  console.error("❌ Error: GITHUB_TOKEN environment variable is required");
  console.error("   Get a token from: https://github.com/settings/tokens");
  console.error("   Required scopes: repo, admin:repo");
  process.exit(1);
}

const [owner, repo] = args.values.repo!.split("/");
const branch = args.values.branch!;
const dryRun = args.values["dry-run"];

const octokit = new Octokit({
  auth: GITHUB_TOKEN,
});

interface BranchProtectionConfig {
  required_status_checks: {
    strict: boolean;
    contexts: string[];
  };
  enforce_admins: boolean;
  required_pull_request_reviews: {
    required_approving_review_count: number;
    dismiss_stale_reviews: boolean;
    require_code_owner_reviews: boolean;
    require_last_push_approval: boolean;
  };
  restrictions: null; // Allow all users to push (or set to specific teams/users)
  required_linear_history: boolean;
  allow_force_pushes: boolean;
  allow_deletions: boolean;
  required_conversation_resolution: boolean;
  lock_branch: boolean;
  allow_fork_syncing: boolean;
}

async function setupBranchProtection() {
  console.log(`\n🔐 Setting up branch protection for: ${owner}/${repo}`);
  console.log(`   Branch: ${branch}`);
  console.log(`   Mode: ${dryRun ? "DRY RUN" : "LIVE"}\n`);

  try {
    // Check if branch exists
    try {
      await octokit.repos.getBranch({
        owner,
        repo,
        branch,
      });
      console.log(`✅ Branch '${branch}' exists`);
    } catch (error: any) {
      if (error.status === 404) {
        console.error(`❌ Error: Branch '${branch}' does not exist`);
        console.error(`   Create it first: git checkout -b ${branch} && git push origin ${branch}`);
        process.exit(1);
      }
      throw error;
    }

    // Configure branch protection
    const protectionConfig: BranchProtectionConfig = {
      required_status_checks: {
        strict: true, // Require branches to be up to date
        contexts: [], // Will be populated by GitHub Actions
      },
      enforce_admins: true, // Apply rules to admins too
      required_pull_request_reviews: {
        required_approving_review_count: 1,
        dismiss_stale_reviews: true,
        require_code_owner_reviews: true, // Enforce CODEOWNERS
        require_last_push_approval: true, // Require approval after new commits
      },
      restrictions: null, // Allow all users (or specify teams/users)
      required_linear_history: true, // Enforces Squash & Merge
      allow_force_pushes: false,
      allow_deletions: false,
      required_conversation_resolution: true, // All comments must be resolved
      lock_branch: false, // Don't lock (allows PRs)
      allow_fork_syncing: true,
    };

    if (dryRun) {
      console.log("📋 Would configure branch protection:");
      console.log(JSON.stringify(protectionConfig, null, 2));
    } else {
      await octokit.repos.updateBranchProtection({
        owner,
        repo,
        branch,
        ...protectionConfig,
      });
      console.log(`✅ Branch protection configured for '${branch}'`);
    }
  } catch (error: any) {
    if (error.status === 404) {
      console.error(`❌ Error: Repository ${owner}/${repo} not found or no access`);
      console.error("   Check your GITHUB_TOKEN has the correct permissions");
      process.exit(1);
    } else if (error.status === 403) {
      console.error(`❌ Error: Insufficient permissions`);
      console.error("   GITHUB_TOKEN needs 'admin:repo' scope for branch protection");
      process.exit(1);
    }
    throw error;
  }
}

async function setDefaultBranch() {
  console.log(`\n📌 Setting default branch to: ${branch}`);

  try {
    if (dryRun) {
      console.log(`📋 Would set default branch to '${branch}'`);
    } else {
      await octokit.repos.update({
        owner,
        repo,
        default_branch: branch,
      });
      console.log(`✅ Default branch set to '${branch}'`);
    }
  } catch (error: any) {
    if (error.status === 403) {
      console.error(`❌ Error: Insufficient permissions to change default branch`);
      console.error("   GITHUB_TOKEN needs 'admin:repo' scope");
      process.exit(1);
    }
    throw error;
  }
}

async function configureMergeSettings() {
  console.log(`\n🔀 Configuring repository merge settings`);

  try {
    // Get current settings
    const { data: repoData } = await octokit.repos.get({
      owner,
      repo,
    });

    const mergeSettings = {
      allow_squash_merge: true,
      allow_merge_commit: false, // Disable regular merge
      allow_rebase_merge: false, // Disable rebase merge
      allow_auto_merge: true, // Allow auto-merge when requirements are met
      delete_branch_on_merge: true, // Auto-delete merged branches
    };

    if (dryRun) {
      console.log("📋 Would configure merge settings:");
      console.log(JSON.stringify(mergeSettings, null, 2));
      console.log("\n📋 Current settings:");
      console.log(`   Squash merge: ${repoData.allow_squash_merge ? "✅" : "❌"}`);
      console.log(`   Merge commit: ${repoData.allow_merge_commit ? "✅" : "❌"}`);
      console.log(`   Rebase merge: ${repoData.allow_rebase_merge ? "✅" : "❌"}`);
      console.log(`   Auto-merge: ${repoData.allow_auto_merge ? "✅" : "❌"}`);
      console.log(`   Delete on merge: ${repoData.delete_branch_on_merge ? "✅" : "❌"}`);
    } else {
      await octokit.repos.update({
        owner,
        repo,
        ...mergeSettings,
      });
      console.log(`✅ Merge settings configured:`);
      console.log(`   ✅ Squash & Merge: enabled`);
      console.log(`   ❌ Merge commit: disabled`);
      console.log(`   ❌ Rebase merge: disabled`);
      console.log(`   ✅ Auto-merge: enabled`);
      console.log(`   ✅ Delete branch on merge: enabled`);
    }
  } catch (error: any) {
    if (error.status === 403) {
      console.error(`❌ Error: Insufficient permissions to configure merge settings`);
      console.error("   GITHUB_TOKEN needs 'admin:repo' scope");
      process.exit(1);
    }
    throw error;
  }
}

async function main() {
  console.log("🚀 Setting up branch protection and repository settings");
  console.log(`   Repository: ${owner}/${repo}`);
  console.log(`   Branch: ${branch}\n`);

  try {
    // 1. Configure merge settings first (easier to change)
    await configureMergeSettings();

    // 2. Set default branch
    await setDefaultBranch();

    // 3. Configure branch protection (most restrictive, do last)
    await setupBranchProtection();

    console.log("\n✅ All configuration complete!");
    if (dryRun) {
      console.log("\n💡 Run without --dry-run to apply these changes");
    } else {
      console.log("\n💡 Next steps:");
      console.log(`   1. Verify branch protection: https://github.com/${owner}/${repo}/settings/branches`);
      console.log(`   2. Test by creating a PR targeting '${branch}'`);
      console.log(`   3. Verify Squash & Merge is the only option available`);
    }
  } catch (error: any) {
    console.error("\n❌ Error:", error.message);
    if (error.response) {
      console.error("   Status:", error.response.status);
      console.error("   Response:", JSON.stringify(error.response.data, null, 2));
    }
    process.exit(1);
  }
}

main();
