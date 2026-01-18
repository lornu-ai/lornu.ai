import { Octokit } from "@octokit/rest";

const octokit = new Octokit({
  auth: process.env.GITHUB_TOKEN,
});

async function rebasePRs() {
  const owner = process.env.GITHUB_OWNER;
  const repo = process.env.GITHUB_REPO;

  if (!owner || !repo) {
    throw new Error("GITHUB_OWNER and GITHUB_REPO must be set");
  }

  const { data: prs } = await octokit.pulls.list({
    owner,
    repo,
    state: "open",
  });

  for (const pr of prs) {
    try {
      await octokit.pulls.updateBranch({
        owner,
        repo,
        pull_number: pr.number,
      });
      console.log(`Successfully rebased PR #${pr.number}`);
    } catch (error) {
      console.error(`Failed to rebase PR #${pr.number}:`, error);
    }
  }
}

rebasePRs().catch(console.error);
