import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..", "..");

test("package.json wires sdkwork-specs verification into verify script", () => {
  const packageJson = JSON.parse(readFileSync(join(repoRoot, "package.json"), "utf8"));
  assert.match(packageJson.scripts.verify, /check:pagination/);
  assert.match(packageJson.scripts.verify, /check:api-envelope/);
});

test("standalone gateway start script uses the declared binary name", () => {
  const packageJson = JSON.parse(readFileSync(join(repoRoot, "package.json"), "utf8"));
  assert.match(
    packageJson.scripts.start,
    /--bin sdkwork-account-standalone-gateway/,
  );
});

test("app openapi exposes unified wallet ledger list route", () => {
  const openapi = JSON.parse(
    readFileSync(
      join(repoRoot, "apis/app-api/account/account-app-api.openapi.json"),
      "utf8",
    ),
  );
  assert.ok(openapi.paths["/app/v3/api/wallet/ledger_entries"]);
});

test("account summary openapi uses explicit points fields", () => {
  const openapi = JSON.parse(
    readFileSync(
      join(repoRoot, "apis/app-api/account/account-app-api.openapi.json"),
      "utf8",
    ),
  );
  const summaryProperties =
    openapi.components.schemas.AccountSummaryItem.properties;

  assert.ok(summaryProperties.availablePoints);
  assert.ok(summaryProperties.monthlyPointsConsumed);
  assert.equal(summaryProperties.availableCredits, undefined);
  assert.equal(summaryProperties.monthlyConsumption, undefined);
  assert.equal(summaryProperties.availablePoints.type, "string");
  assert.equal(summaryProperties.monthlyPointsConsumed.type, "string");
});

test("generated SDK business methods expose typed response models", () => {
  const generatedApiRoots = [
    join(
      repoRoot,
      "sdks/sdkwork-account-app-sdk/sdkwork-account-app-sdk-typescript/generated/server-openapi/src/api",
    ),
    join(
      repoRoot,
      "sdks/sdkwork-account-backend-sdk/sdkwork-account-backend-sdk-typescript/generated/server-openapi/src/api",
    ),
  ];
  const weakMethodReturns = [];

  for (const apiRoot of generatedApiRoots) {
    for (const entry of readdirSync(apiRoot, { withFileTypes: true })) {
      if (!entry.isFile() || !entry.name.endsWith(".ts")) {
        continue;
      }
      const relativePath = apiRoot
        .slice(repoRoot.length + 1)
        .replaceAll("\\", "/");
      const source = readFileSync(join(apiRoot, entry.name), "utf8");
      const matches = source.matchAll(
        /async\s+[\w$]+\([^)]*\):\s*Promise<Record<string,\s*unknown>>/g,
      );

      for (const match of matches) {
        weakMethodReturns.push(`${relativePath}/${entry.name}:${match[0]}`);
      }
    }
  }

  assert.deepEqual(weakMethodReturns, []);
});

test("generated SDK business response data fields are typed", () => {
  const generatedTypeRoots = [
    join(
      repoRoot,
      "sdks/sdkwork-account-app-sdk/sdkwork-account-app-sdk-typescript/generated/server-openapi/src/types",
    ),
    join(
      repoRoot,
      "sdks/sdkwork-account-backend-sdk/sdkwork-account-backend-sdk-typescript/generated/server-openapi/src/types",
    ),
  ];
  const weakResponseDataFields = [];
  const weakSummaryFields = [];

  for (const typeRoot of generatedTypeRoots) {
    for (const entry of readdirSync(typeRoot, { withFileTypes: true })) {
      if (!entry.isFile() || !entry.name.endsWith(".ts")) {
        continue;
      }
      const fullPath = join(typeRoot, entry.name);
      const relativePath = fullPath.slice(repoRoot.length + 1).replaceAll("\\", "/");
      const source = readFileSync(fullPath, "utf8");

      if (
        entry.name.endsWith("-response.ts") &&
        entry.name !== "sdk-work-api-response.ts" &&
        source.includes("data: Record<string, unknown>;")
      ) {
        weakResponseDataFields.push(relativePath);
      }

      if (
        entry.name === "account-summary-item.ts" &&
        /:\s*Record<string,\s*unknown>(?:\[\])?;/.test(source)
      ) {
        weakSummaryFields.push(relativePath);
      }
    }
  }

  assert.deepEqual(weakResponseDataFields, []);
  assert.deepEqual(weakSummaryFields, []);
});
