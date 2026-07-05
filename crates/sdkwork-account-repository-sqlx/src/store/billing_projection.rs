use sdkwork_account_service::AppendLedgerEntryCommand;
use sdkwork_contract_service::{CommerceAccountAssetType, CommerceLedgerDirection, CommerceServiceError};
use sqlx::{Executor, Postgres, Sqlite};

use crate::store::{asset_code_from_type, next_entity_id, next_entity_uuid, store_error};

/// Writes a user-facing billing history row for a committed ledger append (SQLite).
pub async fn insert_billing_history_for_ledger_append<'e, E>(
    executor: E,
    tenant_id: i64,
    organization_id: i64,
    owner_id: i64,
    ledger_id: i64,
    command: &AppendLedgerEntryCommand,
    occurred_at: &str,
) -> Result<(), CommerceServiceError>
where
    E: Executor<'e, Database = Sqlite>,
{
    let points_delta = if command.asset_type == CommerceAccountAssetType::Points {
        let raw = command
            .amount
            .as_str()
            .parse::<i64>()
            .map_err(|_| CommerceServiceError::validation("points amount must be an integer"))?;
        match command.direction {
            CommerceLedgerDirection::Credit => raw,
            CommerceLedgerDirection::Debit => -raw,
        }
    } else {
        0
    };

    let direction = command.direction.as_str().to_owned();
    let title = billing_title_for_business_type(&command.business_type, &command.direction);
    let history_type = command.business_type.clone();
    let history_no = format!("ledger:{}", ledger_id);

    sqlx::query(
        r#"
        INSERT INTO commerce_billing_history
            (id, uuid, tenant_id, organization_id, owner_type, owner_id, history_no, history_type,
             direction, asset_code, amount, currency_code, points_delta, status, title,
             reference_no, source_type, source_id, occurred_at, created_at)
        VALUES (?, ?, ?, ?, 'USER', ?, ?, ?, ?, ?, ?, ?, ?, 1, ?, ?, 'ledger', ?, ?, ?)
        "#,
    )
    .bind(next_entity_id()?)
    .bind(next_entity_uuid())
    .bind(tenant_id)
    .bind(organization_id)
    .bind(owner_id)
    .bind(&history_no)
    .bind(&history_type)
    .bind(&direction)
    .bind(asset_code_from_type(&command.asset_type))
    .bind(command.amount.as_str())
    .bind(command.currency_code.as_deref().unwrap_or(""))
    .bind(points_delta)
    .bind(&title)
    .bind(&command.transaction_no)
    .bind(ledger_id)
    .bind(occurred_at)
    .bind(occurred_at)
    .execute(executor)
    .await
    .map_err(|error| store_error("failed to insert billing history projection", error))?;

    Ok(())
}

/// Writes a user-facing billing history row for a committed ledger append (Postgres).
pub async fn insert_billing_history_for_ledger_append_postgres<'e, E>(
    executor: E,
    tenant_id: i64,
    organization_id: i64,
    owner_id: i64,
    ledger_id: i64,
    command: &AppendLedgerEntryCommand,
    occurred_at: &str,
) -> Result<(), CommerceServiceError>
where
    E: Executor<'e, Database = Postgres>,
{
    let points_delta = if command.asset_type == CommerceAccountAssetType::Points {
        let raw = command
            .amount
            .as_str()
            .parse::<i64>()
            .map_err(|_| CommerceServiceError::validation("points amount must be an integer"))?;
        match command.direction {
            CommerceLedgerDirection::Credit => raw,
            CommerceLedgerDirection::Debit => -raw,
        }
    } else {
        0
    };

    let direction = command.direction.as_str().to_owned();
    let title = billing_title_for_business_type(&command.business_type, &command.direction);
    let history_type = command.business_type.clone();
    let history_no = format!("ledger:{}", ledger_id);

    sqlx::query(
        r#"
        INSERT INTO commerce_billing_history
            (id, uuid, tenant_id, organization_id, owner_type, owner_id, history_no, history_type,
             direction, asset_code, amount, currency_code, points_delta, status, title,
             reference_no, source_type, source_id, occurred_at, created_at)
        VALUES ($1, $2, $3, $4, 'USER', $5, $6, $7, $8, $9, $10, $11, $12, 1, $13, $14, 'ledger', $15, $16::timestamptz, $16::timestamptz)
        "#,
    )
    .bind(next_entity_id()?)
    .bind(next_entity_uuid())
    .bind(tenant_id)
    .bind(organization_id)
    .bind(owner_id)
    .bind(&history_no)
    .bind(&history_type)
    .bind(&direction)
    .bind(asset_code_from_type(&command.asset_type))
    .bind(command.amount.as_str())
    .bind(command.currency_code.as_deref().unwrap_or(""))
    .bind(points_delta)
    .bind(&title)
    .bind(&command.transaction_no)
    .bind(ledger_id)
    .bind(occurred_at)
    .execute(executor)
    .await
    .map_err(|error| store_error("failed to insert billing history projection", error))?;

    Ok(())
}

fn billing_title_for_business_type(
    business_type: &str,
    direction: &CommerceLedgerDirection,
) -> String {
    let verb = match direction {
        CommerceLedgerDirection::Credit => "Credit",
        CommerceLedgerDirection::Debit => "Debit",
    };
    format!("{verb}: {business_type}")
}
