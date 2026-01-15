/**
 * CDK8s GitHub Team Construct
 *
 * Defines GitHub teams as code using ConfigMaps that the Rust engine processes.
 * The engine reads these ConfigMaps and uses the octocrab library to sync teams.
 *
 * Pattern: Declarative GitOps for GitHub Teams
 * - Teams defined in TypeScript (no YAML checked in)
 * - ConfigMaps synthesized by CDK8s
 * - Rust engine watches ConfigMaps and syncs to GitHub
 *
 * Usage:
 *   new GitHubTeamConstruct(chart, 'engineering', {
 *     org: 'lornu-ai',
 *     slug: 'engineering',
 *     description: 'Engineering team',
 *     privacy: 'closed',
 *     members: [
 *       { username: 'stevenirvin', role: 'maintainer' },
 *       { username: 'contributor1', role: 'member' },
 *     ],
 *   });
 */

import { Construct } from "constructs";
import { ApiObject } from "cdk8s";

export interface TeamMember {
  /** GitHub username */
  username: string;
  /** Role in the team: member or maintainer */
  role: "member" | "maintainer";
}

export interface GitHubTeamProps {
  /** GitHub organization name */
  org: string;
  /** Team slug (URL-friendly name) */
  slug: string;
  /** Human-readable team description */
  description?: string;
  /** Team privacy: secret (only visible to org members) or closed (visible to all) */
  privacy?: "secret" | "closed";
  /** Team members */
  members: TeamMember[];
  /** Parent team slug (for nested teams) */
  parentTeamSlug?: string;
  /** Namespace for the ConfigMap (default: lornu-ai) */
  namespace?: string;
}

/**
 * GitHubTeamConstruct - Declarative GitHub team management via CDK8s
 *
 * Creates a ConfigMap that the Rust engine watches and syncs to GitHub.
 * The engine uses the GITHUB_TEAM_PAT secret (synced from GSM) for authentication.
 */
export class GitHubTeamConstruct extends Construct {
  public readonly configMapName: string;

  constructor(scope: Construct, id: string, props: GitHubTeamProps) {
    super(scope, id);

    const namespace = props.namespace ?? "lornu-ai";
    this.configMapName = `github-team-${props.slug}`;

    // Create ConfigMap that the Rust engine watches
    new ApiObject(this, "ConfigMap", {
      apiVersion: "v1",
      kind: "ConfigMap",
      metadata: {
        name: this.configMapName,
        namespace,
        labels: {
          "lornu.ai/managed-by": "cdk8s",
          "lornu.ai/resource-type": "github-team",
          "lornu.ai/team-slug": props.slug,
        },
        annotations: {
          "lornu.ai/github-org": props.org,
          "lornu.ai/sync-enabled": "true",
        },
      },
      data: {
        // Team configuration as JSON
        "team.json": JSON.stringify(
          {
            org: props.org,
            slug: props.slug,
            description: props.description ?? "",
            privacy: props.privacy ?? "closed",
            parentTeamSlug: props.parentTeamSlug,
            members: props.members,
          },
          null,
          2
        ),
      },
    });
  }
}

/**
 * GitHubTeamsChart - Chart containing all GitHub team definitions
 *
 * Usage in infra/main.ts:
 *   const teamsChart = new GitHubTeamsChart(app, 'github-teams', {
 *     org: 'lornu-ai',
 *     teams: [...],
 *   });
 */
export interface GitHubTeamsChartProps {
  /** GitHub organization */
  org: string;
  /** Namespace for ConfigMaps */
  namespace?: string;
}

/**
 * Helper to create multiple teams in bulk
 */
export function createTeams(
  scope: Construct,
  org: string,
  teams: Array<Omit<GitHubTeamProps, "org">>
): GitHubTeamConstruct[] {
  return teams.map(
    (team) =>
      new GitHubTeamConstruct(scope, team.slug, {
        ...team,
        org,
      })
  );
}

// Example usage (can be imported into infra/main.ts):
//
// import { GitHubTeamConstruct, createTeams } from './constructs/github-team';
//
// // In your LornuInfra chart:
// createTeams(this, 'lornu-ai', [
//   {
//     slug: 'engineering',
//     description: 'Core engineering team',
//     privacy: 'closed',
//     members: [
//       { username: 'stevenirvin', role: 'maintainer' },
//     ],
//   },
//   {
//     slug: 'platform',
//     description: 'Platform and infrastructure team',
//     parentTeamSlug: 'engineering',
//     members: [
//       { username: 'stevenirvin', role: 'maintainer' },
//     ],
//   },
// ]);
