import { Construct } from "constructs";
import { ApiObject } from "cdk8s";
import { LornuConstruct } from "../base";

export interface PreviewWorkloadProps {
  image: string;
}

/**
 * PreviewWorkload - Deploys the main preview engine for lornu.ai
 *
 * Uses ApiObject directly to ensure correct probe synthesis and match existing naming.
 */
export class PreviewWorkload extends LornuConstruct {
  constructor(scope: Construct, id: string, props: PreviewWorkloadProps) {
    // Match the user's namespace 'ai-agent-core' and component name
    super(scope, id, "ai-agent-core", "ai-agent-core");

    const labels = {
      ...this.labels,
      "app": "ai-agent-core",
    };

    new ApiObject(this, "Deployment", {
      apiVersion: "apps/v1",
      kind: "Deployment",
      metadata: {
        name: "ai-agent-core",
        namespace: this.namespace(),
        labels: labels,
      },
      spec: {
        replicas: 1,
        selector: {
          matchLabels: {
            "app": "ai-agent-core",
          },
        },
        template: {
          metadata: {
            labels: {
              "app": "ai-agent-core",
            },
          },
          spec: {
            containers: [
              {
                name: "main",
                image: props.image.trim(),
                ports: [{ containerPort: 8080 }],
                env: [
                  { name: "LORNU_ENV", value: this.env },
                  { name: "LORNU_GCP_PROJECT", value: "lornu-v2" },
                ],
                readinessProbe: {
                  httpGet: {
                    path: "/health",
                    port: 8080,
                  },
                  initialDelaySeconds: 5,
                  periodSeconds: 10,
                },
                livenessProbe: {
                  httpGet: {
                    path: "/health",
                    port: 8080,
                  },
                  initialDelaySeconds: 15,
                  periodSeconds: 20,
                },
                resources: {
                  limits: {
                    cpu: "1000m",
                    memory: "1Gi",
                  },
                  requests: {
                    cpu: "500m",
                    memory: "512Mi",
                  },
                },
              },
            ],
          },
        },
      },
    });

    // Service for ai-agent-core
    new ApiObject(this, "Service", {
      apiVersion: "v1",
      kind: "Service",
      metadata: {
        name: "ai-agent-core",
        namespace: this.namespace(),
      },
      spec: {
        type: "ClusterIP",
        selector: {
          "app": "ai-agent-core",
        },
        ports: [
          {
            port: 80,
            targetPort: 8080,
            protocol: "TCP",
          },
        ],
      },
    });

    // Ingress for preview.lornu.ai
    new ApiObject(this, "Ingress", {
      apiVersion: "networking.k8s.io/v1",
      kind: "Ingress",
      metadata: {
        name: "ai-agent-core-ingress",
        namespace: this.namespace(),
        annotations: {
          "kubernetes.io/ingress.class": "gce",
          "networking.gke.io/managed-certificates": "lornu-preview-cert",
          "kubernetes.io/ingress.allow-http": "true",
        },
      },
      spec: {
        rules: [
          {
            host: "preview.lornu.ai",
            http: {
              paths: [
                {
                  path: "/",
                  pathType: "Prefix",
                  backend: {
                    service: {
                      name: "ai-agent-core",
                      port: {
                        number: 80,
                      },
                    },
                  },
                },
              ],
            },
          },
        ],
      },
    });
  }
}
