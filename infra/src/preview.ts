import { Construct } from "constructs";
import { Size, Duration } from "cdk8s";
import * as kplus from "cdk8s-plus-27";
import { LornuConstruct } from "./base.js";

export class PreviewWorkload extends LornuConstruct {
    constructor(scope: Construct, id: string) {
        super(scope, id, "preview-site");

        const deployment = new kplus.Deployment(this, "Deployment", {
            metadata: {
                name: "lornu-v2-preview",
                namespace: this.namespace(),
                labels: this.labels,
            },
            replicas: 1,
        });

        const container = deployment.addContainer({
            image: "nginx:latest", // Placeholder as user didn't specify
            portNumber: 80,
            resources: {
                cpu: { request: kplus.Cpu.millis(100), limit: kplus.Cpu.millis(500) },
                memory: { request: Size.mebibytes(128), limit: Size.mebibytes(256) },
            },
            // The property names are 'readiness' and 'liveness', NOT 'readinessProbe'
            readiness: kplus.Probe.fromHttpGet("/health", {
                port: 80,
                initialDelaySeconds: Duration.seconds(5),
                periodSeconds: Duration.seconds(10),
            }),
            liveness: kplus.Probe.fromHttpGet("/health", {
                port: 80,
                initialDelaySeconds: Duration.seconds(15),
                periodSeconds: Duration.seconds(20),
            }),
        });

        deployment.exposeViaService({
            name: "lornu-v2-preview-svc",
            serviceType: kplus.ServiceType.CLUSTER_IP,
            ports: [{ port: 80, targetPort: 80 }],
        });

        // Ingress implementation omitted for brevity as the issue is about probes
    }
}
