/**
 * Direct-to-Cluster SSA (Server-Side Apply) Orchestrator
 *
 * This script synthesizes CDK8s charts and applies them directly to Kubernetes
 * using Server-Side Apply (SSA) - the state-free GitOps pattern.
 *
 * Features:
 * - Official @kubernetes/client-node for reliable API access
 * - Crossplane health checks - waits for cloud resources to be Ready
 * - Dry-run validation against K8s API schemas
 * - Force conflict resolution (code wins over manual changes)
 *
 * Usage:
 *   bun run ci/apply_ssa.ts                    # Apply to cluster
 *   bun run ci/apply_ssa.ts --dry-run          # Validate without applying
 *   bun run ci/apply_ssa.ts --wait             # Apply and wait for Ready
 *   bun run ci/apply_ssa.ts --timeout=300      # Custom timeout (seconds)
 */

import * as k8s from "@kubernetes/client-node";
import { App } from "cdk8s";
import { LornuInfra } from "../main";

// ============================================================
// Configuration
// ============================================================

const args = process.argv.slice(2);
const DRY_RUN = args.includes("--dry-run");
const WAIT_FOR_READY = args.includes("--wait");
const TIMEOUT_ARG = args.find((a) => a.startsWith("--timeout="));
const TIMEOUT_SECONDS = TIMEOUT_ARG ? parseInt(TIMEOUT_ARG.split("=")[1]) : 300;
const LORNU_ENV = process.env.LORNU_ENV || "dev";

// Crossplane resource types that need health checks
const CROSSPLANE_KINDS = [
  "Instance",        // Cloud SQL, RDS
  "Bucket",          // GCS, S3
  "Cluster",         // GKE, EKS
  "Database",        // Managed databases
  "VPCNetwork",      // Networking
  "Subnetwork",
  "ServiceAccount",
  "IAMMember",
];

// ============================================================
// Kubernetes Client Setup
// ============================================================

const kc = new k8s.KubeConfig();
kc.loadFromDefault();
const client = k8s.KubernetesObjectApi.makeApiClient(kc);
const customApi = kc.makeApiClient(k8s.CustomObjectsApi);

// ============================================================
// Core Functions
// ============================================================

/**
 * Server-Side Apply a single Kubernetes resource
 */
async function applyResource(manifest: k8s.KubernetesObject): Promise<void> {
  const name = manifest.metadata?.name;
  const kind = manifest.kind;
  const namespace = manifest.metadata?.namespace;

  const displayName = namespace ? `${namespace}/${name}` : name;

  try {
    await client.patch(
      manifest,
      undefined,                           // pretty
      undefined,                           // dryRun (handled via options)
      "lornu-manager",                     // fieldManager
      true,                                // force (resolve conflicts)
      {
        headers: { "Content-Type": "application/apply-patch+yaml" },
        ...(DRY_RUN && { qs: { dryRun: "All" } }),
      }
    );

    const prefix = DRY_RUN ? "[DRY-RUN] " : "";
    console.log(`✅ ${prefix}Applied ${kind}: ${displayName}`);
  } catch (err: any) {
    const message = err.body?.message || err.message || String(err);
    throw new Error(`Failed to apply ${kind}/${displayName}: ${message}`);
  }
}

/**
 * Check if a Crossplane managed resource is Ready
 */
async function isResourceReady(
  apiVersion: string,
  kind: string,
  namespace: string | undefined,
  name: string
): Promise<{ ready: boolean; message: string }> {
  try {
    const [group, version] = apiVersion.includes("/")
      ? apiVersion.split("/")
      : ["", apiVersion];

    let resource: any;

    if (namespace) {
      resource = await customApi.getNamespacedCustomObject(
        group,
        version,
        namespace,
        pluralize(kind),
        name
      );
    } else {
      resource = await customApi.getClusterCustomObject(
        group,
        version,
        pluralize(kind),
        name
      );
    }

    const status = resource.body?.status;
    if (!status) {
      return { ready: false, message: "No status yet" };
    }

    // Check Crossplane conditions
    const conditions = status.conditions || [];
    const readyCondition = conditions.find(
      (c: any) => c.type === "Ready" || c.type === "Synced"
    );

    if (readyCondition?.status === "True") {
      return { ready: true, message: readyCondition.reason || "Ready" };
    }

    // Check for errors
    const errorCondition = conditions.find(
      (c: any) => c.status === "False" && c.message
    );

    if (errorCondition) {
      return { ready: false, message: errorCondition.message };
    }

    return { ready: false, message: "Waiting for reconciliation..." };
  } catch (err: any) {
    return { ready: false, message: err.message || "Error checking status" };
  }
}

/**
 * Wait for a Crossplane resource to become Ready
 */
async function waitForReady(
  manifest: k8s.KubernetesObject,
  timeoutSeconds: number
): Promise<void> {
  const kind = manifest.kind!;
  const name = manifest.metadata?.name!;
  const namespace = manifest.metadata?.namespace;
  const apiVersion = manifest.apiVersion!;

  // Skip non-Crossplane resources
  if (!CROSSPLANE_KINDS.includes(kind)) {
    return;
  }

  const displayName = namespace ? `${namespace}/${name}` : name;
  console.log(`⏳ Waiting for ${kind}/${displayName} to be Ready...`);

  const startTime = Date.now();
  const timeoutMs = timeoutSeconds * 1000;

  while (Date.now() - startTime < timeoutMs) {
    const { ready, message } = await isResourceReady(
      apiVersion,
      kind,
      namespace,
      name
    );

    if (ready) {
      console.log(`✅ ${kind}/${displayName} is Ready: ${message}`);
      return;
    }

    // Check for permanent failures
    if (message.includes("Error") || message.includes("Failed")) {
      throw new Error(`${kind}/${displayName} failed: ${message}`);
    }

    // Progress indicator
    const elapsed = Math.round((Date.now() - startTime) / 1000);
    process.stdout.write(`\r   [${elapsed}s] ${message}`.padEnd(80));

    await sleep(5000); // Poll every 5 seconds
  }

  throw new Error(
    `Timeout waiting for ${kind}/${displayName} after ${timeoutSeconds}s`
  );
}

// ============================================================
// Utilities
// ============================================================

function pluralize(kind: string): string {
  const irregulars: Record<string, string> = {
    Ingress: "ingresses",
    NetworkPolicy: "networkpolicies",
  };
  return irregulars[kind] || `${kind.toLowerCase()}s`;
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// ============================================================
// Main Execution
// ============================================================

async function main() {
  console.log(`
╔══════════════════════════════════════════════════════════════╗
║          Lornu AI - CDK8s SSA Orchestrator                   ║
╚══════════════════════════════════════════════════════════════╝
  Environment:  ${LORNU_ENV}
  Mode:         ${DRY_RUN ? "DRY-RUN (validation only)" : "LIVE"}
  Wait:         ${WAIT_FOR_READY ? `Yes (${TIMEOUT_SECONDS}s timeout)` : "No"}
`);

  // Synthesize CDK8s charts
  console.log("📦 Synthesizing infrastructure...\n");
  const app = new App();
  const chart = new LornuInfra(app, `lornu-${LORNU_ENV}`);
  const manifests = chart.toJson() as k8s.KubernetesObject[];

  console.log(`   Found ${manifests.length} resources\n`);

  // Phase 1: Apply all resources
  console.log("🚀 Phase 1: Applying resources...\n");
  const crossplaneResources: k8s.KubernetesObject[] = [];

  for (const manifest of manifests) {
    try {
      await applyResource(manifest);

      // Track Crossplane resources for health checks
      if (CROSSPLANE_KINDS.includes(manifest.kind!)) {
        crossplaneResources.push(manifest);
      }
    } catch (error) {
      console.error(`\n❌ ${error}`);
      if (!DRY_RUN) {
        process.exit(1);
      }
    }
  }

  // Phase 2: Wait for Crossplane resources (if --wait flag)
  if (WAIT_FOR_READY && !DRY_RUN && crossplaneResources.length > 0) {
    console.log(`\n⏳ Phase 2: Waiting for ${crossplaneResources.length} Crossplane resources...\n`);

    for (const manifest of crossplaneResources) {
      try {
        await waitForReady(manifest, TIMEOUT_SECONDS);
      } catch (error) {
        console.error(`\n❌ ${error}`);
        process.exit(1);
      }
    }
  }

  // Summary
  console.log(`
╔══════════════════════════════════════════════════════════════╗
║  ✨ Infrastructure sync complete                              ║
╚══════════════════════════════════════════════════════════════╝
  Applied:      ${manifests.length} resources
  Crossplane:   ${crossplaneResources.length} cloud resources
  Status:       ${DRY_RUN ? "Validated (no changes)" : "Applied successfully"}
`);
}

main().catch((err) => {
  console.error("\n💥 Fatal error:", err.message || err);
  process.exit(1);
});
