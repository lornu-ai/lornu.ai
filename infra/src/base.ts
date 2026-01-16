import { Construct } from "constructs";

// Environment configuration
export type LornuEnv = "dev" | "staging" | "prod";
export const LORNU_ENV: LornuEnv = (process.env.LORNU_ENV as LornuEnv) || "dev";

// Mandatory Lornu labels (enforced by CI)
export const lornuLabels = (component: string, env: LornuEnv = LORNU_ENV) => ({
  "lornu.ai/environment": env,
  "lornu.ai/managed-by": "crossplane",
  "app.kubernetes.io/name": component,
  "app.kubernetes.io/part-of": "lornu-ai",
});

/**
 * Base construct that auto-injects Lornu labels
 */
export abstract class LornuConstruct extends Construct {
  protected readonly env: LornuEnv;
  protected readonly labels: Record<string, string>;
  protected readonly customNamespace?: string;

  constructor(scope: Construct, id: string, component: string, customNamespace?: string) {
    super(scope, id);
    this.env = LORNU_ENV;
    this.labels = lornuLabels(component, this.env);
    this.customNamespace = customNamespace;
  }

  protected namespace(): string {
    return this.customNamespace || `lornu-ai-${this.env}`;
  }
}
