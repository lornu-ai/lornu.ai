import { Construct } from "constructs";
import { ApiObject, Duration, Size } from "cdk8s";
import * as kplus from "cdk8s-plus-27";
import { LornuConstruct } from "./base.js";

interface AiAgentCoreProps {
    projectId: string;
}

export class AiAgentCore extends LornuConstruct {
    constructor(scope: Construct, id: string, props: AiAgentCoreProps) {
        super(scope, id, "ai-agent-core");

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

        const deployment = new kplus.Deployment(this, "Deployment", {
            metadata: {
                name: appName,
                namespace: namespace,
                labels: this.labels,
            },
            replicas: 1,
            containers: [
                {
                    image: image,
                    name: "main",
                    resources: {
                        cpu: { request: kplus.Cpu.millis(100), limit: kplus.Cpu.millis(500) },
                        memory: { request: Size.mebibytes(256), limit: Size.mebibytes(512) },
                    },
                    envFrom: [
                        // Inject the synced secrets as env vars
                        kplus.Env.fromSecret(kplus.Secret.fromSecretName(this, "VarsSecret", "lornu-cluster-vars"))
                    ]
                }
            ]
        });
    }
}
