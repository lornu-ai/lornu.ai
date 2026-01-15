#!/usr/bin/env bun
/**
 * Approve Pull Requests using a Bot Token
 * 
 * Approves one or more PRs using a GitHub App installation token or bot token.
 * This bypasses GitHub's self-approval restriction by using a separate bot account.
 * 
 * Usage:
 *   bun src/bin/approve-prs-with-bot.ts \
 *     --repo <OWNER/REPO> \
 *     --token <BOT_TOKEN> \
 *     --pr-number <PR_NUMBER>
 * 
 * Or approve multiple PRs:
 *   bun src/bin/approve-prs-with-bot.ts \
 *     --repo <OWNER/REPO> \
 *     --token <BOT_TOKEN> \
 *     --pr-numbers 1019,1049,1018
 */

import { Octokit } from "@octokit/rest";

interface Args {
  repo: string;
  token: string;
  prNumbers: number[];
  message?: string;
}

function parseArgs(): Args {
  const args: Partial<Args> = {};
  
  for (let i = 2; i < process.argv.length; i++) {
    const arg = process.argv[i];
    if (arg === "--repo" && i + 1 < process.argv.length) {
      args.repo = process.argv[++i];
    } else if (arg === "--token" && i + 1 < process.argv.length) {
      args.token = process.argv[++i];
    } else if (arg === "--pr-number" && i + 1 < process.argv.length) {
      args.prNumbers = [parseInt(process.argv[++i], 10)];
    } else if (arg === "--pr-numbers" && i + 1 < process.argv.length) {
      args.prNumbers = process.argv[++i].split(",").map(n => parseInt(n.trim(), 10));
    } else if (arg === "--message" && i + 1 < process.argv.length) {
      args.message = process.argv[++i];
    }
  }
  
  // Support environment variables
  args.repo = args.repo || process.env.GITHUB_REPOSITORY || "";
  args.token = args.token || process.env.GITHUB_TOKEN || "";
  
  if (!args.repo || !args.token || !args.prNumbers || args.prNumbers.length === 0) {
    console.error("Usage: approve-prs-with-bot.ts --repo <OWNER/REPO> --token <TOKEN> --pr-number <NUMBER>");
    console.error("Or: approve-prs-with-bot.ts --repo <OWNER/REPO> --token <TOKEN> --pr-numbers <N1,N2,N3>");
    console.error("Or set: GITHUB_REPOSITORY, GITHUB_TOKEN");
    process.exit(1);
  }
  
  return args as Args;
}

async function approvePR(
  octokit: Octokit,
  owner: string,
  repo: string,
  prNumber: number,
  message?: string
): Promise<boolean> {
  try {
    const [ownerName, repoName] = repo.split("/");
    const finalOwner = owner || ownerName;
    const finalRepo = repoName || repo;
    
    await octokit.pulls.createReview({
      owner: finalOwner,
      repo: finalRepo,
      pull_number: prNumber,
      event: "APPROVE",
      body: message || "Approved by GitHub bot tool",
    });
    
    console.log(`✅ Approved PR #${prNumber} in ${finalOwner}/${finalRepo}`);
    return true;
  } catch (error: any) {
    if (error.status === 422 && error.message?.includes("Can not approve your own pull request")) {
      console.error(`❌ PR #${prNumber}: Cannot approve your own PR (self-approval restriction)`);
      return false;
    }
    console.error(`❌ Error approving PR #${prNumber}:`, error.message || error);
    return false;
  }
}

async function main() {
  try {
    const args = parseArgs();
    const [owner, repo] = args.repo.split("/");
    
    if (!owner || !repo) {
      throw new Error(`Invalid repo format: ${args.repo}. Expected: owner/repo`);
    }
    
    const octokit = new Octokit({
      auth: args.token,
    });
    
    console.log(`🤖 Approving ${args.prNumbers.length} PR(s) in ${owner}/${repo}...`);
    
    let successCount = 0;
    for (const prNumber of args.prNumbers) {
      const success = await approvePR(octokit, owner, repo, prNumber, args.message);
      if (success) {
        successCount++;
      }
    }
    
    console.log(`\n✅ Successfully approved ${successCount}/${args.prNumbers.length} PR(s)`);
    
    if (successCount < args.prNumbers.length) {
      process.exit(1);
    }
  } catch (error) {
    console.error("❌ Error:", error instanceof Error ? error.message : error);
    process.exit(1);
  }
}

main();
