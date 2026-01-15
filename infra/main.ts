import { App, Chart, ChartProps, ApiObject } from "cdk8s";
import { Construct } from "constructs";

// Environment configuration
export type LornuEnv = "dev" | "staging" | "prod";
const LORNU_ENV: LornuEnv = (process.env.LORNU_ENV as LornuEnv) || "dev";

// Mandatory Lornu labels (enforced by CI)
export const lornuLabels = (component: string, env: LornuEnv = LORNU_ENV) => ({
  "lornu.ai/environment": env,
  "lornu.ai/managed-by": "crossplane",
  "app.kubernetes.io/name": component,
  "app.kubernetes.io/part-of": "lornu-ai",
});

// Base construct that auto-injects Lornu labels
export abstract class LornuConstruct extends Construct {
  protected readonly env: LornuEnv;
  protected readonly labels: Record<string, string>;

  constructor(scope: Construct, id: string, component: string) {
    super(scope, id);
    this.env = LORNU_ENV;
    this.labels = lornuLabels(component);
  }

  protected namespace(): string {
    return `lornu-ai-${this.env}`;
  }
}

// ============================================================
// Core Infrastructure Chart (Unified)
// ============================================================

/**
 * LornuInfra - The unified infrastructure chart
 *
 * This single chart synthesizes all Lornu AI infrastructure:
 * - Hub: Crossplane control plane, provider configs
 * - Spoke: Application deployments, services
 * - GitOps: Flux Kustomizations, image policies
 * - Secrets: ESO ClusterSecretStores
 */
export class LornuInfra extends Chart {
  public readonly env: LornuEnv;

  constructor(scope: Construct, id: string, props: ChartProps = {}) {
    super(scope, id, props);
    this.env = LORNU_ENV;

    // --------------------------------------------------------
    // Hub Infrastructure (Crossplane control plane)
    // --------------------------------------------------------
    this.synthesizeHub();

    // --------------------------------------------------------
    // Spoke Applications (Workloads)
    // --------------------------------------------------------
    this.synthesizeSpoke();

    // --------------------------------------------------------
    // GitOps (Flux configurations)
    // --------------------------------------------------------
    this.synthesizeGitOps();

    // --------------------------------------------------------
    // Secrets (ESO)
    // --------------------------------------------------------
    this.synthesizeSecrets();
  }

  private synthesizeHub(): void {
    console.log(`[Hub] Synthesizing Crossplane control plane for: ${this.env}`);

    // Bootstrap ConfigMap - proves CDK8s → SSA flow works
    new ApiObject(this, "lornu-bootstrap-config", {
      apiVersion: "v1",
      kind: "ConfigMap",
      metadata: {
        name: "lornu-bootstrap-config",
        namespace: "default",
        labels: lornuLabels("bootstrap", this.env),
      },
      data: {
        LORNU_ENV: this.env,
        MANAGED_BY: "cdk8s-ssa",
        SYNTH_TIME: new Date().toISOString(),
      },
    });

    // TODO: Add Crossplane ProviderConfigs when CRDs are imported
    // Example:
    // new ProviderConfig(this, 'gcp-provider', {
    //   metadata: { name: 'gcp-default', labels: lornuLabels('crossplane') },
    //   spec: { projectID: '${GCP_PROJECT_ID}' }
    // });
  }

  private synthesizeSpoke(): void {
    console.log(`[Spoke] Synthesizing applications for: ${this.env}`);

    // TODO: Add application deployments when constructs are ready
    // Example:
    // new Deployment(this, 'api', {
    //   metadata: { namespace: this.namespace(), labels: lornuLabels('api') },
    //   spec: { ... }
    // });
  }

  private synthesizeGitOps(): void {
    console.log(`[GitOps] Synthesizing Flux configs for: ${this.env}`);

    // TODO: Add Flux Kustomizations when CRDs are imported
    // Example:
    // new Kustomization(this, 'spoke-apps', {
    //   metadata: { namespace: 'flux-system', labels: lornuLabels('flux') },
    //   spec: { path: './crossplane/spoke/apps', ... }
    // });
  }

  private synthesizeSecrets(): void {
    console.log(`[Secrets] Synthesizing ESO config for: ${this.env}`);

    // TODO: Add ClusterSecretStore when CRDs are imported
    // Example:
    // new ClusterSecretStore(this, 'aws-secrets', {
    //   metadata: { name: 'aws-secrets-manager-global' },
    //   spec: { provider: { aws: { region: 'us-east-2' } } }
    // });
  }

  private namespace(): string {
    return `lornu-ai-${this.env}`;
  }
}

// ============================================================
// Standalone Execution (bun run main.ts)
// ============================================================

// Only run synthesis when executed directly (not imported)
const isMainModule = import.meta.main;

if (isMainModule) {
  const app = new App();
  new LornuInfra(app, `lornu-${LORNU_ENV}`);
  app.synth();

  console.log(`\n✅ Synthesized ${LORNU_ENV} manifests to dist/`);
  console.log(`   Run 'bun run apply' to deploy directly to cluster`);
  console.log(`   Or commit dist/ to Git for Flux reconciliation\n`);
}
