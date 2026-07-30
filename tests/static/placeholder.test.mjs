import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..", "..");

const accountOwnedTableNames = [
  "acct_account",
  "acct_ledger_entry",
  "acct_journal",
  "acct_journal_line",
  "acct_hold",
  "acct_transfer",
  "acct_points_lot",
  "acct_points_lot_allocation",
  "acct_token_bank_exchange_rate",
  "acct_token_bank_exchange_quote",
  "acct_token_bank_exchange_snapshot",
  "acct_token_bank_settlement_snapshot",
  "acct_idempotency_record",
  "acct_outbox_event",
  "acct_billing_history",
];

const retiredAccountOwnedTableNames = [
  "commerce_account",
  "commerce_account_ledger",
  "commerce_account_journal",
  "commerce_account_journal_line",
  "commerce_account_hold",
  "commerce_account_transfer",
  "commerce_points_lot",
  "commerce_points_lot_allocation",
  "commerce_token_bank_exchange_rate",
  "commerce_token_bank_exchange_quote",
  "commerce_token_bank_exchange_snapshot",
  "commerce_token_bank_settlement_snapshot",
  "commerce_idempotency_record",
  "commerce_outbox_event",
  "commerce_billing_history",
];

test("package.json wires sdkwork-specs verification into verify script", () => {
  const packageJson = JSON.parse(readFileSync(join(repoRoot, "package.json"), "utf8"));
  assert.match(packageJson.scripts.verify, /check:pagination/);
  assert.match(packageJson.scripts.verify, /check:api-envelope/);
});

test("standalone gateway start script uses the declared binary name", () => {
  const packageJson = JSON.parse(readFileSync(join(repoRoot, "package.json"), "utf8"));
  assert.match(
    packageJson.scripts.start,
    /--bin sdkwork-api-account-standalone-gateway/,
  );
});

test("database materialization script uses acct prefix", () => {
  const packageJson = JSON.parse(readFileSync(join(repoRoot, "package.json"), "utf8"));
  const script = packageJson.scripts["db:materialize:contract"];

  assert.match(script, /--prefixes acct_/);
  assert.doesNotMatch(script, /--prefixes commerce_/);
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

test("database contract registers acct prefix and account-owned tables", () => {
  const manifest = JSON.parse(
    readFileSync(join(repoRoot, "database/database.manifest.json"), "utf8"),
  );
  const prefixRegistry = JSON.parse(
    readFileSync(join(repoRoot, "database/contract/prefix-registry.json"), "utf8"),
  );
  const tableRegistry = JSON.parse(
    readFileSync(join(repoRoot, "database/contract/table-registry.json"), "utf8"),
  );
  const schema = readFileSync(
    join(repoRoot, "database/contract/schema.yaml"),
    "utf8",
  );

  assert.equal(manifest.tablePrefix, "acct_");
  assert.deepEqual(prefixRegistry.prefixes, ["acct_"]);
  assert.deepEqual(tableRegistry.tables, accountOwnedTableNames);
  assert.match(schema, /^table_prefix: acct_$/m);
  for (const tableName of accountOwnedTableNames) {
    assert.match(schema, new RegExp(`name: ${tableName}\\b`));
  }
  for (const tableName of retiredAccountOwnedTableNames) {
    assert.doesNotMatch(schema, new RegExp(`\\b${tableName}\\b`));
  }
});

test("commerce boundary spec separates acct physical tables from commerce order ownership", () => {
  const source = readFileSync(
    join(repoRoot, "specs/COMMERCE_BOUNDARY_SPEC.md"),
    "utf8",
  );

  assert.match(source, /Account-owned physical tables must use `acct_`/);
  assert.match(source, /capability remains `commerce\.account`/);
  assert.match(source, /`commerce_order` remains order-owned/);
  assert.match(source, /Account must never read `commerce_order`/);
});

test("commerce integration spec exposes account storage boundary", () => {
  const integrationSpec = JSON.parse(
    readFileSync(join(repoRoot, "specs/commerce-integration.spec.json"), "utf8"),
  );

  assert.equal(integrationSpec.database.ownedTablePrefix, "acct_");
  assert.equal(integrationSpec.database.capabilityIdentity, "commerce.account");
  assert.deepEqual(integrationSpec.database.externalOwnedTables, ["commerce_order"]);
  assert.equal(integrationSpec.database.directSqlToExternalTablesAllowed, false);
  assert.deepEqual(new Set(integrationSpec.ownedTables), new Set(accountOwnedTableNames));
  assert.ok(integrationSpec.ownedTables.every((tableName) => tableName.startsWith("acct_")));
});

test("account boundary assigns value-order orchestration to order and channel execution to payment", () => {
  const boundarySpec = readFileSync(
    join(repoRoot, "specs/COMMERCE_BOUNDARY_SPEC.md"),
    "utf8",
  );
  const integrationSpec = JSON.parse(
    readFileSync(join(repoRoot, "specs/commerce-integration.spec.json"), "utf8"),
  );

  assert.match(
    boundarySpec,
    /Recharge, coupon redemption, refund, and withdrawal orchestration belong to `sdkwork-order`/,
  );
  assert.match(
    boundarySpec,
    /`sdkwork-order`\s+-> `sdkwork-payment`/,
  );
  assert.match(
    boundarySpec,
    /`sdkwork-payment` may only reference `commerce_order` for read-only validation/,
  );
  assert.doesNotMatch(
    boundarySpec,
    /sdkwork-payment\s+-> sdkwork-order\s+\(pay\/refund existing orderId\)/,
  );

  assert.deepEqual(integrationSpec.valueOrderOrchestration, {
    owner: "sdkwork-order",
    ledgerExecutor: "sdkwork-account",
    paymentExecutor: "sdkwork-payment",
    directPaymentToAccountDependencyAllowed: false,
    orderSubjects: [
      "points_recharge",
      "token_bank_recharge",
      "token_bank_plan_purchase",
      "token_bank_plan_renewal",
      "account_recharge_package",
      "coupon_recharge",
      "refund_request",
      "cash_withdrawal",
    ],
  });
});

test("account pc withdraw delegates to order withdrawal request flow", () => {
  const integrationSpec = JSON.parse(
    readFileSync(join(repoRoot, "specs/commerce-integration.spec.json"), "utf8"),
  );
  const shellSource = readFileSync(
    join(repoRoot, "apps/sdkwork-account-pc/packages/sdkwork-account-pc-shell/src/index.tsx"),
    "utf8",
  );
  const walletIndexSource = readFileSync(
    join(repoRoot, "apps/sdkwork-account-pc/packages/sdkwork-account-pc-wallet/src/index.ts"),
    "utf8",
  );
  const withdrawalNavigationSource = readFileSync(
    join(
      repoRoot,
      "apps/sdkwork-account-pc/packages/sdkwork-account-pc-wallet/src/wallet-withdrawal-navigation.ts",
    ),
    "utf8",
  );
  const integratorGuide = readFileSync(
    join(repoRoot, "docs/guides/integrator/README.md"),
    "utf8",
  );

  const walletSpec = integrationSpec.pcPackages["account-pc-wallet"];
  const withdrawalPort = walletSpec.requiredIntegratorPorts.find((entry) =>
    entry.methods.includes("withdrawals.requests.create")
  );
  assert.ok(withdrawalPort);
  assert.equal(withdrawalPort.capability, "sdkwork-order");
  assert.equal(withdrawalPort.defaultRoute, "/withdrawals/requests");
  assert.ok(walletSpec.forbiddenSdkUsage.includes("direct navigation to provider payout routes"));
  assert.ok(walletSpec.forbiddenSdkUsage.includes("withdrawals.* on account-service"));

  assert.match(shellSource, /VITE_SDKWORK_ORDER_WITHDRAWAL_REQUEST_BASE/);
  assert.doesNotMatch(shellSource, /VITE_SDKWORK_PAYMENT_PAYOUT_BASE|payments\/payout|payoutBasePath|payoutFlow/);
  assert.match(walletIndexSource, /wallet-withdrawal-navigation/);
  assert.doesNotMatch(walletIndexSource, /wallet-payout-navigation/);
  assert.match(withdrawalNavigationSource, /DEFAULT_WITHDRAWAL_REQUEST_BASE_PATH = "\/withdrawals\/requests"/);
  assert.doesNotMatch(withdrawalNavigationSource, /payments\/payout|payoutBasePath|payoutFlow/);
  assert.match(integratorGuide, /withdrawalRequestBasePath="\/withdrawals\/requests"/);
  assert.doesNotMatch(integratorGuide, /payments\/payout|payoutBasePath|payoutFlow|orders\.pay\b/);
});

test("database DDL and repository SQL use acct account-owned table names", () => {
  const baselineFiles = [
    "database/ddl/baseline/postgres/0001_account_baseline.sql",
    "tests/fixtures/database/sqlite/ddl/baseline/0001_account_baseline.sql",
  ];
  const repositorySchemaFiles = [
    "crates/sdkwork-account-repository-sqlx/test_migrations/0001_account_repository_test.sql",
  ];

  for (const relativeFile of baselineFiles) {
    const source = readFileSync(join(repoRoot, relativeFile), "utf8");
    for (const tableName of accountOwnedTableNames) {
      assert.match(source, new RegExp(`\\b${tableName}\\b`));
    }
    for (const tableName of retiredAccountOwnedTableNames) {
      assert.doesNotMatch(source, new RegExp(`\\b${tableName}\\b`));
    }
  }

  for (const relativeFile of repositorySchemaFiles) {
    const source = readFileSync(join(repoRoot, relativeFile), "utf8");
    for (const tableName of retiredAccountOwnedTableNames) {
      assert.doesNotMatch(source, new RegExp(`\\b${tableName}\\b`));
    }
  }
});

test("outbox aggregate type uses logical account aggregate", () => {
  const source = readFileSync(
    join(repoRoot, "crates/sdkwork-account-repository-sqlx/src/store/outbox.rs"),
    "utf8",
  );

  assert.match(
    source,
    /OUTBOX_AGGREGATE_TYPE_ACCOUNT:\s*&str\s*=\s*"account"/,
  );
  assert.doesNotMatch(source, /OUTBOX_AGGREGATE_TYPE_ACCOUNT:\s*&str\s*=\s*"acct_account"/);
  assert.doesNotMatch(source, /OUTBOX_AGGREGATE_TYPE_ACCOUNT:\s*&str\s*=\s*"commerce_account"/);
});

test("account openapi response schemas use named data contracts", () => {
  const apiFiles = [
    "apis/app-api/account/account-app-api.openapi.json",
    "apis/backend-api/account/account-backend-api.openapi.json",
  ];
  const weakResponses = [];
  const inlineSuccessResponses = [];

  for (const apiFile of apiFiles) {
    const openapi = JSON.parse(readFileSync(join(repoRoot, apiFile), "utf8"));
    for (const [schemaName, schema] of Object.entries(openapi.components.schemas)) {
      if (!schemaName.endsWith("Response") || schemaName === "SdkWorkApiResponse") {
        continue;
      }
      if (schema.allOf || !schema.properties?.data?.$ref) {
        weakResponses.push(`${apiFile}#/components/schemas/${schemaName}`);
      }
    }

    for (const [path, pathItem] of Object.entries(openapi.paths)) {
      for (const [method, operation] of Object.entries(pathItem)) {
        const responses = operation.responses ?? {};
        for (const [status, response] of Object.entries(responses)) {
          if (!/^2\d\d$/.test(status) || status === "204") {
            continue;
          }
          const schema = response.content?.["application/json"]?.schema;
          if (schema && !schema.$ref) {
            inlineSuccessResponses.push(`${method.toUpperCase()} ${path} ${status}`);
          }
        }
      }
    }
  }

  assert.deepEqual(weakResponses, []);
  assert.deepEqual(inlineSuccessResponses, []);
});

test("account openapi represents int64 values as strings", () => {
  const apiFiles = [
    "apis/app-api/account/account-app-api.openapi.json",
    "apis/backend-api/account/account-backend-api.openapi.json",
  ];
  const integerInt64Schemas = [];
  const invalidInt64StringSchemas = [];

  function visit(value, path) {
    if (!value || typeof value !== "object") {
      return;
    }
    if (value.type === "integer" && value.format === "int64") {
      integerInt64Schemas.push(path);
    }
    if (value.format === "int64") {
      if (
        value.type !== "string" ||
        value["x-sdkwork-int64-string"] !== true ||
        value["x-sdkwork-rust-type"] !== "i64" ||
        typeof value.pattern !== "string" ||
        !value.pattern.includes("[0-9]")
      ) {
        invalidInt64StringSchemas.push(path);
      }
    }
    if (Array.isArray(value)) {
      value.forEach((item, index) => visit(item, `${path}/${index}`));
      return;
    }
    for (const [key, child] of Object.entries(value)) {
      visit(child, `${path}/${key}`);
    }
  }

  for (const apiFile of apiFiles) {
    const openapi = JSON.parse(readFileSync(join(repoRoot, apiFile), "utf8"));
    visit(openapi, apiFile);
  }

  assert.deepEqual(integerInt64Schemas, []);
  assert.deepEqual(invalidInt64StringSchemas, []);
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
