//! Account app-api gateway route manifest (materialized from the authored
//! OpenAPI contract; all operations use dual-token auth).

use sdkwork_web_core::{HttpMethod, HttpRoute, HttpRouteManifest};

const HTTP_ROUTES: &[HttpRoute] = &[
    HttpRoute::dual_token(HttpMethod::Get, "/app/v3/api/wallet/overview", "account", "wallet.overview.retrieve"),
    HttpRoute::dual_token(HttpMethod::Get, "/app/v3/api/wallet/accounts", "account", "wallet.accounts.list"),
    HttpRoute::dual_token(HttpMethod::Get, "/app/v3/api/wallet/accounts/cash", "account", "wallet.accounts.cash.retrieve"),
    HttpRoute::dual_token(HttpMethod::Get, "/app/v3/api/wallet/accounts/points", "account", "wallet.accounts.points.retrieve"),
    HttpRoute::dual_token(HttpMethod::Get, "/app/v3/api/wallet/ledger_entries", "account", "wallet.ledgerEntries.list"),
    HttpRoute::dual_token(HttpMethod::Get, "/app/v3/api/wallet/ledger_entries/cash", "account", "wallet.ledgerEntries.cash.list"),
    HttpRoute::dual_token(HttpMethod::Get, "/app/v3/api/wallet/ledger_entries/points", "account", "wallet.ledgerEntries.points.list"),
    HttpRoute::dual_token(HttpMethod::Get, "/app/v3/api/wallet/ledger_entries/{ledgerEntryId}", "account", "wallet.ledgerEntries.retrieve"),
    HttpRoute::dual_token(HttpMethod::Get, "/app/v3/api/wallet/ledger_entries/{ledgerEntryId}/allocations", "account", "wallet.ledgerEntries.allocations.list"),
    HttpRoute::dual_token(HttpMethod::Get, "/app/v3/api/wallet/points/summary", "account", "wallet.points.summary.retrieve"),
    HttpRoute::dual_token(HttpMethod::Get, "/app/v3/api/wallet/points/lots", "account", "wallet.points.lots.list"),
    HttpRoute::dual_token(HttpMethod::Get, "/app/v3/api/billing/history", "account", "billing.history.list"),
    HttpRoute::dual_token(HttpMethod::Get, "/app/v3/api/wallet/holds", "account", "wallet.holds.list"),
    HttpRoute::dual_token(HttpMethod::Get, "/app/v3/api/wallet/holds/{holdId}", "account", "wallet.holds.retrieve"),
    HttpRoute::dual_token(HttpMethod::Get, "/app/v3/api/accounts/current/summary", "account", "accounts.current.summary.retrieve"),
    HttpRoute::dual_token(HttpMethod::Get, "/app/v3/api/token_bank/account", "account", "tokenBank.account.retrieve"),
    HttpRoute::dual_token(HttpMethod::Get, "/app/v3/api/token_bank/overview", "account", "tokenBank.overview.retrieve"),
    HttpRoute::dual_token(HttpMethod::Get, "/app/v3/api/token_bank/ledger_entries", "account", "tokenBank.ledgerEntries.list"),
    HttpRoute::dual_token(HttpMethod::Get, "/app/v3/api/token_bank/holds", "account", "tokenBank.holds.list"),
    HttpRoute::dual_token(HttpMethod::Get, "/app/v3/api/wallet/portfolio", "account", "retrieveWalletPortfolio"),
];

pub fn gateway_route_manifest() -> HttpRouteManifest {
    HttpRouteManifest::new(HTTP_ROUTES)
}
