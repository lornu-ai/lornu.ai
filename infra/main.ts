import { App, Chart, ChartProps, ApiObject } from "cdk8s";
import { Construct } from "constructs";
import { LornuEnv, LORNU_ENV, lornuLabels } from "./src/base.js";
import { AiAgentCore } from "./src/ai_agent_core.js";
import {
  AgentXRDs,
  AgentCompositions,
  ExampleClaims,
  PreviewWorkload,
} from "./src/constructs";
// ============================================================
// Core Infrastructure Chart (Unified)
// ============================================================

export class LornuInfra extends Chart {
  public readonly env: LornuEnv;

  constructor(scope: Construct, id: string, props: ChartProps = {}) {
    super(scope, id, props);
    this.env = LORNU_ENV;

    this.synthesizeHub();
    this.synthesizeSpoke();
    this.synthesizeGitOps();
    this.synthesizeSecrets();
  }

  private synthesizeHub(): void {
    console.log(`[Hub] Synthesizing Crossplane control plane for: ${this.env}`);

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
    // --------------------------------------------------------
    // Crossplane XRDs (AgentMemory, AgentWorker)
    // --------------------------------------------------------
    new AgentXRDs(this, "agent-xrds", { env: this.env });

    // --------------------------------------------------------
    // Crossplane Compositions (GCP implementations)
    // --------------------------------------------------------
    new AgentCompositions(this, "agent-compositions", { env: this.env });

    // --------------------------------------------------------
    // Example Claims (for testing/demonstration)
    // Only create in dev environment to avoid prod resource creation
    // --------------------------------------------------------
    if (this.env === "dev") {
      const ns = `lornu-ai-${this.env}`;
      new ExampleClaims(this, "example-claims", {
        namespace: ns,
        env: this.env,
      });
      console.log(`[Hub] Created example claims in ${ns}`);
    }

    // TODO: Add Crossplane ProviderConfigs when CRDs are imported
    // Example:
    // new ProviderConfig(this, 'gcp-provider', {
    //   metadata: { name: 'gcp-default', labels: lornuLabels('crossplane') },
    //   spec: { projectID: '${GCP_PROJECT_ID}' }
    // });
  }

  private synthesizeSpoke(): void {
    console.log(`[Spoke] Synthesizing applications for: ${this.env}`);

    const projectId = process.env.GCP_PROJECT_ID || "lornu-ai";

    if (this.env === "dev") {
      // Deploy the main preview engine in dev environment
      new PreviewWorkload(this, "preview-engine", {
        image: "gcr.io/lornu-v2/ai-agent-core:latest",
      });
      console.log(`[Spoke] Created preview workload for ${this.env}`);

      // Fix for Issue #95 - Deploy AiAgentCore with resolving image name
      new AiAgentCore(this, "ai-agent-core", {
        projectId: projectId,
      });
    }
  }

  private synthesizeGitOps(): void {
    console.log(`[GitOps] Synthesizing Flux configs for: ${this.env}`);
  }

  private synthesizeSecrets(): void {
    console.log(`[Secrets] Synthesizing ESO config for: ${this.env}`);
  }
}

// ============================================================
// Standalone Execution (bun run main.ts)
// ============================================================

const isMainModule = import.meta.main;

if (isMainModule) {
  const app = new App();
  new LornuInfra(app, `lornu-${LORNU_ENV}`);
  app.synth();

  console.log(`\n✅ Synthesized ${LORNU_ENV} manifests to dist/`);
  console.log(`   Run 'bun run apply' to deploy directly to cluster`);
  console.log(`   Or commit dist/ to Git for Flux reconciliation\n`);
}
