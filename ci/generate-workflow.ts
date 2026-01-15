#!/usr/bin/env bun
/**
 * Generate GitHub Actions Workflow YAML from TypeScript
 * 
 * This script generates .github/workflows/dagger-pipeline.yml
 * following the "no YAML" principle - all YAML is generated from code.
 */

import { writeFileSync, mkdirSync } from "fs";
import { dirname, join } from "path";

interface WorkflowStep {
  name: string;
  uses?: string;
  with?: Record<string, string>;
  run?: string;
  env?: Record<string, string>;
  if?: string;
  id?: string;
}

interface WorkflowJob {
  name?: string;
  "runs-on": string;
  steps: WorkflowStep[];
}

interface Workflow {
  name: string;
  on: {
    push?: {
      branches: string[];
    };
    pull_request?: {
      branches: string[];
    };
    workflow_dispatch?: {
      inputs?: Record<string, {
        description: string;
        required: boolean;
        default?: string;
      }>;
    };
  };
  jobs: Record<string, WorkflowJob>;
}

function generateWorkflow(): Workflow {
  return {
    name: "Dagger Pipeline",
    on: {
      push: {
        branches: ["feat/**", "infra/**", "agent/**", "ta"],
      },
      pull_request: {
        branches: ["ta"],
      },
      workflow_dispatch: {
        inputs: {
          branch: {
            description: "Branch name to simulate",
            required: true,
            default: "feat/test",
          },
        },
      },
    },
    jobs: {
      "dagger-pipeline": {
        name: "Dagger Pipeline: ${{ github.ref_name }}",
        "runs-on": "ubuntu-latest",
        steps: [
          {
            name: "Checkout",
            uses: "actions/checkout@v4",
          },
          {
            name: "Setup Bun",
            uses: "oven-sh/setup-bun@v2",
            with: {
              "bun-version": "latest",
            },
          },
          {
            name: "Install Dagger",
            uses: "dagger/dagger-for-github@v7",
            with: {
              version: "latest",
            },
          },
          {
            name: "Determine branch pattern",
            id: "branch-pattern",
            run: 'BRANCH="${{ github.ref_name }}"\n' +
              'if [[ "$BRANCH" == feat/* ]]; then\n' +
              '  echo "pattern=feat" >> $GITHUB_OUTPUT\n' +
              'elif [[ "$BRANCH" == infra/* ]]; then\n' +
              '  echo "pattern=infra" >> $GITHUB_OUTPUT\n' +
              'elif [[ "$BRANCH" == agent/* ]]; then\n' +
              '  echo "pattern=agent" >> $GITHUB_OUTPUT\n' +
              'elif [[ "$BRANCH" == "ta" ]]; then\n' +
              '  echo "pattern=trunk" >> $GITHUB_OUTPUT\n' +
              'else\n' +
              '  echo "pattern=unknown" >> $GITHUB_OUTPUT\n' +
              'fi',
          },
          {
            name: "Run Dagger Pipeline",
            env: {
              DAGGER_LOG_FORMAT: "plain",
              DAGGER_LOG_LEVEL: "info",
            },
            run: 'EVENT_TYPE="push"\n' +
              'if [ "${{ github.event_name }}" == "pull_request" ]; then\n' +
              '  EVENT_TYPE="pull_request"\n' +
              'elif [ "${{ github.event_name }}" == "workflow_dispatch" ]; then\n' +
              '  EVENT_TYPE="push"\n' +
              'fi\n' +
              '\n' +
              'bun ci/dagger.ts \\\n' +
              '  --branch "${{ github.ref_name }}" \\\n' +
              '  --event "$EVENT_TYPE" \\\n' +
              '  --base-branch "${{ github.base_ref }}"',
          },
          {
            name: "Cleanup agent sandbox (on branch delete)",
            if: "github.event_name == 'delete' && startsWith(github.ref, 'refs/heads/agent/')",
            run: 'echo "🗑️  Branch deleted, cleaning up agent sandbox..."\n' +
              '# This would trigger sandbox deletion via webhook or scheduled job\n' +
              '# For now, log the action\n' +
              'echo "Sandbox cleanup triggered for: ${{ github.ref }}"',
          },
        ],
      },
    },
  };
}

function workflowToYAML(workflow: Workflow): string {
  const lines: string[] = [];
  
  lines.push(`name: ${workflow.name}`);
  lines.push("");
  lines.push("on:");
  
  if (workflow.on.push) {
    lines.push("  push:");
    lines.push("    branches:");
    for (const branch of workflow.on.push.branches) {
      lines.push(`      - "${branch}"`);
    }
  }
  
  if (workflow.on.pull_request) {
    lines.push("  pull_request:");
    lines.push("    branches:");
    for (const branch of workflow.on.pull_request.branches) {
      lines.push(`      - "${branch}"`);
    }
  }
  
  if (workflow.on.workflow_dispatch) {
    lines.push("  workflow_dispatch:");
    if (workflow.on.workflow_dispatch.inputs) {
      lines.push("    inputs:");
      for (const [key, value] of Object.entries(workflow.on.workflow_dispatch.inputs)) {
        lines.push(`      ${key}:`);
        lines.push(`        description: "${value.description}"`);
        lines.push(`        required: ${value.required}`);
        if (value.default) {
          lines.push(`        default: "${value.default}"`);
        }
      }
    }
  }
  
  lines.push("");
  lines.push("jobs:");
  
  for (const [jobName, job] of Object.entries(workflow.jobs)) {
    lines.push(`  ${jobName}:`);
    if (job.name) {
      // Quote job name if it contains GitHub expressions or special characters
      const nameValue = job.name.includes("${{") ? `"${job.name}"` : `"${job.name}"`;
      lines.push(`    name: ${nameValue}`);
    }
    lines.push(`    runs-on: ${job["runs-on"]}`);
    lines.push("    steps:");
    
    for (const step of job.steps) {
      lines.push(`      - name: ${step.name}`);
      
      if (step.id) {
        lines.push(`        id: ${step.id}`);
      }
      
      if (step.uses) {
        lines.push(`        uses: ${step.uses}`);
      }
      
      if (step.with) {
        lines.push("        with:");
        for (const [key, value] of Object.entries(step.with)) {
          lines.push(`          ${key}: ${value}`);
        }
      }
      
      if (step.env) {
        lines.push("        env:");
        for (const [key, value] of Object.entries(step.env)) {
          lines.push(`          ${key}: ${value}`);
        }
      }
      
      if (step.if) {
        lines.push(`        if: ${step.if}`);
      }
      
      if (step.run) {
        lines.push("        run: |");
        for (const line of step.run.split("\n")) {
          lines.push(`          ${line}`);
        }
      }
    }
  }
  
  // Ensure file ends with newline
  lines.push("");
  
  return lines.join("\n");
}

function main() {
  const workflow = generateWorkflow();
  const yaml = workflowToYAML(workflow);
  
  const outputPath = join(process.cwd(), ".github", "workflows", "dagger-pipeline.yml");
  const outputDir = dirname(outputPath);
  
  mkdirSync(outputDir, { recursive: true });
  writeFileSync(outputPath, yaml, "utf-8");
  
  console.log("✅ Generated .github/workflows/dagger-pipeline.yml");
  console.log(`   ${outputPath}`);
}

if (import.meta.main) {
  main();
}
