-- sdkwork:migration
-- id: 0001_account_tables
-- engine: postgres
-- module: account
-- purpose: Account wallet, ledger, and idempotency tables owned by sdkwork-account
-- reversible: true
-- transactional: true

CREATE TABLE IF NOT EXISTS commerce_idempotency_key (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    scope TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    request_hash TEXT NOT NULL,
    response_json TEXT,
    status TEXT NOT NULL,
    locked_until TEXT,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (tenant_id, scope, idempotency_key)
);

CREATE TABLE IF NOT EXISTS commerce_account (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    owner_user_id TEXT NOT NULL,
    asset_type TEXT NOT NULL,
    currency_code TEXT,
    available_amount TEXT NOT NULL DEFAULT '0',
    frozen_amount TEXT NOT NULL DEFAULT '0',
    version INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (tenant_id, organization_id, owner_user_id, asset_type, currency_code)
);

CREATE TABLE IF NOT EXISTS commerce_account_ledger_entry (
    id TEXT PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    organization_id TEXT,
    account_id TEXT NOT NULL,
    owner_user_id TEXT NOT NULL,
    asset_type TEXT NOT NULL,
    direction TEXT NOT NULL,
    amount TEXT NOT NULL,
    balance_after TEXT NOT NULL,
    business_type TEXT NOT NULL,
    transaction_no TEXT NOT NULL,
    request_no TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    source_type TEXT,
    source_id TEXT,
    remark TEXT,
    created_at TEXT NOT NULL,
    UNIQUE (tenant_id, transaction_no)
);

CREATE INDEX IF NOT EXISTS idx_commerce_account_owner_asset
    ON commerce_account (tenant_id, owner_user_id, asset_type, currency_code);

CREATE INDEX IF NOT EXISTS idx_commerce_account_ledger_account_created_at
    ON commerce_account_ledger_entry (tenant_id, account_id, created_at);

CREATE INDEX IF NOT EXISTS idx_commerce_account_ledger_request_no
    ON commerce_account_ledger_entry (tenant_id, request_no);

CREATE INDEX IF NOT EXISTS idx_commerce_account_ledger_idempotency_key
    ON commerce_account_ledger_entry (tenant_id, idempotency_key);
