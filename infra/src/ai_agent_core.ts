import { Construct } from "constructs";
import { ApiObject, Duration, Size } from "cdk8s";
import * as kplus from "cdk8s-plus-27";
import { LornuConstruct } from "./base.js";

interface AiAgentCoreProps {
    projectId: string;
}

export class AiAgentCore extends LornuConstruct {
    constructor(scope: Construct, id: string, props: AiAgentCoreProps) {
        // Fix for Issue #95 - Use explicit namespace to match user's cluster status
        super(scope, id, "ai-agent-core", "ai-agent-core");

        const namespace = this.namespace();
        const appName = "ai-agent-core";

        // 1. ExternalSecret to sync global vars (per issue #95)
        // This creates 'lornu-cluster-vars' in the namespace
        new ApiObject(this, "ExternalSecret", {
            apiVersion: "external-secrets.io/v1beta1",
            kind: "ExternalSecret",
            metadata: {
                name: "lornu-cluster-vars",
                namespace: namespace,
                labels: {
                    ...this.labels,
                    "lornu.ai/eso-sync": "true"
                },
            },
            spec: {
                secretStoreRef: {
                    name: "cluster-secret-store-aws",
                    kind: "ClusterSecretStore", // Assuming this exists as per issue desc
                },
                target: {
                    name: "lornu-cluster-vars",
                },
                dataFrom: [
                    {
                        extract: {
                            key: "lornu/cluster-vars",
                        },
                    },
                ],
            },
        });

        // 2. Deployment
        // Explicitly resolving GCP_PROJECT_ID here fixes the InvalidImageName error
        // instead of relying on fragile postBuild substitution.
        const image = `us-central1-docker.pkg.dev/${props.projectId}/lornu-ai/${appName}:latest`;

        new ApiObject(this, "Deployment", {
            apiVersion: "apps/v1",
            kind: "Deployment",
            metadata: {
                name: appName,
                namespace: namespace,
                labels: this.labels,
            },
            spec: {
                replicas: 1,
                selector: {
                    matchLabels: {
                        "app.kubernetes.io/name": appName,
                    },
                },
                template: {
                    metadata: {
                        labels: {
                            "app.kubernetes.io/name": appName,
                            ...this.labels,
                        },
                    },
                    spec: {
                        containers: [
                            {
                                name: "main",
                                image: image,
                                ports: [{ containerPort: 8080 }],
                                resources: {
                                    limits: { cpu: "500m", memory: "512Mi" },
                                    requests: { cpu: "100m", memory: "256Mi" },
                                },
                                envFrom: [
                                    {
                                        secretRef: {
                                            name: "lornu-cluster-vars",
                                        },
                                    },
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
                            }
                        ]
                    }
                }
            }
        });

        // 3. Service
        new ApiObject(this, "Service", {
            apiVersion: "v1",
            kind: "Service",
            metadata: {
                name: appName,
                namespace: namespace,
            },
            spec: {
                type: "ClusterIP",
                selector: {
                    "app.kubernetes.io/name": appName,
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

        // 4. Ingress
        new ApiObject(this, "Ingress", {
            apiVersion: "networking.k8s.io/v1",
            kind: "Ingress",
            metadata: {
                name: "ai-agent-core-ingress",
                namespace: namespace,
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
                                            name: appName,
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
