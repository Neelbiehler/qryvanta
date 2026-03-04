import http from "k6/http";
import { check, fail } from "k6";

const BENCHMARK_PROFILE = (__ENV.BENCHMARK_PROFILE || "mixed").toLowerCase();
const BASE_URL = __ENV.QRYVANTA_API_BASE_URL || "http://127.0.0.1:3001";
const FRONTEND_URL = __ENV.QRYVANTA_FRONTEND_URL || "http://localhost:3000";
const LOGIN_EMAIL = __ENV.QRYVANTA_BENCH_EMAIL || "admin@qryvanta.local";
const LOGIN_PASSWORD = __ENV.QRYVANTA_BENCH_PASSWORD || "admin";
const BOOTSTRAP_TOKEN = __ENV.QRYVANTA_BENCH_BOOTSTRAP_TOKEN;
const BOOTSTRAP_SUBJECT =
  __ENV.QRYVANTA_BENCH_BOOTSTRAP_SUBJECT || "perf.benchmark@qryvanta.local";
const DURATION = __ENV.BENCHMARK_DURATION || "90s";
const WORKSPACE_RATE = Number(__ENV.BENCHMARK_WORKSPACE_RATE || "15");
const RUNTIME_QUERY_RATE = Number(__ENV.BENCHMARK_RUNTIME_QUERY_RATE || "20");
const WORKFLOW_EXEC_RATE = Number(__ENV.BENCHMARK_WORKFLOW_EXEC_RATE || "5");
const PREALLOCATED_VUS = Number(__ENV.BENCHMARK_PREALLOCATED_VUS || "20");
const MAX_VUS = Number(__ENV.BENCHMARK_MAX_VUS || "80");
const SUMMARY_PATH = __ENV.BENCHMARK_SUMMARY_PATH;

const mutatingHeaders = {
  "Content-Type": "application/json",
  Origin: FRONTEND_URL,
  Referer: `${FRONTEND_URL}/`,
};

export const options = {
  discardResponseBodies: true,
  scenarios: buildScenarios(BENCHMARK_PROFILE),
  thresholds: {
    http_req_failed: ["rate<0.01"],
    checks: ["rate>0.99"],
    "http_req_duration{scenario:workspace_read}": ["p(95)<350"],
    "http_req_duration{scenario:runtime_query}": ["p(95)<500"],
    "http_req_duration{scenario:workflow_execute}": ["p(95)<700"],
  },
};

function buildScenarios(profile) {
  const workspaceRead = {
    executor: "constant-arrival-rate",
    exec: "workspaceReadScenario",
    rate: WORKSPACE_RATE,
    timeUnit: "1s",
    duration: DURATION,
    preAllocatedVUs: PREALLOCATED_VUS,
    maxVUs: MAX_VUS,
  };
  const runtimeQuery = {
    executor: "constant-arrival-rate",
    exec: "runtimeQueryScenario",
    rate: RUNTIME_QUERY_RATE,
    timeUnit: "1s",
    duration: DURATION,
    preAllocatedVUs: PREALLOCATED_VUS,
    maxVUs: MAX_VUS,
  };
  const workflowExecute = {
    executor: "constant-arrival-rate",
    exec: "workflowExecuteScenario",
    rate: WORKFLOW_EXEC_RATE,
    timeUnit: "1s",
    duration: DURATION,
    preAllocatedVUs: PREALLOCATED_VUS,
    maxVUs: MAX_VUS,
  };

  switch (profile) {
    case "workspace":
      return { workspace_read: workspaceRead };
    case "runtime":
      return { runtime_query: runtimeQuery };
    case "workflow":
      return { workflow_execute: workflowExecute };
    case "mixed":
      return {
        workspace_read: workspaceRead,
        runtime_query: runtimeQuery,
        workflow_execute: workflowExecute,
      };
    default:
      fail(
        `unsupported BENCHMARK_PROFILE '${profile}'. Expected workspace|runtime|workflow|mixed.`,
      );
  }
}

export function setup() {
  if (BOOTSTRAP_TOKEN) {
    return authenticateWithBootstrap();
  }

  return authenticateWithLogin();
}

function authenticateWithBootstrap() {
  const bootstrapResponse = http.post(
    `${BASE_URL}/auth/bootstrap`,
    JSON.stringify({
      subject: BOOTSTRAP_SUBJECT,
      token: BOOTSTRAP_TOKEN,
    }),
    {
      headers: mutatingHeaders,
      tags: { endpoint: "auth_bootstrap" },
    },
  );

  const validBootstrap = check(bootstrapResponse, {
    "bootstrap auth returns 204": (response) => response.status === 204,
  });

  if (!validBootstrap) {
    fail(
      `benchmark bootstrap auth failed (status=${bootstrapResponse.status}, body=${bootstrapResponse.body})`,
    );
  }

  return { cookie_header: extractCookieHeader(bootstrapResponse) };
}

function authenticateWithLogin() {
  const loginResponse = http.post(
    `${BASE_URL}/auth/login`,
    JSON.stringify({
      email: LOGIN_EMAIL,
      password: LOGIN_PASSWORD,
    }),
    {
      headers: mutatingHeaders,
      tags: { endpoint: "auth_login" },
    },
  );

  const validLogin = check(loginResponse, {
    "login returns 200": (response) => response.status === 200,
  });

  if (!validLogin) {
    fail(
      `benchmark authentication failed (status=${loginResponse.status}, body=${loginResponse.body})`,
    );
  }

  const cookieHeader = extractCookieHeader(loginResponse);
  return { cookie_header: cookieHeader };
}

function extractCookieHeader(response) {
  const cookieParts = [];
  for (const [name, values] of Object.entries(response.cookies || {})) {
    if (Array.isArray(values) && values.length > 0) {
      cookieParts.push(`${name}=${values[0].value}`);
    }
  }

  if (cookieParts.length === 0) {
    fail(
      `benchmark authentication succeeded without session cookies (status=${response.status})`,
    );
  }

  return cookieParts.join("; ");
}

function withAuthHeaders(benchmarkSetup, includeMutatingHeaders) {
  if (!benchmarkSetup || !benchmarkSetup.cookie_header) {
    fail("missing benchmark setup session cookie");
  }

  if (includeMutatingHeaders) {
    return {
      ...mutatingHeaders,
      Cookie: benchmarkSetup.cookie_header,
    };
  }

  return {
    Cookie: benchmarkSetup.cookie_header,
  };
}

export function workspaceReadScenario(benchmarkSetup) {

  const response = http.get(`${BASE_URL}/api/v1/workspace/apps`, {
    headers: withAuthHeaders(benchmarkSetup, false),
    tags: { endpoint: "workspace_apps_list" },
  });

  check(response, {
    "workspace apps list returns 200": (result) => result.status === 200,
  });
}

export function runtimeQueryScenario(benchmarkSetup) {

  const response = http.post(
    `${BASE_URL}/api/v1/workspace/apps/sales_hub/entities/deal/records/query`,
    JSON.stringify({
      limit: 25,
      offset: 0,
    }),
    {
      headers: withAuthHeaders(benchmarkSetup, true),
      tags: { endpoint: "workspace_deal_query" },
    },
  );

  check(response, {
    "runtime query returns 200": (result) => result.status === 200,
  });
}

export function workflowExecuteScenario(benchmarkSetup) {

  const response = http.post(
    `${BASE_URL}/api/v1/workflows/deal_created_notify/execute`,
    JSON.stringify({
      trigger_payload: {
        source: "perf-benchmark-suite",
        synthetic: true,
      },
    }),
    {
      headers: withAuthHeaders(benchmarkSetup, true),
      tags: { endpoint: "workflow_execute_deal_created_notify" },
    },
  );

  check(response, {
    "workflow execute returns 200": (result) => result.status === 200,
  });
}

function metricValue(metrics, metricName, valueKey) {
  return metrics[metricName] && metrics[metricName].values
    ? metrics[metricName].values[valueKey]
    : null;
}

export function handleSummary(data) {
  const summary = {
    profile: BENCHMARK_PROFILE,
    base_url: BASE_URL,
    duration: DURATION,
    vus: {
      pre_allocated: PREALLOCATED_VUS,
      max: MAX_VUS,
    },
    rates_per_second: {
      workspace: WORKSPACE_RATE,
      runtime_query: RUNTIME_QUERY_RATE,
      workflow_execute: WORKFLOW_EXEC_RATE,
    },
    metrics: {
      iterations: metricValue(data.metrics, "iterations", "count"),
      http_requests: metricValue(data.metrics, "http_reqs", "count"),
      http_req_failed_rate: metricValue(data.metrics, "http_req_failed", "rate"),
      http_req_duration_avg_ms: metricValue(data.metrics, "http_req_duration", "avg"),
      http_req_duration_p95_ms: metricValue(data.metrics, "http_req_duration", "p(95)"),
      checks_pass_rate: metricValue(data.metrics, "checks", "rate"),
      data_received_bytes: metricValue(data.metrics, "data_received", "count"),
      data_sent_bytes: metricValue(data.metrics, "data_sent", "count"),
    },
  };

  const encodedSummary = `${JSON.stringify(summary, null, 2)}\n`;
  if (SUMMARY_PATH) {
    return {
      stdout: encodedSummary,
      [SUMMARY_PATH]: encodedSummary,
    };
  }

  return { stdout: encodedSummary };
}
