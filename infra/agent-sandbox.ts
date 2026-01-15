#!/usr/bin/env bun
/**
 * Agent Sandbox Management
 * 
 * Creates temporary Kubernetes namespaces for agent experiments.
 * Uses Crossplane to provision isolated environments that are automatically
 * cleaned up when the branch is deleted.
 */

import { $ } from "bun";
import { parseArgs } from "util";

interface SandboxConfig {
  name: string;
  branch: string;
  agentName: string;
  namespace?: string;
}

const args = parseArgs({
  options: {
    name: {
      type: "string",
      description: "Sandbox name (derived from agent name)",
    },
    branch: {
      type: "string",
      description: "Git branch name (e.g., agent/researcher/exp-1)",
    },
    "agent-name": {
      type: "string",
      description: "Agent name (e.g., researcher)",
    },
    namespace: {
      type: "string",
      description: "Kubernetes namespace (defaults to sandbox name)",
    },
    action: {
      type: "string",
      description: "Action: create, delete, list",
      default: "create",
    },
    help: {
      type: "boolean",
      description: "Show help message",
      short: "h",
    },
  },
});

if (args.values.help) {
  console.log(`
Agent Sandbox Management

Usage:
  bun infra/agent-sandbox.ts [options]

Options:
  --name <name>          Sandbox name
  --branch <branch>      Git branch name
  --agent-name <name>    Agent name
  --namespace <ns>       Kubernetes namespace (defaults to sandbox name)
  --action <action>      Action: create, delete, list (default: create)
  --help, -h             Show this help message

Examples:
  # Create sandbox
  bun infra/agent-sandbox.ts --name researcher-exp-1 --branch agent/researcher/exp-1

  # Delete sandbox
  bun infra/agent-sandbox.ts --name researcher-exp-1 --action delete

  # List sandboxes
  bun infra/agent-sandbox.ts --action list
`);
  process.exit(0);
}

async function createSandbox(config: SandboxConfig): Promise<void> {
  const namespace = config.namespace || `agent-${config.name}`;
  
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("🏗️  Creating Agent Sandbox");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log(`   Name: ${config.name}`);
  console.log(`   Agent: ${config.agentName}`);
  console.log(`   Branch: ${config.branch}`);
  console.log(`   Namespace: ${namespace}`);
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

  // Create namespace via kubectl (or Crossplane Composition)
  console.log("\n📦 Creating namespace...");
  try {
    await $`kubectl create namespace ${namespace}`.quiet();
    console.log(`✅ Namespace created: ${namespace}`);
  } catch (error) {
    // Namespace might already exist
    console.log(`ℹ️  Namespace already exists: ${namespace}`);
  }

  // Create Crossplane Composition for agent sandbox
  // This would create: Knative service, ServiceAccount, ConfigMap, etc.
  const composition = {
    apiVersion: "apiextensions.crossplane.io/v1",
    kind: "Composition",
    metadata: {
      name: `agent-sandbox-${config.name}`,
      namespace: namespace,
    },
    spec: {
      resources: [
        {
          name: "namespace",
          base: {
            apiVersion: "v1",
            kind: "Namespace",
            metadata: {
              name: namespace,
              labels: {
                "lornu.ai/agent-sandbox": "true",
                "lornu.ai/branch": config.branch,
                "lornu.ai/agent": config.agentName,
              },
            },
          },
        },
        {
          name: "knative-service",
          base: {
            apiVersion: "serving.knative.dev/v1",
            kind: "Service",
            metadata: {
              name: config.agentName,
              namespace: namespace,
            },
            spec: {
              template: {
                metadata: {
                  labels: {
                    "lornu.ai/agent": config.agentName,
                    "lornu.ai/sandbox": config.name,
                  },
                },
                spec: {
                  containers: [
                    {
                      name: config.agentName,
                      image: `ghcr.io/lornu-ai/${config.agentName}:${config.branch.replace(/\//g, "-")}`,
                      env: [
                        {
                          name: "AGENT_NAME",
                          value: config.agentName,
                        },
                        {
                          name: "SANDBOX_NAME",
                          value: config.name,
                        },
                      ],
                    },
                  ],
                },
              },
            },
          },
        },
      ],
    },
  };

  // Apply composition (in real implementation, this would use Crossplane API)
  console.log("\n📝 Applying Crossplane composition...");
  console.log("   (In production, this would use Crossplane API client)");
  
  // For now, create a simple namespace with labels
  const namespaceManifest = {
    apiVersion: "v1",
    kind: "Namespace",
    metadata: {
      name: namespace,
      labels: {
        "lornu.ai/agent-sandbox": "true",
        "lornu.ai/branch": config.branch,
        "lornu.ai/agent": config.agentName,
        "lornu.ai/auto-cleanup": "true",
      },
      annotations: {
        "lornu.ai/branch": config.branch,
        "lornu.ai/created-at": new Date().toISOString(),
      },
    },
  };

  // Write manifest and apply
  const manifestPath = `/tmp/sandbox-${config.name}.yaml`;
  await Bun.write(manifestPath, JSON.stringify(namespaceManifest, null, 2));
  
  try {
    await $`kubectl apply -f ${manifestPath}`.quiet();
    console.log(`✅ Sandbox created: ${namespace}`);
  } catch (error) {
    console.error(`❌ Failed to create sandbox: ${error}`);
    throw error;
  }

  console.log("\n💡 Sandbox will be automatically deleted when branch is deleted");
  console.log(`   To manually delete: bun infra/agent-sandbox.ts --name ${config.name} --action delete`);
}

async function deleteSandbox(name: string): Promise<void> {
  const namespace = `agent-${name}`;
  
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("🗑️  Deleting Agent Sandbox");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log(`   Name: ${name}`);
  console.log(`   Namespace: ${namespace}`);
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

  try {
    await $`kubectl delete namespace ${namespace}`.quiet();
    console.log(`✅ Sandbox deleted: ${namespace}`);
  } catch (error) {
    console.error(`❌ Failed to delete sandbox: ${error}`);
    throw error;
  }
}

async function listSandboxes(): Promise<void> {
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("📋 Agent Sandboxes");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

  try {
    const output = await $`kubectl get namespaces -l lornu.ai/agent-sandbox=true -o json`.json();
    const namespaces = output.items || [];
    
    if (namespaces.length === 0) {
      console.log("   No sandboxes found");
      return;
    }

    for (const ns of namespaces) {
      const labels = ns.metadata?.labels || {};
      const annotations = ns.metadata?.annotations || {};
      console.log(`\n   Namespace: ${ns.metadata?.name}`);
      console.log(`   Agent: ${labels["lornu.ai/agent"] || "unknown"}`);
      console.log(`   Branch: ${labels["lornu.ai/branch"] || "unknown"}`);
      console.log(`   Created: ${annotations["lornu.ai/created-at"] || "unknown"}`);
    }
  } catch (error) {
    console.error(`❌ Failed to list sandboxes: ${error}`);
  }
}

async function main() {
  const { values } = args;
  const action = values.action || "create";

  if (action === "list") {
    await listSandboxes();
    return;
  }

  if (!values.name) {
    console.error("❌ Error: --name is required for create/delete actions");
    process.exit(1);
  }

  if (action === "create") {
    if (!values.branch) {
      console.error("❌ Error: --branch is required for create action");
      process.exit(1);
    }

    // Extract agent name from branch if not provided
    const agentName = values["agent-name"] || values.branch.split("/")[1] || "unknown";

    const config: SandboxConfig = {
      name: values.name,
      branch: values.branch,
      agentName: agentName,
      namespace: values.namespace,
    };

    await createSandbox(config);
  } else if (action === "delete") {
    await deleteSandbox(values.name);
  } else {
    console.error(`❌ Unknown action: ${action}`);
    process.exit(1);
  }
}

main().catch((error) => {
  console.error("❌ Error:", error.message);
  process.exit(1);
});
