import { parseArgs } from "util";

/**
 * Agent Deployment Script
 *
 * Deploys an agent to a Knative sandbox.
 * Usage: bun infra/agent-deploy.ts --name <name> --sandbox <sandbox> [--image <image>]
 */

const args = parseArgs({
  options: {
    name: { type: "string" },
    sandbox: { type: "string" },
    image: { type: "string" },
  },
});

const name = args.values.name;
const sandbox = args.values.sandbox;
const image = args.values.image || `gcr.io/lornu-v2/agent-${name}:latest`;

if (!name || !sandbox) {
  console.error("❌ Error: --name and --sandbox are required");
  process.exit(1);
}

console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
console.log(`🚀 Deploying Agent: ${name}`);
console.log(`📦 Sandbox: ${sandbox}`);
console.log(`🖼️  Image: ${image}`);
console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

// In a real environment, this would call 'kn service apply' or 'kubectl apply'
// For now, we generate the Knative Service manifest

const knativeService = {
  apiVersion: "serving.knative.dev/v1",
  kind: "Service",
  metadata: {
    name,
    namespace: sandbox,
    labels: {
      "lornu.ai/managed-by": "dagger",
      "lornu.ai/agent-name": name,
    },
  },
  spec: {
    template: {
      spec: {
        containers: [
          {
            image,
            ports: [{ containerPort: 8080 }],
            env: [
              { name: "LORNU_ENV", value: "dev" },
              { name: "AGENT_NAME", value: name },
            ],
          },
        ],
      },
    },
  },
};

console.log("📋 Knative Service Manifest:");
console.log(JSON.stringify(knativeService, null, 2));

console.log("\n✅ Deployment manifest generated successfully!");
console.log("ℹ️  In CI, this would be applied to the cluster using 'kn' or 'kubectl'");
