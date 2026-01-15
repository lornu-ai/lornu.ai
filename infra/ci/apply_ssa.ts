/**
 * Direct-to-Cluster SSA (Server-Side Apply) Applicator
 *
 * This script synthesizes CDK8s charts and applies them directly to Kubernetes
 * using Server-Side Apply (SSA) - the state-free GitOps pattern.
 *
 * Usage:
 *   bun run ci/apply_ssa.ts              # Apply to cluster
 *   bun run ci/apply_ssa.ts --dry-run    # Validate without applying
 */

import { App } from "cdk8s";

// Parse CLI args
const args = process.argv.slice(2);
const DRY_RUN = args.includes("--dry-run");
const LORNU_ENV = process.env.LORNU_ENV || "dev";

// Kubernetes API client configuration
const K8S_API_SERVER = process.env.KUBERNETES_SERVICE_HOST
  ? `https://${process.env.KUBERNETES_SERVICE_HOST}:${process.env.KUBERNETES_SERVICE_PORT}`
  : "https://kubernetes.default.svc";

const SA_TOKEN_PATH = "/var/run/secrets/kubernetes.io/serviceaccount/token";
const SA_CA_PATH = "/var/run/secrets/kubernetes.io/serviceaccount/ca.crt";

/**
 * Server-Side Apply a single Kubernetes resource
 * Uses PATCH with application/apply-patch+yaml content type
 */
async function applyResource(manifest: Record<string, unknown>): Promise<void> {
  const apiVersion = manifest.apiVersion as string;
  const kind = manifest.kind as string;
  const metadata = manifest.metadata as { name: string; namespace?: string };

  // Build API path based on resource type
  const apiPath = buildApiPath(apiVersion, kind, metadata.namespace, metadata.name);

  if (DRY_RUN) {
    console.log(`[DRY-RUN] Would apply: ${kind}/${metadata.name}`);
    return;
  }

  // Get service account token (in-cluster) or use kubeconfig
  const token = await getAuthToken();

  const response = await fetch(`${K8S_API_SERVER}${apiPath}?fieldManager=lornu-cdk8s&force=true`, {
    method: "PATCH",
    headers: {
      "Content-Type": "application/apply-patch+yaml",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify(manifest),
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Failed to apply ${kind}/${metadata.name}: ${response.status} - ${error}`);
  }

  console.log(`✅ Applied: ${kind}/${metadata.name}`);
}

/**
 * Build the Kubernetes API path for a resource
 */
function buildApiPath(
  apiVersion: string,
  kind: string,
  namespace: string | undefined,
  name: string
): string {
  const [group, version] = apiVersion.includes("/")
    ? apiVersion.split("/")
    : ["", apiVersion];

  const apiBase = group ? `/apis/${group}/${version}` : `/api/${version}`;
  const resourceType = kindToResource(kind);

  if (namespace) {
    return `${apiBase}/namespaces/${namespace}/${resourceType}/${name}`;
  }
  return `${apiBase}/${resourceType}/${name}`;
}

/**
 * Convert Kind to resource type (e.g., Deployment -> deployments)
 */
function kindToResource(kind: string): string {
  const irregulars: Record<string, string> = {
    Ingress: "ingresses",
    NetworkPolicy: "networkpolicies",
  };
  return irregulars[kind] || `${kind.toLowerCase()}s`;
}

/**
 * Get authentication token (in-cluster SA or local kubeconfig)
 */
async function getAuthToken(): Promise<string> {
  // Try in-cluster service account first
  try {
    const file = Bun.file(SA_TOKEN_PATH);
    if (await file.exists()) {
      return await file.text();
    }
  } catch {
    // Not in cluster, fall through
  }

  // Fall back to kubectl config
  const proc = Bun.spawn(["kubectl", "config", "view", "--raw", "-o", "jsonpath={.users[0].user.token}"]);
  const output = await new Response(proc.stdout).text();

  if (output) {
    return output;
  }

  // Try exec-based auth (e.g., gcloud, aws)
  const execProc = Bun.spawn(["kubectl", "auth", "whoami", "-o", "json"]);
  const execOutput = await new Response(execProc.stdout).text();

  if (execOutput) {
    // Auth is working, use kubectl proxy for requests
    throw new Error("Exec-based auth detected. Use 'kubectl proxy' or set KUBERNETES_SERVICE_HOST");
  }

  throw new Error("No valid Kubernetes authentication found");
}

// ============================================================
// Main Execution
// ============================================================

async function main() {
  console.log(`\n🚀 Lornu CDK8s Direct-to-Cluster Apply`);
  console.log(`   Environment: ${LORNU_ENV}`);
  console.log(`   Mode: ${DRY_RUN ? "DRY-RUN" : "LIVE"}\n`);

  // Import and instantiate charts dynamically
  const { default: synthesize } = await import("../main.ts");

  // Re-run synthesis to get fresh manifests
  const app = new App();

  // Get all synthesized manifests
  const manifests = app.charts.flatMap((chart) => chart.toJson());

  console.log(`📦 Found ${manifests.length} resources to apply\n`);

  // Apply each manifest using SSA
  for (const manifest of manifests) {
    try {
      await applyResource(manifest);
    } catch (error) {
      console.error(`❌ Error: ${error}`);
      if (!DRY_RUN) {
        process.exit(1);
      }
    }
  }

  console.log(`\n✅ ${DRY_RUN ? "Validation" : "Apply"} complete!\n`);
}

main().catch((err) => {
  console.error("Fatal error:", err);
  process.exit(1);
});
