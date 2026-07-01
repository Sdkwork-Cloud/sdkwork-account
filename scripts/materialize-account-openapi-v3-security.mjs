#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

const targets = [
  "apis/app-api/account/account-app-api.openapi.json",
  "apis/backend-api/account/account-backend-api.openapi.json",
  "sdks/sdkwork-account-app-sdk/openapi/account-app-api.openapi.json",
  "sdks/sdkwork-account-backend-sdk/openapi/account-backend-api.openapi.json",
];

const securitySchemes = {
  AuthToken: {
    type: "http",
    scheme: "bearer",
    bearerFormat: "JWT",
    description: "SDKWork auth token carried as Authorization: Bearer <auth_token>.",
  },
  AccessToken: {
    type: "apiKey",
    in: "header",
    name: "Access-Token",
    description: "SDKWork access isolation token.",
  },
};

const authenticatedSecurity = [{ AuthToken: [], AccessToken: [] }];
const publicSecurity = [];

function isPublicHealthOperation(method, rawPath) {
  return method === "get" && rawPath.endsWith("/wallet/health");
}

function patchSpec(spec) {
  spec.components ??= {};
  spec.components.securitySchemes = securitySchemes;
  spec.security = authenticatedSecurity;

  for (const [rawPath, pathItem] of Object.entries(spec.paths ?? {})) {
    for (const method of ["get", "post", "put", "patch", "delete"]) {
      const operation = pathItem?.[method];
      if (!operation || typeof operation !== "object") {
        continue;
      }
      operation.security = isPublicHealthOperation(method, rawPath)
        ? publicSecurity
        : authenticatedSecurity;
    }
  }

  return spec;
}

for (const relativePath of targets) {
  const absolutePath = path.join(repoRoot, relativePath);
  const spec = JSON.parse(fs.readFileSync(absolutePath, "utf8"));
  patchSpec(spec);
  fs.writeFileSync(absolutePath, `${JSON.stringify(spec, null, 2)}\n`, "utf8");
  console.log(`patched ${relativePath}`);
}
