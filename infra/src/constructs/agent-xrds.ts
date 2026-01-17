/**
 * CDK8s Constructs for Crossplane XRDs and Compositions
 *
 * Defines the AgentMemory and AgentWorker composite resources that
 * enable agents to provision infrastructure at runtime.
 */

import { Construct } from "constructs";
import { ApiObject } from "cdk8s";

export interface AgentXRDsProps {
  env: string;
}

/**
 * Defines Crossplane XRDs (CompositeResourceDefinitions) for agent infrastructure.
 */
export class AgentXRDs extends Construct {
  constructor(scope: Construct, id: string, props: AgentXRDsProps) {
    super(scope, id);

    // AgentMemory XRD - for provisioning databases/storage
    new ApiObject(this, "agent-memory-xrd", {
      apiVersion: "apiextensions.crossplane.io/v1",
      kind: "CompositeResourceDefinition",
      metadata: {
        name: "agentmemories.lornu.ai",
        labels: {
          "lornu.ai/environment": props.env,
          "lornu.ai/managed-by": "crossplane",
        },
      },
      spec: {
        group: "lornu.ai",
        names: {
          kind: "AgentMemory",
          plural: "agentmemories",
        },
        claimNames: {
          kind: "AgentMemoryClaim",
          plural: "agentmemoryclaims",
        },
        versions: [
          {
            name: "v1alpha1",
            served: true,
            referenceable: true,
            schema: {
              openAPIV3Schema: {
                type: "object",
                properties: {
                  spec: {
                    type: "object",
                    required: ["provider", "type"],
                    properties: {
                      provider: {
                        type: "string",
                        enum: ["gcp", "aws", "azure"],
                        description: "Cloud provider for the memory resource",
                      },
                      type: {
                        type: "string",
                        enum: ["postgres", "redis", "qdrant", "elasticsearch"],
                        description: "Type of memory/database",
                      },
                      size: {
                        type: "string",
                        default: "10Gi",
                        description: "Storage size",
                      },
                      tier: {
                        type: "string",
                        enum: ["small", "medium", "large"],
                        default: "small",
                        description: "Performance tier",
                      },
                    },
                  },
                },
              },
            },
          },
        ],
      },
    });

    // AgentWorker XRD - for provisioning compute resources
    new ApiObject(this, "agent-worker-xrd", {
      apiVersion: "apiextensions.crossplane.io/v1",
      kind: "CompositeResourceDefinition",
      metadata: {
        name: "agentworkers.lornu.ai",
        labels: {
          "lornu.ai/environment": props.env,
          "lornu.ai/managed-by": "crossplane",
        },
      },
      spec: {
        group: "lornu.ai",
        names: {
          kind: "AgentWorker",
          plural: "agentworkers",
        },
        claimNames: {
          kind: "AgentWorkerClaim",
          plural: "agentworkerclaims",
        },
        versions: [
          {
            name: "v1alpha1",
            served: true,
            referenceable: true,
            schema: {
              openAPIV3Schema: {
                type: "object",
                properties: {
                  spec: {
                    type: "object",
                    required: ["provider"],
                    properties: {
                      provider: {
                        type: "string",
                        enum: ["gcp", "aws", "azure"],
                        description: "Cloud provider for compute",
                      },
                      gpu: {
                        type: "boolean",
                        default: false,
                        description: "Whether GPU is required",
                      },
                      gpuType: {
                        type: "string",
                        enum: ["nvidia-tesla-t4", "nvidia-tesla-a100", "nvidia-l4"],
                        description: "Type of GPU (if gpu=true)",
                      },
                      replicas: {
                        type: "integer",
                        default: 1,
                        minimum: 1,
                        maximum: 10,
                        description: "Number of worker replicas",
                      },
                      timeout: {
                        type: "string",
                        default: "30m",
                        description: "Maximum execution timeout",
                      },
                    },
                  },
                },
              },
            },
          },
        ],
      },
    });
  }
}

/**
 * Defines Crossplane Compositions that implement the XRDs.
 */
export class AgentCompositions extends Construct {
  constructor(scope: Construct, id: string, props: AgentXRDsProps) {
    super(scope, id);

    // AgentMemory Composition - GCP CloudSQL
    new ApiObject(this, "agent-memory-composition-gcp", {
      apiVersion: "apiextensions.crossplane.io/v1",
      kind: "Composition",
      metadata: {
        name: "agentmemory-gcp-cloudsql",
        labels: {
          "lornu.ai/environment": props.env,
          "lornu.ai/managed-by": "crossplane",
          "crossplane.io/xrd": "agentmemories.lornu.ai",
          "provider": "gcp",
        },
      },
      spec: {
        compositeTypeRef: {
          apiVersion: "lornu.ai/v1alpha1",
          kind: "AgentMemory",
        },
        mode: "Pipeline",
        pipeline: [
          {
            step: "patch-and-transform",
            functionRef: { name: "function-patch-and-transform" },
            input: {
              apiVersion: "pt.fn.crossplane.io/v1beta1",
              kind: "Resources",
              resources: [
                {
                  name: "cloudsql-instance",
                  base: {
                    apiVersion: "sql.gcp.upbound.io/v1beta1",
                    kind: "DatabaseInstance",
                    spec: {
                      forProvider: {
                        databaseVersion: "POSTGRES_15",
                        region: "us-central1",
                        settings: [
                          {
                            tier: "db-f1-micro",
                            diskSize: 10,
                            diskType: "PD_SSD",
                          },
                        ],
                        deletionProtection: false,
                      },
                    },
                  },
                  patches: [
                    {
                      type: "FromCompositeFieldPath",
                      fromFieldPath: "spec.size",
                      toFieldPath: "spec.forProvider.settings[0].diskSize",
                      transforms: [
                        {
                          type: "string",
                          string: { type: "TrimSuffix", trim: "Gi" },
                        },
                        { type: "convert", convert: { toType: "int" } },
                      ],
                    },
                    {
                      type: "FromCompositeFieldPath",
                      fromFieldPath: "spec.tier",
                      toFieldPath: "spec.forProvider.settings[0].tier",
                      transforms: [
                        {
                          type: "map",
                          map: {
                            small: "db-f1-micro",
                            medium: "db-custom-2-4096",
                            large: "db-custom-4-8192",
                          },
                        },
                      ],
                    },
                  ],
                },
              ],
            },
          },
        ],
      },
    });

    // AgentWorker Composition - GKE Job
    new ApiObject(this, "agent-worker-composition-gcp", {
      apiVersion: "apiextensions.crossplane.io/v1",
      kind: "Composition",
      metadata: {
        name: "agentworker-gcp-gke",
        labels: {
          "lornu.ai/environment": props.env,
          "lornu.ai/managed-by": "crossplane",
          "crossplane.io/xrd": "agentworkers.lornu.ai",
          "provider": "gcp",
        },
      },
      spec: {
        compositeTypeRef: {
          apiVersion: "lornu.ai/v1alpha1",
          kind: "AgentWorker",
        },
        mode: "Pipeline",
        pipeline: [
          {
            step: "patch-and-transform",
            functionRef: { name: "function-patch-and-transform" },
            input: {
              apiVersion: "pt.fn.crossplane.io/v1beta1",
              kind: "Resources",
              resources: [
                {
                  name: "worker-job",
                  base: {
                    apiVersion: "kubernetes.crossplane.io/v1alpha2",
                    kind: "Object",
                    spec: {
                      forProvider: {
                        manifest: {
                          apiVersion: "batch/v1",
                          kind: "Job",
                          metadata: {
                            namespace: "lornu-ai",
                          },
                          spec: {
                            template: {
                              spec: {
                                containers: [
                                  {
                                    name: "worker",
                                    image: "gcr.io/gcp-lornu-ai/lornu-agent-worker:v0.1.0",
                                    resources: {
                                      limits: { cpu: "1", memory: "2Gi" },
                                    },
                                  },
                                ],
                                restartPolicy: "Never",
                              },
                            },
                            backoffLimit: 3,
                          },
                        },
                      },
                    },
                  },
                  patches: [
                    {
                      type: "FromCompositeFieldPath",
                      fromFieldPath: "spec.gpu",
                      toFieldPath:
                        "spec.forProvider.manifest.spec.template.spec.containers[0].resources.limits[nvidia.com/gpu]",
                      transforms: [
                        {
                          type: "convert",
                          convert: {
                            toType: "string",
                            format: "%d",
                          },
                        },
                      ],
                    },
                  ],
                },
              ],
            },
          },
        ],
      },
    });
  }
}
