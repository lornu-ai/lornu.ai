#!/usr/bin/env bun
/**
 * Cross-Project Google Secret Manager Sync Script
 *
 * Syncs secrets from GitHub Secrets to GCP Secret Manager with support for:
 * - Cross-project secret access (secrets in one project, accessed by workloads in another)
 * - Project-specific secret storage
 * - Automatic IAM binding for cross-project access
 * - Optional sync to Kubernetes secrets
 *
 * Usage:
 *   bun ci/sync_secrets.ts --project-id lornu-legacy --secret-name OPENAI_KEY
 *   bun ci/sync_secrets.ts --project-id lornu-v2 --secret-name DATABASE_URL
 *   bun ci/sync_secrets.ts --config secrets.json
 */

import { $ } from "bun";
import { parseArgs } from "util";

interface SecretConfig {
  name: string;
  projectId: string;
  accessorProjectId?: string; // Project that needs access (for cross-project)
  accessorServiceAccount?: string; // SA in accessor project that needs access
  value?: string; // If provided, use this; otherwise read from env
  envVar?: string; // Environment variable name (defaults to secret name)
  k8sSecretName?: string; // Kubernetes secret name (for GSM-to-K8s sync)
  k8sNamespace?: string; // Kubernetes namespace (default: lornu-ai)
}

interface SyncConfig {
  secrets: SecretConfig[];
  defaultProjectId?: string;
}

const args = parseArgs({
  options: {
    "project-id": {
      type: "string",
      description: "GCP Project ID where the secret will be stored",
    },
    "secret-name": {
      type: "string",
      description: "Name of the secret in GCP Secret Manager",
    },
    "accessor-project-id": {
      type: "string",
      description: "GCP Project ID that needs access to this secret (for cross-project)",
    },
    "accessor-service-account": {
      type: "string",
      description: "Service Account email in accessor project that needs access",
    },
    "env-var": {
      type: "string",
      description: "Environment variable name to read secret value from (defaults to secret name)",
    },
    "config": {
      type: "string",
      description: "Path to JSON config file with multiple secrets",
    },
    "dry-run": {
      type: "boolean",
      description: "Show what would be done without making changes",
      default: false,
    },
    "help": {
      type: "boolean",
      description: "Show help message",
      short: "h",
    },
  },
});

if (args.values.help) {
  console.log(`
Cross-Project Google Secret Manager Sync

Usage:
  bun ci/sync_secrets.ts [options]

Options:
  --project-id <id>              GCP Project ID where secret is stored
  --secret-name <name>           Secret name in GCP Secret Manager
  --accessor-project-id <id>     GCP Project ID that needs access (cross-project)
  --accessor-service-account <sa> Service Account email in accessor project
  --env-var <name>               Environment variable name (defaults to secret name)
  --config <path>                JSON config file with multiple secrets
  --dry-run                      Show what would be done without making changes
  --help, -h                     Show this help message

Examples:
  # Single secret in same project
  bun ci/sync_secrets.ts --project-id lornu-v2 --secret-name OPENAI_KEY

  # Cross-project secret (secret in lornu-legacy, accessed by lornu-v2)
  bun ci/sync_secrets.ts \\
    --project-id lornu-legacy \\
    --secret-name OPENAI_KEY \\
    --accessor-project-id lornu-v2 \\
    --accessor-service-account engine-sa@lornu-v2.iam.gserviceaccount.com

  # Using config file
  bun ci/sync_secrets.ts --config secrets.json

Config file format (secrets.json):
{
  "defaultProjectId": "lornu-legacy",
  "secrets": [
    {
      "name": "OPENAI_KEY",
      "projectId": "lornu-legacy",
      "accessorProjectId": "lornu-v2",
      "accessorServiceAccount": "engine-sa@lornu-v2.iam.gserviceaccount.com",
      "envVar": "OPENAI_KEY"
    },
    {
      "name": "DATABASE_URL",
      "projectId": "lornu-v2",
      "envVar": "DATABASE_URL"
    }
  ]
}
`);
  process.exit(0);
}

async function ensureSecretExists(projectId: string, secretName: string, dryRun: boolean): Promise<void> {
  console.log(`🔍 Checking if secret exists: projects/${projectId}/secrets/${secretName}`);

  try {
    await $`gcloud secrets describe ${secretName} --project=${projectId}`.quiet();
    console.log(`✅ Secret already exists: ${secretName}`);
  } catch (error) {
    if (dryRun) {
      console.log(`[DRY RUN] Would create secret: projects/${projectId}/secrets/${secretName}`);
      return;
    }

    console.log(`📝 Creating secret: projects/${projectId}/secrets/${secretName}`);
    await $`gcloud secrets create ${secretName} --project=${projectId} --replication-policy="automatic"`;
    console.log(`✅ Secret created: ${secretName}`);
  }
}

async function addSecretVersion(
  projectId: string,
  secretName: string,
  secretValue: string,
  dryRun: boolean
): Promise<void> {
  if (dryRun) {
    console.log(`[DRY RUN] Would add version to: projects/${projectId}/secrets/${secretName}`);
    return;
  }

  console.log(`📝 Adding version to secret: ${secretName}`);
  await $`echo -n ${secretValue} | gcloud secrets versions add ${secretName} --project=${projectId} --data-file=-`;
  console.log(`✅ Secret version added: ${secretName}`);
}

async function grantCrossProjectAccess(
  projectId: string,
  secretName: string,
  accessorServiceAccount: string,
  dryRun: boolean
): Promise<void> {
  if (dryRun) {
    console.log(
      `[DRY RUN] Would grant access to ${accessorServiceAccount} for secret: projects/${projectId}/secrets/${secretName}`
    );
    return;
  }

  console.log(`🔐 Granting cross-project access to ${accessorServiceAccount}...`);

  try {
    await $`gcloud secrets add-iam-policy-binding ${secretName} \
      --project=${projectId} \
      --role="roles/secretmanager.secretAccessor" \
      --member="serviceAccount:${accessorServiceAccount}"`.quiet();
    console.log(`✅ Access granted to ${accessorServiceAccount}`);
  } catch (error) {
    // Check if binding already exists
    const output = await $`gcloud secrets get-iam-policy ${secretName} --project=${projectId} --format=json`.json();
    const bindings = output.bindings || [];
    const hasAccess = bindings.some(
      (b: any) =>
        b.role === "roles/secretmanager.secretAccessor" &&
        b.members?.includes(`serviceAccount:${accessorServiceAccount}`)
    );

    if (hasAccess) {
      console.log(`ℹ️  Access already granted to ${accessorServiceAccount}`);
    } else {
      throw error;
    }
  }
}

async function syncToKubernetes(
  projectId: string,
  gsmSecretName: string,
  k8sSecretName: string,
  k8sNamespace: string,
  dryRun: boolean
): Promise<void> {
  if (dryRun) {
    console.log(`[DRY RUN] Would sync to K8s: ${k8sSecretName} in namespace ${k8sNamespace}`);
    return;
  }

  console.log(`🔄 Syncing to Kubernetes: ${k8sSecretName} in ${k8sNamespace}`);

  // Fetch secret from GSM
  const secretValue = await $`gcloud secrets versions access latest --secret=${gsmSecretName} --project=${projectId}`.text();

  // Create or update K8s secret (using stdin to avoid exposing value in command line)
  const secretYaml = `
apiVersion: v1
kind: Secret
metadata:
  name: ${k8sSecretName}
  namespace: ${k8sNamespace}
  labels:
    lornu.ai/managed-by: sync-secrets
    lornu.ai/source: gsm
type: Opaque
stringData:
  value: "${secretValue.trim().replace(/"/g, '\\"')}"
`;

  await $`echo ${secretYaml} | kubectl apply -f -`.quiet();
  console.log(`✅ K8s secret synced: ${k8sSecretName}`);
}

async function syncSecret(config: SecretConfig, dryRun: boolean): Promise<void> {
  const { name, projectId, accessorProjectId, accessorServiceAccount, value, envVar, k8sSecretName, k8sNamespace } = config;

  // Get secret value
  const secretValue = value || process.env[envVar || name];
  if (!secretValue) {
    throw new Error(
      `Secret value not found. Provide --value or set environment variable: ${envVar || name}`
    );
  }

  console.log(`\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━`);
  console.log(`📦 Syncing secret: ${name}`);
  console.log(`   Project: ${projectId}`);
  if (accessorProjectId) {
    console.log(`   Cross-project access: ${accessorProjectId}`);
  }
  console.log(`━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━`);

  // Ensure secret exists
  await ensureSecretExists(projectId, name, dryRun);

  // Add secret version
  await addSecretVersion(projectId, name, secretValue, dryRun);

  // Grant cross-project access if needed
  if (accessorProjectId && accessorServiceAccount) {
    await grantCrossProjectAccess(projectId, name, accessorServiceAccount, dryRun);
    console.log(`\n💡 To access this secret from ${accessorProjectId}, use:`);
    console.log(`   projects/${projectId}/secrets/${name}/versions/latest`);
  }

  // Sync to Kubernetes if configured
  if (k8sSecretName) {
    await syncToKubernetes(projectId, name, k8sSecretName, k8sNamespace || "lornu-ai", dryRun);
  }

  console.log(`✅ Secret synced successfully: ${name}\n`);
}

async function main() {
  const { values } = args;
  const dryRun = values["dry-run"] || false;

  if (dryRun) {
    console.log("🔍 DRY RUN MODE - No changes will be made\n");
  }

  // Load config from file if provided
  if (values.config) {
    const configPath = values.config;
    const configFile = await Bun.file(configPath).json() as SyncConfig;
    const defaultProjectId = configFile.defaultProjectId;

    for (const secretConfig of configFile.secrets) {
      // Apply default project ID if not specified
      if (!secretConfig.projectId && defaultProjectId) {
        secretConfig.projectId = defaultProjectId;
      }

      // Default envVar to secret name if not specified
      if (!secretConfig.envVar) {
        secretConfig.envVar = secretConfig.name;
      }

      await syncSecret(secretConfig, dryRun);
    }
  } else if (values["project-id"] && values["secret-name"]) {
    // Single secret mode
    const config: SecretConfig = {
      name: values["secret-name"],
      projectId: values["project-id"],
      accessorProjectId: values["accessor-project-id"],
      accessorServiceAccount: values["accessor-service-account"],
      envVar: values["env-var"],
    };

    await syncSecret(config, dryRun);
  } else {
    console.error("❌ Error: Either --config or --project-id + --secret-name must be provided");
    console.error("   Run with --help for usage information");
    process.exit(1);
  }

  console.log("✅ All secrets synced successfully!");
}

main().catch((error) => {
  console.error("❌ Error:", error.message);
  process.exit(1);
});
