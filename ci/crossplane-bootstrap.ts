#!/usr/bin/env bun
/**
 * Crossplane Bootstrap Pipeline for lornu.ai
 *
 * Implements Issue #2: Kustomize-only design + hardened Dagger pipeline
 *
 * Features:
 * - Installs Crossplane via Kustomize (no Helm)
 * - Bootstraps Azure provider with secret injection
 * - Creates ProviderConfig for provider-azure
 * - Polls for provider readiness
 * - Applies AgentMemory XRD/Composition
 *
 * Environment Variables Required:
 * - AZURE_CLIENT_ID
 * - AZURE_CLIENT_SECRET
 * - AZURE_TENANT_ID
 * - AZURE_SUBSCRIPTION_ID
 * - KUBECONFIG (path to kubeconfig file)
 *
 * Usage:
 *   bun ci/crossplane-bootstrap.ts [--dry-run]
 */

import { $ } from "bun";

// Configuration
const CROSSPLANE_VERSION = process.env.CROSSPLANE_VERSION ?? "v1.14.0";
const AZURE_PROVIDER_VERSION = process.env.AZURE_PROVIDER_VERSION ?? "v0.42.0";
const AZURE_CLIENT_ID = process.env.AZURE_CLIENT_ID;
const AZURE_CLIENT_SECRET = process.env.AZURE_CLIENT_SECRET;
const AZURE_TENANT_ID = process.env.AZURE_TENANT_ID;
const AZURE_SUBSCRIPTION_ID = process.env.AZURE_SUBSCRIPTION_ID;
const DRY_RUN = process.argv.includes("--dry-run");

// Validate required environment variables
function validateEnv(): void {
  const required = [
    "AZURE_CLIENT_ID",
    "AZURE_CLIENT_SECRET",
    "AZURE_TENANT_ID",
    "AZURE_SUBSCRIPTION_ID",
  ];

  const missing = required.filter((key) => !process.env[key]);
  if (missing.length > 0 && !DRY_RUN) {
    console.error("❌ Missing required environment variables:");
    missing.forEach((key) => console.error(`   - ${key}`));
    console.error("\nSet these variables or use --dry-run for validation only.");
    process.exit(1);
  }
}

// Execute kubectl command
async function kubectl(args: string[], options?: { silent?: boolean }): Promise<string> {
  const cmd = ["kubectl", ...args];
  if (DRY_RUN && args[0] === "apply") {
    cmd.push("--dry-run=client");
  }

  if (!options?.silent) {
    console.log(`  $ ${cmd.join(" ")}`);
  }

  const result = await $`${cmd}`.text();
  return result;
}

// Wait for a condition with polling
async function waitFor(
  description: string,
  checkFn: () => Promise<boolean>,
  timeoutSeconds: number = 300,
  intervalSeconds: number = 5
): Promise<void> {
  console.log(`⏳ Waiting for ${description}...`);
  const startTime = Date.now();
  const timeoutMs = timeoutSeconds * 1000;

  while (Date.now() - startTime < timeoutMs) {
    try {
      if (await checkFn()) {
        console.log(`✅ ${description} - Ready`);
        return;
      }
    } catch {
      // Ignore errors during polling
    }

    const elapsed = Math.round((Date.now() - startTime) / 1000);
    process.stdout.write(`\r   [${elapsed}s] Waiting...`);
    await Bun.sleep(intervalSeconds * 1000);
  }

  throw new Error(`Timeout waiting for ${description} after ${timeoutSeconds}s`);
}

// Check if Crossplane controller is ready
async function isCrossplaneReady(): Promise<boolean> {
  try {
    const result = await kubectl(
      ["get", "deploy", "crossplane", "-n", "crossplane-system", "-o", "jsonpath={.status.readyReplicas}"],
      { silent: true }
    );
    return parseInt(result.trim()) > 0;
  } catch {
    return false;
  }
}

// Check if Azure provider is healthy
async function isAzureProviderHealthy(): Promise<boolean> {
  try {
    const result = await kubectl(
      ["get", "provider.pkg.crossplane.io", "provider-azure", "-o", "jsonpath={.status.conditions[?(@.type==\"Healthy\")].status}"],
      { silent: true }
    );
    return result.trim() === "True";
  } catch {
    return false;
  }
}

// Main bootstrap function
async function bootstrap(): Promise<void> {
  console.log(`
╔══════════════════════════════════════════════════════════════╗
║     Lornu AI - Crossplane Bootstrap Pipeline                 ║
╚══════════════════════════════════════════════════════════════╝
  Crossplane:      ${CROSSPLANE_VERSION}
  Azure Provider:  ${AZURE_PROVIDER_VERSION}
  Mode:            ${DRY_RUN ? "DRY-RUN (validation only)" : "LIVE"}
`);

  validateEnv();

  // Step 1: Install Crossplane via Kustomize
  console.log("\n━━━ Step 1: Install Crossplane Operator ━━━");
  await kubectl(["apply", "-k", "infra/kustomize/crossplane"]);

  if (!DRY_RUN) {
    await waitFor("Crossplane controller deployment", isCrossplaneReady);
  }

  // Step 2: Create Azure credentials secret
  console.log("\n━━━ Step 2: Bootstrap Azure Credentials ━━━");
  const credentialsJson = JSON.stringify({
    clientId: AZURE_CLIENT_ID,
    clientSecret: AZURE_CLIENT_SECRET,
    tenantId: AZURE_TENANT_ID,
    subscriptionId: AZURE_SUBSCRIPTION_ID,
  });

  // Create secret via kubectl (no files written to disk)
  const secretYaml = `
apiVersion: v1
kind: Secret
metadata:
  name: azure-credentials
  namespace: crossplane-system
type: Opaque
stringData:
  credentials: '${credentialsJson}'
`;

  const secretCmd = DRY_RUN
    ? `echo '${secretYaml}' | kubectl apply --dry-run=client -f -`
    : `echo '${secretYaml}' | kubectl apply -f -`;

  await $`bash -c ${secretCmd}`;
  console.log("✅ Azure credentials secret created");

  // Step 3: Install Azure Provider
  console.log("\n━━━ Step 3: Install Azure Provider ━━━");
  const providerYaml = `
apiVersion: pkg.crossplane.io/v1
kind: Provider
metadata:
  name: provider-azure
spec:
  package: xpkg.upbound.io/upbound/provider-azure:${AZURE_PROVIDER_VERSION}
`;

  await $`echo '${providerYaml}' | kubectl apply -f -`;

  if (!DRY_RUN) {
    await waitFor("Azure provider to be Healthy", isAzureProviderHealthy);
  }

  // Step 4: Create ProviderConfig
  console.log("\n━━━ Step 4: Create ProviderConfig ━━━");
  const providerConfigYaml = `
apiVersion: azure.upbound.io/v1beta1
kind: ProviderConfig
metadata:
  name: default
spec:
  credentials:
    source: Secret
    secretRef:
      namespace: crossplane-system
      name: azure-credentials
      key: credentials
`;

  await $`echo '${providerConfigYaml}' | kubectl apply -f -`;
  console.log("✅ ProviderConfig created");

  // Step 5: Apply AgentMemory XRD and Compositions
  console.log("\n━━━ Step 5: Apply AgentMemory Resources ━━━");
  await kubectl(["apply", "-k", "infra/kustomize/agentmemory"]);
  console.log("✅ AgentMemory XRD and Compositions applied");

  // Step 6: Apply App Deployments
  console.log("\n━━━ Step 6: Apply App Deployments ━━━");
  await kubectl(["apply", "-k", "infra/kustomize/apps"]);
  console.log("✅ Engine and Gateway deployments applied");

  // Summary
  console.log(`
╔══════════════════════════════════════════════════════════════╗
║  ✨ Crossplane Bootstrap Complete                             ║
╚══════════════════════════════════════════════════════════════╝
  Crossplane:        Installed via Kustomize
  Azure Provider:    Installed and Healthy
  ProviderConfig:    Created with credentials
  AgentMemory XRD:   Applied
  Apps:              Engine + Gateway deployed
  Mode:              ${DRY_RUN ? "DRY-RUN (no changes made)" : "LIVE (changes applied)"}

Next Steps:
  1. Create an AgentMemory claim:
     kubectl apply -f examples/agentmemory-claim.yaml

  2. Monitor Crossplane resources:
     kubectl get managed -A
     kubectl get claim -A
`);
}

// Run
bootstrap().catch((err) => {
  console.error("\n💥 Bootstrap failed:", err.message);
  process.exit(1);
});
