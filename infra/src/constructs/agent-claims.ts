/**
 * CDK8s Constructs for Crossplane Claims
 *
 * Example claims that demonstrate how to use the AgentMemory and AgentWorker XRDs.
 * These claims can be used for testing or as templates for runtime provisioning.
 */

import { Construct } from "constructs";
import { ApiObject } from "cdk8s";

export interface AgentMemoryClaimProps {
  /** Name of the claim */
  name: string;
  /** Target namespace for the claim */
  namespace: string;
  /** Cloud provider (gcp, aws, azure) */
  provider: "gcp" | "aws" | "azure";
  /** Type of memory/database */
  type: "postgres" | "redis" | "qdrant" | "elasticsearch";
  /** Storage size (e.g., "10Gi") */
  size?: string;
  /** Performance tier */
  tier?: "small" | "medium" | "large";
  /** Environment label */
  env: string;
}

export interface AgentWorkerClaimProps {
  /** Name of the claim */
  name: string;
  /** Target namespace for the claim */
  namespace: string;
  /** Cloud provider */
  provider: "gcp" | "aws" | "azure";
  /** Whether GPU is required */
  gpu?: boolean;
  /** Type of GPU if required */
  gpuType?: "nvidia-tesla-t4" | "nvidia-tesla-a100" | "nvidia-l4";
  /** Number of worker replicas */
  replicas?: number;
  /** Maximum execution timeout */
  timeout?: string;
  /** Environment label */
  env: string;
}

/**
 * Creates an AgentMemory claim for provisioning databases/storage.
 *
 * @example
 * ```typescript
 * new AgentMemoryClaim(this, 'my-db', {
 *   name: 'agent-memory-postgres',
 *   namespace: 'lornu-ai-dev',
 *   provider: 'gcp',
 *   type: 'postgres',
 *   size: '20Gi',
 *   tier: 'medium',
 *   env: 'dev',
 * });
 * ```
 */
export class AgentMemoryClaim extends Construct {
  constructor(scope: Construct, id: string, props: AgentMemoryClaimProps) {
    super(scope, id);

    new ApiObject(this, "claim", {
      apiVersion: "lornu.ai/v1alpha1",
      kind: "AgentMemoryClaim",
      metadata: {
        name: props.name,
        namespace: props.namespace,
        labels: {
          "lornu.ai/environment": props.env,
          "lornu.ai/managed-by": "cdk8s",
          "lornu.ai/provider": props.provider,
          "lornu.ai/type": props.type,
        },
      },
      spec: {
        provider: props.provider,
        type: props.type,
        size: props.size || "10Gi",
        tier: props.tier || "small",
      },
    });
  }
}

/**
 * Creates an AgentWorker claim for provisioning compute resources.
 *
 * @example
 * ```typescript
 * new AgentWorkerClaim(this, 'my-worker', {
 *   name: 'agent-worker-gpu',
 *   namespace: 'lornu-ai-dev',
 *   provider: 'gcp',
 *   gpu: true,
 *   gpuType: 'nvidia-tesla-t4',
 *   replicas: 2,
 *   env: 'dev',
 * });
 * ```
 */
export class AgentWorkerClaim extends Construct {
  constructor(scope: Construct, id: string, props: AgentWorkerClaimProps) {
    super(scope, id);

    const spec: Record<string, unknown> = {
      provider: props.provider,
      gpu: props.gpu || false,
      replicas: props.replicas || 1,
      timeout: props.timeout || "30m",
    };

    // Only include gpuType if GPU is enabled
    if (props.gpu && props.gpuType) {
      spec.gpuType = props.gpuType;
    }

    new ApiObject(this, "claim", {
      apiVersion: "lornu.ai/v1alpha1",
      kind: "AgentWorkerClaim",
      metadata: {
        name: props.name,
        namespace: props.namespace,
        labels: {
          "lornu.ai/environment": props.env,
          "lornu.ai/managed-by": "cdk8s",
          "lornu.ai/provider": props.provider,
          "lornu.ai/gpu": String(props.gpu || false),
        },
      },
      spec,
    });
  }
}

/**
 * Creates a set of example claims for demonstration purposes.
 *
 * This construct creates one of each claim type to demonstrate
 * the full capability of the AgentMemory and AgentWorker XRDs.
 */
export class ExampleClaims extends Construct {
  constructor(
    scope: Construct,
    id: string,
    props: { namespace: string; env: string }
  ) {
    super(scope, id);

    // Example: PostgreSQL database for agent memory
    new AgentMemoryClaim(this, "example-postgres", {
      name: "example-memory-postgres",
      namespace: props.namespace,
      provider: "gcp",
      type: "postgres",
      size: "10Gi",
      tier: "small",
      env: props.env,
    });

    // Example: Redis cache for agent state
    new AgentMemoryClaim(this, "example-redis", {
      name: "example-memory-redis",
      namespace: props.namespace,
      provider: "gcp",
      type: "redis",
      size: "5Gi",
      tier: "small",
      env: props.env,
    });

    // Example: Qdrant vector database for embeddings
    new AgentMemoryClaim(this, "example-qdrant", {
      name: "example-memory-qdrant",
      namespace: props.namespace,
      provider: "gcp",
      type: "qdrant",
      size: "20Gi",
      tier: "medium",
      env: props.env,
    });

    // Example: GPU worker for inference
    new AgentWorkerClaim(this, "example-gpu-worker", {
      name: "example-worker-gpu",
      namespace: props.namespace,
      provider: "gcp",
      gpu: true,
      gpuType: "nvidia-tesla-t4",
      replicas: 1,
      timeout: "1h",
      env: props.env,
    });

    // Example: CPU worker for batch processing
    new AgentWorkerClaim(this, "example-cpu-worker", {
      name: "example-worker-cpu",
      namespace: props.namespace,
      provider: "gcp",
      gpu: false,
      replicas: 3,
      timeout: "30m",
      env: props.env,
    });
  }
}
