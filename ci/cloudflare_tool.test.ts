import { expect, test, mock, spyOn } from "bun:test";

// 1. Mock the Tool Dispatcher
// This simulates the interface between your TS logic and the Rust tool
const toolDispatcher = {
  execute: async (toolName: string, params: any) => {
    // In real life, this would hit the Rust Engine via RPC/API
    return { status: "success", tool: toolName };
  },
};

test("Agent can request a DNS update with correct parameters", async () => {
  // 2. Setup a spy to watch the dispatcher
  const executeSpy = spyOn(toolDispatcher, "execute");

  // 3. Simulate the Agent's intent
  const agentIntent = {
    action: "upsert_dns",
    params: {
      zone_id: "lornu-zone-abc",
      name: "test-agent.lornu.ai",
      ip: "127.0.0.1",
    },
  };

  // 4. Run the logic
  await toolDispatcher.execute(agentIntent.action, agentIntent.params);

  // 5. Assertions
  expect(executeSpy).toHaveBeenCalled();
  expect(executeSpy).toHaveBeenCalledWith("upsert_dns", {
    zone_id: "lornu-zone-abc",
    name: "test-agent.lornu.ai",
    ip: "127.0.0.1",
  });
});