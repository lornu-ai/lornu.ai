/**
 * CDK8s Constructs for Lornu AI Infrastructure
 *
 * These constructs extend the base infrastructure with:
 * - AgentMemory XRD: Database/storage provisioning for agents
 * - AgentWorker XRD: Compute resource provisioning for agents
 * - Compositions: GCP implementations of the XRDs
 * - Claims: Example claims for testing and demonstration
 */

export { AgentXRDs, AgentCompositions } from "./agent-xrds";
export {
  AgentMemoryClaim,
  AgentWorkerClaim,
  ExampleClaims,
} from "./agent-claims";
