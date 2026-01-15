#!/usr/bin/env bun
/**
 * Generate Tekton Pipeline/Task YAML from TypeScript
 *
 * This script generates Tekton resources for the Dagger CI pipeline
 * following the "no YAML" principle - all YAML is generated from code.
 *
 * Output: ci/tekton/
 *   - task-dagger-pipeline.yaml
 *   - pipeline-dagger.yaml
 *   - pipelinerun-template.yaml
 */

import { writeFileSync, mkdirSync } from "fs";
import { join } from "path";
import * as yaml from "yaml";

interface TektonMetadata {
  name: string;
  namespace?: string;
  labels?: Record<string, string>;
  annotations?: Record<string, string>;
}

interface TektonParam {
  name: string;
  type?: string;
  description?: string;
  default?: string;
}

interface TektonStep {
  name: string;
  image: string;
  script?: string;
  command?: string[];
  args?: string[];
  env?: Array<{ name: string; value: string }>;
  workingDir?: string;
}

interface TektonTask {
  apiVersion: string;
  kind: "Task";
  metadata: TektonMetadata;
  spec: {
    description?: string;
    params?: TektonParam[];
    workspaces?: Array<{ name: string; description?: string }>;
    steps: TektonStep[];
  };
}

interface TektonPipeline {
  apiVersion: string;
  kind: "Pipeline";
  metadata: TektonMetadata;
  spec: {
    description?: string;
    params?: TektonParam[];
    workspaces?: Array<{ name: string; description?: string }>;
    tasks: Array<{
      name: string;
      taskRef: { name: string };
      params?: Array<{ name: string; value: string }>;
      workspaces?: Array<{ name: string; workspace: string }>;
    }>;
  };
}

interface TektonPipelineRun {
  apiVersion: string;
  kind: "PipelineRun";
  metadata: TektonMetadata;
  spec: {
    pipelineRef: { name: string };
    params?: Array<{ name: string; value: string }>;
    workspaces?: Array<{
      name: string;
      volumeClaimTemplate?: {
        spec: {
          accessModes: string[];
          resources: { requests: { storage: string } };
        };
      };
    }>;
  };
}

const LABELS = {
  "lornu.ai/component": "tekton",
  "lornu.ai/managed-by": "cdk8s",
  "app.kubernetes.io/part-of": "lornu-ai",
};

function generateDaggerTask(): TektonTask {
  return {
    apiVersion: "tekton.dev/v1",
    kind: "Task",
    metadata: {
      name: "dagger-pipeline",
      namespace: "tekton-pipelines",
      labels: LABELS,
      annotations: {
        "tekton.dev/displayName": "Dagger Pipeline Task",
      },
    },
    spec: {
      description:
        "Runs the Dagger CI pipeline for lornu.ai based on branch pattern",
      params: [
        {
          name: "branch",
          type: "string",
          description: "Git branch name",
        },
        {
          name: "event",
          type: "string",
          description: "Git event type (push, pull_request, merge)",
          default: "push",
        },
        {
          name: "base-branch",
          type: "string",
          description: "Base branch for PR events",
          default: "",
        },
      ],
      workspaces: [
        {
          name: "source",
          description: "The git repo source code",
        },
      ],
      steps: [
        {
          name: "install-dependencies",
          image: "oven/bun:latest",
          workingDir: "$(workspaces.source.path)",
          script: `#!/bin/bash
set -e
echo "📦 Installing dependencies..."
bun install
echo "✅ Dependencies installed"
`,
        },
        {
          name: "run-dagger-pipeline",
          image: "ghcr.io/dagger/dagger:latest",
          workingDir: "$(workspaces.source.path)",
          env: [
            { name: "DAGGER_LOG_FORMAT", value: "plain" },
            { name: "DAGGER_LOG_LEVEL", value: "info" },
          ],
          script: `#!/bin/bash
set -e

BRANCH="$(params.branch)"
EVENT="$(params.event)"
BASE_BRANCH="$(params.base-branch)"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 Branch: $BRANCH"
echo "🔍 Event: $EVENT"
echo "🔍 Base Branch: $BASE_BRANCH"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Determine branch pattern
if [[ "$BRANCH" == feat/* ]]; then
  PATTERN="feat"
elif [[ "$BRANCH" == infra/* ]]; then
  PATTERN="infra"
elif [[ "$BRANCH" == agent/* ]]; then
  PATTERN="agent"
elif [[ "$BRANCH" == "ta" ]]; then
  PATTERN="trunk"
else
  PATTERN="unknown"
fi

echo "🔍 Pattern: $PATTERN"

# Run Dagger pipeline
bun ci/dagger.ts \\
  --branch "$BRANCH" \\
  --event "$EVENT" \\
  --base-branch "$BASE_BRANCH"

echo "✅ Dagger pipeline completed!"
`,
        },
      ],
    },
  };
}

function generateDaggerPipeline(): TektonPipeline {
  return {
    apiVersion: "tekton.dev/v1",
    kind: "Pipeline",
    metadata: {
      name: "dagger-ci",
      namespace: "tekton-pipelines",
      labels: LABELS,
      annotations: {
        "tekton.dev/displayName": "Dagger CI Pipeline",
      },
    },
    spec: {
      description:
        "CI Pipeline for lornu.ai using Dagger. Clones repo and runs branch-specific pipeline.",
      params: [
        {
          name: "repo-url",
          type: "string",
          description: "Git repository URL",
          default: "https://github.com/lornu-ai/lornu.ai.git",
        },
        {
          name: "revision",
          type: "string",
          description: "Git revision (branch, tag, or SHA)",
          default: "ta",
        },
        {
          name: "event",
          type: "string",
          description: "Git event type",
          default: "push",
        },
        {
          name: "base-branch",
          type: "string",
          description: "Base branch for PR events",
          default: "",
        },
      ],
      workspaces: [
        {
          name: "shared-workspace",
          description: "Workspace shared across tasks",
        },
      ],
      tasks: [
        {
          name: "clone-repo",
          taskRef: { name: "git-clone" },
          params: [
            { name: "url", value: "$(params.repo-url)" },
            { name: "revision", value: "$(params.revision)" },
            { name: "depth", value: "1" },
          ],
          workspaces: [{ name: "output", workspace: "shared-workspace" }],
        },
        {
          name: "run-dagger",
          taskRef: { name: "dagger-pipeline" },
          params: [
            { name: "branch", value: "$(params.revision)" },
            { name: "event", value: "$(params.event)" },
            { name: "base-branch", value: "$(params.base-branch)" },
          ],
          workspaces: [{ name: "source", workspace: "shared-workspace" }],
        },
      ],
    },
  };
}

function generatePipelineRunTemplate(): TektonPipelineRun {
  return {
    apiVersion: "tekton.dev/v1",
    kind: "PipelineRun",
    metadata: {
      name: "dagger-ci-run-TIMESTAMP",
      namespace: "tekton-pipelines",
      labels: {
        ...LABELS,
        "tekton.dev/pipeline": "dagger-ci",
      },
    },
    spec: {
      pipelineRef: { name: "dagger-ci" },
      params: [
        { name: "repo-url", value: "https://github.com/lornu-ai/lornu.ai.git" },
        { name: "revision", value: "feat/example-branch" },
        { name: "event", value: "push" },
        { name: "base-branch", value: "" },
      ],
      workspaces: [
        {
          name: "shared-workspace",
          volumeClaimTemplate: {
            spec: {
              accessModes: ["ReadWriteOnce"],
              resources: { requests: { storage: "1Gi" } },
            },
          },
        },
      ],
    },
  };
}

function main() {
  const outputDir = join(process.cwd(), "ci", "tekton");
  mkdirSync(outputDir, { recursive: true });

  // Generate Task
  const task = generateDaggerTask();
  const taskYaml = yaml.stringify(task, { lineWidth: 0 });
  writeFileSync(join(outputDir, "task-dagger-pipeline.yaml"), taskYaml);
  console.log("✅ Generated ci/tekton/task-dagger-pipeline.yaml");

  // Generate Pipeline
  const pipeline = generateDaggerPipeline();
  const pipelineYaml = yaml.stringify(pipeline, { lineWidth: 0 });
  writeFileSync(join(outputDir, "pipeline-dagger.yaml"), pipelineYaml);
  console.log("✅ Generated ci/tekton/pipeline-dagger.yaml");

  // Generate PipelineRun template
  const pipelineRun = generatePipelineRunTemplate();
  const pipelineRunYaml =
    "# Template PipelineRun - replace TIMESTAMP with actual value\n" +
    "# Example: kubectl create -f - <<< \"$(sed 's/TIMESTAMP/'$(date +%s)'/g' ci/tekton/pipelinerun-template.yaml)\"\n" +
    yaml.stringify(pipelineRun, { lineWidth: 0 });
  writeFileSync(join(outputDir, "pipelinerun-template.yaml"), pipelineRunYaml);
  console.log("✅ Generated ci/tekton/pipelinerun-template.yaml");

  // Generate kustomization.yaml
  const kustomization = {
    apiVersion: "kustomize.config.k8s.io/v1beta1",
    kind: "Kustomization",
    resources: ["task-dagger-pipeline.yaml", "pipeline-dagger.yaml"],
  };
  const kustomizationYaml = yaml.stringify(kustomization);
  writeFileSync(join(outputDir, "kustomization.yaml"), kustomizationYaml);
  console.log("✅ Generated ci/tekton/kustomization.yaml");

  console.log("\n📋 To apply to your GKE cluster:");
  console.log("   kubectl apply -k ci/tekton/");
  console.log("\n📋 To trigger a pipeline run:");
  console.log(
    '   kubectl create -f - <<< "$(sed \'s/TIMESTAMP/\'$(date +%s)\'/g\' ci/tekton/pipelinerun-template.yaml)"'
  );
}

if (import.meta.main) {
  main();
}
