//! Account backend-api gateway route manifest (materialized from the authored
//! OpenAPI contract; all operations use dual-token auth).

use sdkwork_web_core::{HttpMethod, HttpRoute, HttpRouteManifest};

const HTTP_ROUTES: &[HttpRoute] = &[
    HttpRoute::dual_token(HttpMethod::Get, "/backend/v3/api/wallet/health", "account", "wallet.health.retrieve"),
    HttpRoute::dual_token(HttpMethod::Post, "/backend/v3/api/wallet/outbox/dispatch", "account", "wallet.outbox.dispatch"),
    HttpRoute::dual_token(HttpMethod::Post, "/backend/v3/api/wallet/adjustments", "account", "wallet.adjustments.create"),
    HttpRoute::dual_token(HttpMethod::Post, "/backend/v3/api/wallet/adjustments/cash", "account", "wallet.adjustments.cash.create"),
    HttpRoute::dual_token(HttpMethod::Post, "/backend/v3/api/wallet/adjustments/points", "account", "wallet.adjustments.points.create"),
    HttpRoute::dual_token(HttpMethod::Post, "/backend/v3/api/wallet/points/lots/expire", "account", "wallet.points.lots.expire"),
    HttpRoute::dual_token(HttpMethod::Post, "/backend/v3/api/wallet/points/reconciliation", "account", "wallet.points.reconciliation"),
    HttpRoute::dual_token(HttpMethod::Post, "/backend/v3/api/wallet/holds", "account", "wallet.holds.create"),
    HttpRoute::dual_token(HttpMethod::Post, "/backend/v3/api/wallet/holds/{holdId}/settle", "account", "wallet.holds.settle"),
    HttpRoute::dual_token(HttpMethod::Post, "/backend/v3/api/wallet/holds/{holdId}/release", "account", "wallet.holds.release"),
    HttpRoute::dual_token(HttpMethod::Post, "/backend/v3/api/wallet/holds/expire", "account", "wallet.holds.expire"),
    HttpRoute::dual_token(HttpMethod::Post, "/backend/v3/api/wallet/transfers", "account", "wallet.transfers.create"),
    HttpRoute::dual_token(HttpMethod::Post, "/backend/v3/api/token_bank/credits", "account", "tokenBank.credits.create"),
    HttpRoute::dual_token(HttpMethod::Post, "/backend/v3/api/token_bank/debits", "account", "tokenBank.debits.create"),
    HttpRoute::dual_token(HttpMethod::Post, "/backend/v3/api/token_bank/grants", "account", "tokenBank.grants.create"),
    HttpRoute::dual_token(HttpMethod::Post, "/backend/v3/api/token_bank/reversals", "account", "tokenBank.reversals.create"),
    HttpRoute::dual_token(HttpMethod::Post, "/backend/v3/api/token_bank/holds", "account", "tokenBank.holds.create"),
    HttpRoute::dual_token(HttpMethod::Post, "/backend/v3/api/token_bank/holds/{holdId}/settle", "account", "tokenBank.holds.settle"),
    HttpRoute::dual_token(HttpMethod::Post, "/backend/v3/api/token_bank/holds/{holdId}/release", "account", "tokenBank.holds.release"),
    HttpRoute::dual_token(HttpMethod::Post, "/backend/v3/api/token_bank/transfers", "account", "tokenBank.transfers.create"),
];

pub fn gateway_route_manifest() -> HttpRouteManifest {
    HttpRouteManifest::new(HTTP_ROUTES)
}
