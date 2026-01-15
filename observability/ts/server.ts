import { serve } from "bun";

// Simple in-memory metrics
const metrics = {
  requests: 0,
  healthz: 0,
  metrics: 0,
};

function log(message: string) {
  // Simple log to stdout, can be replaced with more advanced logger
  console.log(`[${new Date().toISOString()}] ${message}`);
}

serve({
  port: process.env.PORT ? Number(process.env.PORT) : 8081,
  fetch(req) {
    metrics.requests++;
    const url = new URL(req.url);
    if (url.pathname === "/healthz") {
      metrics.healthz++;
      log("Health check requested");
      return new Response("ok", { status: 200 });
    }
    if (url.pathname === "/metrics") {
      metrics.metrics++;
      log("Metrics endpoint requested");
      return new Response(
        `# HELP requests_total Total HTTP requests\nrequests_total ${metrics.requests}\nhealthz_total ${metrics.healthz}\nmetrics_total ${metrics.metrics}\n`,
        { status: 200, headers: { "Content-Type": "text/plain" } }
      );
    }
    log(`404 Not Found: ${url.pathname}`);
    return new Response("Not found", { status: 404 });
  },
});

log("Observability service started on :8081");
