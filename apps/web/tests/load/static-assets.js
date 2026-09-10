import http from "k6/http";
import { check, sleep } from "k6";

const baseUrl = __ENV.BASE_URL || "http://127.0.0.1:4173";

export const options = {
  scenarios: {
    static_asset_smoke: {
      executor: "ramping-vus",
      stages: [
        { duration: "2s", target: 5 },
        { duration: "5s", target: 5 },
        { duration: "2s", target: 0 },
      ],
      gracefulRampDown: "2s",
    },
  },
  thresholds: {
    checks: ["rate>0.99"],
    http_req_failed: ["rate<0.01"],
    http_req_duration: ["p(95)<1000"],
  },
};

export function setup() {
  const index = http.get(`${baseUrl}/`);
  const assetPaths = [
    ...index.body.matchAll(/(?:src|href)="([^"]+\.(?:js|css))"/g),
  ].map((match) => match[1]);
  const scriptPath = assetPaths.find((path) => path.endsWith(".js"));
  if (scriptPath) {
    const script = http.get(`${baseUrl}${scriptPath}`);
    const wasmPath = script.body.match(/(\/assets\/[A-Za-z0-9_-]+\.wasm)/)?.[1];
    if (wasmPath) assetPaths.push(wasmPath);
  }
  return {
    urls: [`${baseUrl}/`, ...assetPaths.map((path) => `${baseUrl}${path}`)],
  };
}

export default function ({ urls }) {
  const responses = http.batch(urls.map((url) => ["GET", url]));
  check(responses, {
    "all static assets respond with 200": (results) =>
      results.every((result) => result.status === 200),
    "index contains app mount": (results) =>
      results[0].body.includes('id="root"'),
  });
  sleep(0.1);
}
