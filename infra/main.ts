import { App, Chart, ChartProps } from "cdk8s";
import { Construct } from "constructs";

// Environment configuration
type LornuEnv = "dev" | "staging" | "prod";
const LORNU_ENV: LornuEnv = (process.env.LORNU_ENV as LornuEnv) || "dev";

// Mandatory Lornu labels (enforced by CI)
const lornuLabels = (component: string) => ({
  "lornu.ai/environment": LORNU_ENV,
  "lornu.ai/managed-by": "crossplane",
  "app.kubernetes.io/name": component,
  "app.kubernetes.io/part-of": "lornu-ai",
});

// Base construct that auto-injects Lornu labels
abstract class LornuConstruct extends Construct {
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
// Core Infrastructure Charts
// ============================================================

/**
 * Hub Infrastructure - Crossplane control plane components
 * Deployed to: flux-system, crossplane-system namespaces
 */
class HubInfraChart extends Chart {
  constructor(scope: Construct, id: string, props: ChartProps = {}) {
    super(scope, id, props);

    // Crossplane ProviderConfigs will be defined here
    // Using imported CRDs from cdk8s import

    console.log(`[HubInfra] Synthesizing for environment: ${LORNU_ENV}`);
  }
}

/**
 * Spoke Applications - Workload deployments
 * Deployed to: lornu-ai-{dev,staging,prod} namespaces
 */
class SpokeAppsChart extends Chart {
  constructor(scope: Construct, id: string, props: ChartProps = {}) {
    super(scope, id, props);

    console.log(`[SpokeApps] Synthesizing for environment: ${LORNU_ENV}`);
  }
}

/**
 * GitOps Configuration - Flux Kustomizations
 * Deployed to: flux-system namespace
 */
class GitOpsChart extends Chart {
  constructor(scope: Construct, id: string, props: ChartProps = {}) {
    super(scope, id, props);

    console.log(`[GitOps] Synthesizing Flux configs for: ${LORNU_ENV}`);
  }
}

/**
 * Secrets Infrastructure - ESO ClusterSecretStores
 * Deployed to: external-secrets namespace
 */
class SecretsChart extends Chart {
  constructor(scope: Construct, id: string, props: ChartProps = {}) {
    super(scope, id, props);

    console.log(`[Secrets] Synthesizing ESO config for: ${LORNU_ENV}`);
  }
}

// ============================================================
// Main Application Entry Point
// ============================================================

const app = new App();

// Synthesize all charts
new HubInfraChart(app, `lornu-hub-${LORNU_ENV}`);
new SpokeAppsChart(app, `lornu-spoke-${LORNU_ENV}`);
new GitOpsChart(app, `lornu-gitops-${LORNU_ENV}`);
new SecretsChart(app, `lornu-secrets-${LORNU_ENV}`);

// Generate manifests to dist/
app.synth();

console.log(`\n✅ Synthesized ${LORNU_ENV} manifests to dist/`);
console.log(`   Run 'bun run apply' to deploy directly to cluster`);
console.log(`   Or commit dist/ to Git for Flux reconciliation\n`);
