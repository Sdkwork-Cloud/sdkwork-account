#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

const pairs = [
  [
    "apis/app-api/account/account-app-api.openapi.json",
    "sdks/sdkwork-account-app-sdk/openapi/account-app-api.openapi.json",
  ],
  [
    "apis/backend-api/account/account-backend-api.openapi.json",
    "sdks/sdkwork-account-backend-sdk/openapi/account-backend-api.openapi.json",
  ],
];

for (const [source, target] of pairs) {
  fs.copyFileSync(path.join(repoRoot, source), path.join(repoRoot, target));
  console.log(`synced ${target}`);
}
