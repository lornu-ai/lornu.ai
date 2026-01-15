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
    "create-teams": {
      type: "boolean",
      description: "Create GitHub teams if they don't exist (default: false)",
      default: false,
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
  --create-teams         Create GitHub teams if they don't exist
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
  console.error("   Required scopes: repo, admin:repo (and admin:org for --create-teams)");
  process.exit(1);
}

async function checkTokenPermissions() {
  console.log("🔍 Checking token permissions...\n");
  
  try {
    // Check if we can read the repo (basic repo scope)
    await octokit.repos.get({
      owner,
      repo,
    });
    console.log("✅ Repository read access: OK");

    // Check if we can update repo settings (admin:repo scope)
    try {
      const { data: repoData } = await octokit.repos.get({
        owner,
        repo,
      });
      // Try a harmless read to verify we have access
      console.log("✅ Repository access: OK");
    } catch (error: any) {
      if (error.status === 403) {
        console.error("❌ Repository access denied");
        throw error;
      }
    }

    // Check admin:repo by trying to get branch protection (read-only check)
    try {
      await octokit.repos.getBranchProtection({
        owner,
        repo,
        branch,
      });
      console.log("✅ Branch protection read access: OK (admin:repo scope present)");
    } catch (error: any) {
      if (error.status === 403) {
        console.error("❌ Missing 'admin:repo' scope");
        console.error("\n💡 To fix this:");
        console.error("   1. Go to: https://github.com/settings/tokens");
        console.error("   2. Create a new token (or edit existing)");
        console.error("   3. Select these scopes:");
        console.error("      - ✅ repo (full control)");
        console.error("      - ✅ admin:repo (for branch protection)");
        if (createTeams) {
          console.error("      - ✅ admin:org (for team creation)");
        }
        console.error("   4. Generate token and use it as GITHUB_TOKEN");
        throw new Error("Insufficient permissions: admin:repo scope required");
      } else if (error.status === 404) {
        // Branch protection doesn't exist yet, which is fine
        console.log("ℹ️  Branch protection not set yet (will be created)");
      }
    }

    if (createTeams) {
      // Check admin:org by trying to list teams
      try {
        await octokit.teams.list({
          org: owner,
          per_page: 1,
        });
        console.log("✅ Organization access: OK (admin:org scope present)");
      } catch (error: any) {
        if (error.status === 403) {
          console.error("❌ Missing 'admin:org' scope (required for --create-teams)");
          console.error("\n💡 To fix this:");
          console.error("   1. Go to: https://github.com/settings/tokens");
          console.error("   2. Edit your token");
          console.error("   3. Add 'admin:org' scope");
          console.error("   4. Or run without --create-teams and create teams manually");
          throw new Error("Insufficient permissions: admin:org scope required for team creation");
        }
      }
    }

    console.log("");
  } catch (error: any) {
    if (error.message.includes("Insufficient permissions")) {
      process.exit(1);
    }
    // If it's a different error, let it bubble up
    throw error;
  }
}

const [owner, repo] = args.values.repo!.split("/");
const branch = args.values.branch!;
const dryRun = args.values["dry-run"];
const createTeams = args.values["create-teams"];

const octokit = new Octokit({
  auth: GITHUB_TOKEN,
});

// Teams required by CODEOWNERS
const REQUIRED_TEAMS = [
  { name: "engine-team", description: "Rust engine development team" },
  { name: "ui-team", description: "Bun/Next.js web application team" },
  { name: "infra-ops", description: "Infrastructure and CI/CD operations team" },
];

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

async function createOrVerifyTeams() {
  console.log(`\n👥 Checking GitHub teams...`);

  for (const team of REQUIRED_TEAMS) {
    try {
      // Check if team exists
      const { data: existingTeam } = await octokit.teams.getByName({
        org: owner,
        team_slug: team.name,
      });

      console.log(`✅ Team '@${team.name}' already exists`);
    } catch (error: any) {
      if (error.status === 404) {
        // Team doesn't exist
        if (dryRun) {
          console.log(`📋 Would create team '@${team.name}'`);
        } else if (createTeams) {
          try {
            const { data: newTeam } = await octokit.teams.create({
              org: owner,
              name: team.name,
              description: team.description,
              privacy: "closed", // Closed teams are visible to organization members
            });
            console.log(`✅ Created team '@${team.name}' (ID: ${newTeam.id})`);
            console.log(`   💡 Add members at: https://github.com/orgs/${owner}/teams/${team.name}/members`);
          } catch (createError: any) {
            if (createError.status === 403) {
              console.error(`❌ Error: Insufficient permissions to create team '@${team.name}'`);
              console.error("   GITHUB_TOKEN needs 'admin:org' scope for team creation");
            } else {
              throw createError;
            }
          }
        } else {
          console.log(`⚠️  Team '@${team.name}' does not exist`);
          console.log(`   Run with --create-teams to create it, or create manually in GitHub`);
        }
      } else {
        throw error;
      }
    }
  }

  if (!createTeams && !dryRun) {
    console.log(`\n💡 Tip: Use --create-teams to automatically create missing teams`);
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
    // 0. Check token permissions first
    if (!dryRun) {
      await checkTokenPermissions();
    } else {
      console.log("🔍 Skipping permission check (dry-run mode)\n");
    }

    // 1. Create or verify teams (if requested)
    if (createTeams || dryRun) {
      await createOrVerifyTeams();
    }

    // 2. Configure merge settings first (easier to change)
    await configureMergeSettings();

    // 3. Set default branch
    await setDefaultBranch();

    // 4. Configure branch protection (most restrictive, do last)
    await setupBranchProtection();

    console.log("\n✅ All configuration complete!");
    if (dryRun) {
      console.log("\n💡 Run without --dry-run to apply these changes");
    } else {
      console.log("\n💡 Next steps:");
      if (!createTeams) {
        console.log(`   1. Create GitHub teams if they don't exist:`);
        console.log(`      - Run with --create-teams flag, or`);
        console.log(`      - Create manually: https://github.com/orgs/${owner}/teams`);
      }
      console.log(`   ${createTeams ? "1" : "2"}. Verify branch protection: https://github.com/${owner}/${repo}/settings/branches`);
      console.log(`   ${createTeams ? "2" : "3"}. Add team members to teams (if created):`);
      REQUIRED_TEAMS.forEach(team => {
        console.log(`      - @${team.name}: https://github.com/orgs/${owner}/teams/${team.name}/members`);
      });
      console.log(`   ${createTeams ? "3" : "4"}. Test by creating a PR targeting '${branch}'`);
      console.log(`   ${createTeams ? "4" : "5"}. Verify Squash & Merge is the only option available`);
      console.log(`   ${createTeams ? "5" : "6"}. Verify CODEOWNERS automatically requests reviewers`);
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
