use chrono::Utc;
use sdkwork_account_service::{
    AccountHoldDetailQuery, AccountHoldItem, AccountHoldListQuery, AccountTransferItem,
    CreateAccountHoldCommand, CreateAccountTransferCommand, HoldMutationOutcome,
    ReleaseAccountHoldCommand, SettleAccountHoldCommand, TransferMutationOutcome,
    WalletTransactionItem,
};
use sdkwork_contract_service::{
    CommerceAccountAssetType, CommerceLedgerDirection, CommerceRequestHash,
    CommerceServiceError,
};
use sqlx::{Postgres, Row, Transaction};

use crate::postgres_account::PostgresCommerceAccountStore;
use crate::postgres_account::StoredAccount;
use crate::postgres_account::{format_amount_minor, parse_amount_minor};
use crate::store::{
    asset_code_from_type, asset_type_from_code, format_i64, hold_status_label,
    next_entity_id, next_entity_uuid, optional_org_string, org_id_from_option, parse_subject_i64,
    store_error, ACCOUNT_STATUS_ACTIVE, HOLD_CREATE_SCOPE, HOLD_RELEASE_SCOPE, HOLD_SETTLE_SCOPE,
    HOLD_STATUS_HELD, HOLD_STATUS_RELEASED, HOLD_STATUS_SETTLED, OWNER_TYPE_USER,
    TRANSFER_CREATE_SCOPE, TRANSFER_STATUS_COMPLETED,
};

impl PostgresCommerceAccountStore {
    pub async fn create_account_hold(
        &self,
        command: CreateAccountHoldCommand,
        request_hash: CommerceRequestHash,
    ) -> Result<HoldMutationOutcome, CommerceServiceError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|error| store_error("failed to begin hold transaction", error))?;
        let now = Utc::now().to_rfc3339();
        let tenant_id = parse_subject_i64("tenant_id", &command.tenant_id)?;
        let organization_id = org_id_from_option(command.organization_id.as_deref())?;
        let owner_id = parse_subject_i64("owner_user_id", &command.owner_user_id)?;

        if let Some(replayed) =
            try_replay_hold_mutation(&mut tx, tenant_id, HOLD_CREATE_SCOPE, &command.idempotency_key, request_hash.as_str()).await?
        {
            tx.commit().await.map_err(|error| store_error("failed to commit hold replay", error))?;
            return Ok(replayed);
        }

        insert_idempotency_scoped(
            &mut tx,
            tenant_id,
            HOLD_CREATE_SCOPE,
            &command.idempotency_key,
            request_hash.as_str(),
            "hold",
            &now,
        )
        .await?;

        let account = load_account_for_hold(
            &mut tx,
            tenant_id,
            organization_id,
            owner_id,
            &command.account_id,
            &command.asset_type,
            &now,
        )
        .await?;
        let hold_amount = parse_amount_minor(command.amount.as_str())?;
        let available = parse_amount_minor(&account.available_amount)?;
        if available < hold_amount {
            return Err(CommerceServiceError::invalid_state(
                "insufficient available balance for hold",
            ));
        }
        let frozen = parse_amount_minor(&account.frozen_amount)?;
        let next_available = format_amount_minor(available - hold_amount);
        let next_frozen = format_amount_minor(frozen + hold_amount);
        let account = update_account_balances(
            &mut tx,
            &account,
            &next_available,
            &next_frozen,
            &now,
        )
        .await?;

        let hold_id = next_entity_id()?;
        let hold_uuid = next_entity_uuid();
        let source_id = parse_subject_i64("source_id", &command.source_id)?;
        sqlx::query(
            r#"
            INSERT INTO commerce_account_hold
                (id, uuid, tenant_id, organization_id, account_id, owner_type, owner_id, asset_code,
                 amount, settled_amount, released_amount, status, business_type, business_no,
                 source_type, source_id, idempotency_key, request_no, expires_at, version,
                 created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, '0', '0', $10, $11, $12, $13, $14, $15, $16, $17, 0, $18, $19)
            "#,
        )
        .bind(hold_id)
        .bind(&hold_uuid)
        .bind(tenant_id)
        .bind(organization_id)
        .bind(account.id)
        .bind(OWNER_TYPE_USER)
        .bind(owner_id)
        .bind(asset_code_from_type(&command.asset_type))
        .bind(command.amount.as_str())
        .bind(HOLD_STATUS_HELD)
        .bind(&command.business_type)
        .bind(&command.business_no)
        .bind(&command.source_type)
        .bind(source_id)
        .bind(&command.idempotency_key)
        .bind(&command.request_no)
        .bind(command.expires_at.as_deref())
        .bind(&now)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(|error| store_error("failed to insert account hold", error))?;

        let hold = load_hold_by_uuid(&mut tx, tenant_id, &hold_uuid).await?;
        complete_idempotency_scoped(
            &mut tx,
            tenant_id,
            HOLD_CREATE_SCOPE,
            &command.idempotency_key,
            hold_id,
            &serde_json::json!({ "holdUuid": hold_uuid }).to_string(),
            &now,
        )
        .await?;

        tx.commit()
            .await
            .map_err(|error| store_error("failed to commit hold transaction", error))?;

        Ok(HoldMutationOutcome {
            hold,
            account: account.to_wallet_item()?,
            ledger_entry: None,
            replayed: false,
        })
    }

    pub async fn release_account_hold(
        &self,
        command: ReleaseAccountHoldCommand,
        request_hash: CommerceRequestHash,
    ) -> Result<HoldMutationOutcome, CommerceServiceError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|error| store_error("failed to begin hold release transaction", error))?;
        let now = Utc::now().to_rfc3339();
        let tenant_id = parse_subject_i64("tenant_id", &command.tenant_id)?;

        if let Some(replayed) =
            try_replay_hold_mutation(&mut tx, tenant_id, HOLD_RELEASE_SCOPE, &command.idempotency_key, request_hash.as_str()).await?
        {
            tx.commit().await.map_err(|error| store_error("failed to commit hold release replay", error))?;
            return Ok(replayed);
        }

        insert_idempotency_scoped(
            &mut tx,
            tenant_id,
            HOLD_RELEASE_SCOPE,
            &command.idempotency_key,
            request_hash.as_str(),
            "hold",
            &now,
        )
        .await?;

        let hold = load_hold_by_uuid(&mut tx, tenant_id, &command.hold_id).await?;
        if hold.status != "held" {
            return Err(CommerceServiceError::invalid_state(
                "hold is not in held status",
            ));
        }
        let hold_amount = parse_amount_minor(&hold.amount)?;
        let account = load_account_by_internal_id(&mut tx, tenant_id, parse_subject_i64("account_id", &hold.account_id)?).await?;
        let available = parse_amount_minor(&account.available_amount)?;
        let frozen = parse_amount_minor(&account.frozen_amount)?;
        if frozen < hold_amount {
            return Err(CommerceServiceError::invalid_state(
                "hold release exceeds frozen balance",
            ));
        }
        let account = update_account_balances(
            &mut tx,
            &account,
            &format_amount_minor(available + hold_amount),
            &format_amount_minor(frozen - hold_amount),
            &now,
        )
        .await?;

        sqlx::query(
            r#"
            UPDATE commerce_account_hold
            SET status = $1, released_amount = amount, released_at = $2, updated_at = $3, version = version + 1
            WHERE tenant_id = $4 AND uuid = $5 AND status = $6
            "#,
        )
        .bind(HOLD_STATUS_RELEASED)
        .bind(&now)
        .bind(&now)
        .bind(tenant_id)
        .bind(&command.hold_id)
        .bind(HOLD_STATUS_HELD)
        .execute(&mut *tx)
        .await
        .map_err(|error| store_error("failed to release account hold", error))?;

        let hold = load_hold_by_uuid(&mut tx, tenant_id, &command.hold_id).await?;
        complete_idempotency_scoped(
            &mut tx,
            tenant_id,
            HOLD_RELEASE_SCOPE,
            &command.idempotency_key,
            parse_subject_i64("hold_id", &hold.id)?,
            &serde_json::json!({ "holdUuid": hold.uuid }).to_string(),
            &now,
        )
        .await?;

        tx.commit()
            .await
            .map_err(|error| store_error("failed to commit hold release transaction", error))?;

        Ok(HoldMutationOutcome {
            hold,
            account: account.to_wallet_item()?,
            ledger_entry: None,
            replayed: false,
        })
    }

    pub async fn settle_account_hold(
        &self,
        command: SettleAccountHoldCommand,
        request_hash: CommerceRequestHash,
    ) -> Result<HoldMutationOutcome, CommerceServiceError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|error| store_error("failed to begin hold settle transaction", error))?;
        let now = Utc::now().to_rfc3339();
        let tenant_id = parse_subject_i64("tenant_id", &command.tenant_id)?;

        if let Some(replayed) =
            try_replay_hold_mutation(&mut tx, tenant_id, HOLD_SETTLE_SCOPE, &command.idempotency_key, request_hash.as_str()).await?
        {
            tx.commit().await.map_err(|error| store_error("failed to commit hold settle replay", error))?;
            return Ok(replayed);
        }

        insert_idempotency_scoped(
            &mut tx,
            tenant_id,
            HOLD_SETTLE_SCOPE,
            &command.idempotency_key,
            request_hash.as_str(),
            "hold",
            &now,
        )
        .await?;

        let hold = load_hold_by_uuid(&mut tx, tenant_id, &command.hold_id).await?;
        if hold.status != "held" {
            return Err(CommerceServiceError::invalid_state(
                "hold is not in held status",
            ));
        }
        let hold_amount = parse_amount_minor(&hold.amount)?;
        let account = load_account_by_internal_id(&mut tx, tenant_id, parse_subject_i64("account_id", &hold.account_id)?).await?;
        let frozen = parse_amount_minor(&account.frozen_amount)?;
        if frozen < hold_amount {
            return Err(CommerceServiceError::invalid_state(
                "hold settle exceeds frozen balance",
            ));
        }
        let account = update_account_balances(
            &mut tx,
            &account,
            &account.available_amount,
            &format_amount_minor(frozen - hold_amount),
            &now,
        )
        .await?;

        let asset_type = asset_type_from_code(&hold.asset_type)?;
        let ledger_entry = append_settlement_ledger(
            &mut tx,
            tenant_id,
            &account,
            asset_type,
            hold_amount,
            parse_subject_i64("hold_id", &hold.id)?,
            &command,
            &now,
        )
        .await?;

        sqlx::query(
            r#"
            UPDATE commerce_account_hold
            SET status = $1, settled_amount = amount, settled_at = $2, updated_at = $3, version = version + 1
            WHERE tenant_id = $4 AND uuid = $5 AND status = $6
            "#,
        )
        .bind(HOLD_STATUS_SETTLED)
        .bind(&now)
        .bind(&now)
        .bind(tenant_id)
        .bind(&command.hold_id)
        .bind(HOLD_STATUS_HELD)
        .execute(&mut *tx)
        .await
        .map_err(|error| store_error("failed to settle account hold", error))?;

        let hold = load_hold_by_uuid(&mut tx, tenant_id, &command.hold_id).await?;
        complete_idempotency_scoped(
            &mut tx,
            tenant_id,
            HOLD_SETTLE_SCOPE,
            &command.idempotency_key,
            parse_subject_i64("hold_id", &hold.id)?,
            &serde_json::json!({ "holdUuid": hold.uuid, "ledgerEntryUuid": ledger_entry.uuid }).to_string(),
            &now,
        )
        .await?;

        tx.commit()
            .await
            .map_err(|error| store_error("failed to commit hold settle transaction", error))?;

        Ok(HoldMutationOutcome {
            hold,
            account: account.to_wallet_item()?,
            ledger_entry: Some(ledger_entry),
            replayed: false,
        })
    }

    pub async fn list_account_holds(
        &self,
        query: AccountHoldListQuery,
    ) -> Result<Vec<AccountHoldItem>, CommerceServiceError> {
        let tenant_id = parse_subject_i64("tenant_id", &query.tenant_id)?;
        let organization_id = org_id_from_option(query.organization_id.as_deref())?;
        let owner_id = parse_subject_i64("owner_user_id", &query.owner_user_id)?;
        let account_id = match query.account_id.as_deref() {
            Some(value) if !value.trim().is_empty() => Some(parse_subject_i64("account_id", value)?),
            _ => None,
        };
        let asset_code = query
            .asset_type
            .as_ref()
            .map(asset_code_from_type)
            .map(str::to_owned);
        let status = query.status.as_deref().map(hold_status_to_code);

        let rows = sqlx::query(
            r#"
            SELECT hold.id, hold.uuid, hold.tenant_id, hold.organization_id, hold.account_id,
                   hold.owner_id, hold.asset_code, hold.amount, hold.settled_amount,
                   hold.released_amount, hold.status, hold.business_type, hold.business_no,
                   hold.source_type, hold.source_id, hold.request_no, hold.idempotency_key,
                   hold.expires_at, hold.settled_at, hold.released_at, hold.version,
                   hold.created_at, hold.updated_at
            FROM commerce_account_hold hold
            WHERE hold.tenant_id = $1
              AND hold.organization_id = $2
              AND hold.owner_type = $3
              AND hold.owner_id = $4
              AND ($5 IS NULL OR hold.account_id = $6)
              AND ($7 IS NULL OR hold.asset_code = $8)
              AND ($9 IS NULL OR hold.status = $10)
            ORDER BY hold.created_at DESC
            LIMIT $11 OFFSET $12
            "#,
        )
        .bind(tenant_id)
        .bind(organization_id)
        .bind(OWNER_TYPE_USER)
        .bind(owner_id)
        .bind(account_id)
        .bind(account_id)
        .bind(asset_code.as_deref())
        .bind(asset_code.as_deref())
        .bind(status)
        .bind(status)
        .bind(query.limit())
        .bind(query.offset())
        .fetch_all(&self.pool)
        .await
        .map_err(|error| store_error("failed to list account holds", error))?;

        rows.iter().map(map_hold_row).collect()
    }

    pub async fn retrieve_account_hold(
        &self,
        query: AccountHoldDetailQuery,
    ) -> Result<Option<AccountHoldItem>, CommerceServiceError> {
        let tenant_id = parse_subject_i64("tenant_id", &query.tenant_id)?;
        let organization_id = org_id_from_option(query.organization_id.as_deref())?;
        let owner_id = parse_subject_i64("owner_user_id", &query.owner_user_id)?;

        let row = sqlx::query(
            r#"
            SELECT hold.id, hold.uuid, hold.tenant_id, hold.organization_id, hold.account_id,
                   hold.owner_id, hold.asset_code, hold.amount, hold.settled_amount,
                   hold.released_amount, hold.status, hold.business_type, hold.business_no,
                   hold.source_type, hold.source_id, hold.request_no, hold.idempotency_key,
                   hold.expires_at, hold.settled_at, hold.released_at, hold.version,
                   hold.created_at, hold.updated_at
            FROM commerce_account_hold hold
            WHERE hold.tenant_id = $1
              AND hold.organization_id = $2
              AND hold.owner_type = $3
              AND hold.owner_id = $4
              AND hold.uuid = $5
            LIMIT 1
            "#,
        )
        .bind(tenant_id)
        .bind(organization_id)
        .bind(OWNER_TYPE_USER)
        .bind(owner_id)
        .bind(query.hold_id.trim())
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| store_error("failed to retrieve account hold", error))?;

        row.as_ref().map(map_hold_row).transpose()
    }

    pub async fn create_account_transfer(
        &self,
        command: CreateAccountTransferCommand,
        request_hash: CommerceRequestHash,
    ) -> Result<TransferMutationOutcome, CommerceServiceError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|error| store_error("failed to begin transfer transaction", error))?;
        let now = Utc::now().to_rfc3339();
        let tenant_id = parse_subject_i64("tenant_id", &command.tenant_id)?;
        let organization_id = org_id_from_option(command.organization_id.as_deref())?;
        let from_id = parse_subject_i64("from_account_id", &command.from_account_id)?;
        let to_id = parse_subject_i64("to_account_id", &command.to_account_id)?;
        if from_id == to_id {
            return Err(CommerceServiceError::validation(
                "from_account_id and to_account_id must differ",
            ));
        }

        if let Some(row) = load_idempotency_scoped(&mut tx, tenant_id, TRANSFER_CREATE_SCOPE, &command.idempotency_key).await? {
            let stored_hash = string_cell(&row, "request_hash");
            if stored_hash != request_hash.as_str() {
                return Err(CommerceServiceError::conflict(
                    "idempotency key was used with a different request hash",
                ));
            }
            if string_cell(&row, "status") == "COMPLETED" {
                let replayed = load_transfer_replay(&mut tx, tenant_id, &command.idempotency_key).await?;
                tx.commit().await.map_err(|error| store_error("failed to commit transfer replay", error))?;
                return Ok(replayed);
            }
        } else {
            insert_idempotency_scoped(
                &mut tx,
                tenant_id,
                TRANSFER_CREATE_SCOPE,
                &command.idempotency_key,
                request_hash.as_str(),
                "transfer",
                &now,
            )
            .await?;
        }

        let mut from_account = load_account_by_internal_id(&mut tx, tenant_id, from_id).await?;
        let to_account = load_account_by_internal_id(&mut tx, tenant_id, to_id).await?;
        if from_account.asset_type != command.asset_type
            || to_account.asset_type != command.asset_type
        {
            return Err(CommerceServiceError::validation(
                "transfer accounts must match command asset_type",
            ));
        }

        let amount = parse_amount_minor(command.amount.as_str())?;
        let from_available = parse_amount_minor(&from_account.available_amount)?;
        if from_available < amount {
            return Err(CommerceServiceError::invalid_state(
                "insufficient available balance for transfer",
            ));
        }

        from_account = update_account_balances(
            &mut tx,
            &from_account,
            &format_amount_minor(from_available - amount),
            &from_account.frozen_amount,
            &now,
        )
        .await?;
        let to_available = parse_amount_minor(&to_account.available_amount)?;
        let to_account = update_account_balances(
            &mut tx,
            &to_account,
            &format_amount_minor(to_available + amount),
            &to_account.frozen_amount,
            &now,
        )
        .await?;

        let journal_id = next_entity_id()?;
        let journal_uuid = next_entity_uuid();
        let trace_id = next_entity_uuid();
        sqlx::query(
            r#"
            INSERT INTO commerce_account_journal
                (id, uuid, tenant_id, business_type, business_no, request_no, idempotency_key,
                 status, trace_id, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(journal_id)
        .bind(&journal_uuid)
        .bind(tenant_id)
        .bind(&command.business_type)
        .bind(&command.business_no)
        .bind(&command.request_no)
        .bind(&command.idempotency_key)
        .bind(TRANSFER_STATUS_COMPLETED)
        .bind(&trace_id)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(|error| store_error("failed to insert transfer journal", error))?;

        let debit_entry = insert_transfer_ledger(
            &mut tx,
            tenant_id,
            organization_id,
            &from_account,
            journal_id,
            CommerceLedgerDirection::Debit,
            amount,
            &command,
            &trace_id,
            &now,
        )
        .await?;
        let credit_entry = insert_transfer_ledger(
            &mut tx,
            tenant_id,
            organization_id,
            &to_account,
            journal_id,
            CommerceLedgerDirection::Credit,
            amount,
            &command,
            &trace_id,
            &now,
        )
        .await?;

        let transfer_id = next_entity_id()?;
        let transfer_uuid = next_entity_uuid();
        sqlx::query(
            r#"
            INSERT INTO commerce_account_transfer
                (id, uuid, tenant_id, organization_id, from_account_id, to_account_id, asset_code,
                 amount, status, business_type, business_no, idempotency_key, request_no,
                 journal_id, trace_id, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            "#,
        )
        .bind(transfer_id)
        .bind(&transfer_uuid)
        .bind(tenant_id)
        .bind(organization_id)
        .bind(from_account.id)
        .bind(to_account.id)
        .bind(asset_code_from_type(&command.asset_type))
        .bind(command.amount.as_str())
        .bind(TRANSFER_STATUS_COMPLETED)
        .bind(&command.business_type)
        .bind(&command.business_no)
        .bind(&command.idempotency_key)
        .bind(&command.request_no)
        .bind(journal_id)
        .bind(&trace_id)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(|error| store_error("failed to insert account transfer", error))?;

        let transfer = map_transfer_row(
            transfer_id,
            &transfer_uuid,
            tenant_id,
            organization_id,
            from_account.id,
            to_account.id,
            &command,
            journal_id,
            &trace_id,
            &now,
        )?;

        complete_idempotency_scoped(
            &mut tx,
            tenant_id,
            TRANSFER_CREATE_SCOPE,
            &command.idempotency_key,
            transfer_id,
            &serde_json::json!({ "transferUuid": transfer_uuid }).to_string(),
            &now,
        )
        .await?;

        tx.commit()
            .await
            .map_err(|error| store_error("failed to commit transfer transaction", error))?;

        Ok(TransferMutationOutcome {
            transfer,
            from_account: from_account.to_wallet_item()?,
            to_account: to_account.to_wallet_item()?,
            debit_entry,
            credit_entry,
            replayed: false,
        })
    }
}

async fn append_settlement_ledger(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: i64,
    account: &StoredAccount,
    asset_type: CommerceAccountAssetType,
    amount: i128,
    hold_id: i64,
    command: &SettleAccountHoldCommand,
    now: &str,
) -> Result<WalletTransactionItem, CommerceServiceError> {
    let balance_before = account.available_amount.clone();
    let balance_after = account.available_amount.clone();
    let journal_id = next_entity_id()?;
    let journal_uuid = next_entity_uuid();
    let ledger_id = next_entity_id()?;
    let ledger_uuid = next_entity_uuid();
    let trace_id = next_entity_uuid();

    sqlx::query(
        r#"
        INSERT INTO commerce_account_journal
            (id, uuid, tenant_id, business_type, business_no, request_no, idempotency_key,
             status, trace_id, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
    )
    .bind(journal_id)
    .bind(&journal_uuid)
    .bind(tenant_id)
    .bind(&command.business_type)
    .bind(&command.transaction_no)
    .bind(&command.request_no)
    .bind(&command.idempotency_key)
    .bind(ACCOUNT_STATUS_ACTIVE)
    .bind(&trace_id)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to insert settlement journal", error))?;

    sqlx::query(
        r#"
        INSERT INTO commerce_account_ledger
            (id, uuid, tenant_id, organization_id, account_id, journal_id, owner_type, owner_id,
             asset_code, currency_code, ledger_type, entry_type, direction, amount,
             balance_before, balance_after, business_type, business_no, request_no,
             idempotency_key, hold_id, trace_id, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'AVAILABLE', 'DEBIT', 'debit', $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
        "#,
    )
    .bind(ledger_id)
    .bind(&ledger_uuid)
    .bind(tenant_id)
    .bind(account.organization_id)
    .bind(account.id)
    .bind(journal_id)
    .bind(OWNER_TYPE_USER)
    .bind(account.owner_id)
    .bind(asset_code_from_type(&asset_type))
    .bind(&account.currency_code)
    .bind(format_amount_minor(amount))
    .bind(&balance_before)
    .bind(&balance_after)
    .bind(&command.business_type)
    .bind(&command.transaction_no)
    .bind(&command.request_no)
    .bind(&command.idempotency_key)
    .bind(hold_id)
    .bind(&trace_id)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to insert settlement ledger", error))?;

    if asset_type == CommerceAccountAssetType::Points {
        crate::postgres_account::apply_points_lot_movement_for_hold_postgres(
            tx,
            tenant_id,
            account.id,
            &CommerceLedgerDirection::Debit,
            i64::try_from(amount).map_err(|_| {
                CommerceServiceError::validation("points amount exceeds supported lot range")
            })?,
            &command.business_type,
            ledger_id,
            now,
        )
        .await?;
    }

    WalletTransactionItem::new(
        &format_i64(ledger_id),
        &ledger_uuid,
        &format_i64(account.id),
        &format_i64(tenant_id),
        optional_org_string(account.organization_id).as_deref(),
        &format_i64(account.owner_id),
        asset_type,
        CommerceLedgerDirection::Debit,
        &format_amount_minor(amount),
        &balance_before,
        &balance_after,
        &command.business_type,
        &command.transaction_no,
        &command.request_no,
        &command.idempotency_key,
        now,
    )
}

async fn insert_transfer_ledger(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: i64,
    organization_id: i64,
    account: &StoredAccount,
    journal_id: i64,
    direction: CommerceLedgerDirection,
    amount: i128,
    command: &CreateAccountTransferCommand,
    trace_id: &str,
    now: &str,
) -> Result<WalletTransactionItem, CommerceServiceError> {
    let available = parse_amount_minor(&account.available_amount)?;
    let (balance_before, balance_after) = match direction {
        CommerceLedgerDirection::Debit => {
            let before = available + amount;
            (format_amount_minor(before), account.available_amount.clone())
        }
        CommerceLedgerDirection::Credit => {
            let before = available - amount;
            (format_amount_minor(before), account.available_amount.clone())
        }
    };
    let ledger_id = next_entity_id()?;
    let ledger_uuid = next_entity_uuid();
    let entry_type = match direction {
        CommerceLedgerDirection::Credit => "CREDIT",
        CommerceLedgerDirection::Debit => "DEBIT",
    };
    let ledger_business_no = format!("{}:{}", command.business_no, direction.as_str());

    sqlx::query(
        r#"
        INSERT INTO commerce_account_ledger
            (id, uuid, tenant_id, organization_id, account_id, journal_id, owner_type, owner_id,
             asset_code, currency_code, ledger_type, entry_type, direction, amount,
             balance_before, balance_after, business_type, business_no, request_no,
             idempotency_key, trace_id, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'AVAILABLE', $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
        "#,
    )
    .bind(ledger_id)
    .bind(&ledger_uuid)
    .bind(tenant_id)
    .bind(organization_id)
    .bind(account.id)
    .bind(journal_id)
    .bind(OWNER_TYPE_USER)
    .bind(account.owner_id)
    .bind(asset_code_from_type(&account.asset_type))
    .bind(&account.currency_code)
    .bind(entry_type)
    .bind(direction.as_str())
    .bind(format_amount_minor(amount))
    .bind(&balance_before)
    .bind(&balance_after)
    .bind(&command.business_type)
    .bind(&ledger_business_no)
    .bind(&command.request_no)
    .bind(format!("{}:{}", command.idempotency_key, direction.as_str()))
    .bind(trace_id)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to insert transfer ledger", error))?;

    sqlx::query(
        r#"
        INSERT INTO commerce_account_journal_line
            (id, journal_id, account_id, direction, amount, ledger_id, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(next_entity_id()?)
    .bind(journal_id)
    .bind(account.id)
    .bind(direction.as_str())
    .bind(format_amount_minor(amount))
    .bind(ledger_id)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to insert transfer journal line", error))?;

    let entry_idempotency = format!("{}:{}", command.idempotency_key, direction.as_str());
    WalletTransactionItem::new(
        &format_i64(ledger_id),
        &ledger_uuid,
        &format_i64(account.id),
        &format_i64(tenant_id),
        optional_org_string(organization_id).as_deref(),
        &format_i64(account.owner_id),
        account.asset_type.clone(),
        direction,
        &format_amount_minor(amount),
        &balance_before,
        &balance_after,
        &command.business_type,
        &ledger_business_no,
        &command.request_no,
        &entry_idempotency,
        now,
    )
}

async fn load_account_for_hold(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: i64,
    organization_id: i64,
    owner_id: i64,
    account_id: &str,
    asset_type: &CommerceAccountAssetType,
    now: &str,
) -> Result<StoredAccount, CommerceServiceError> {
    let trimmed = account_id.trim();
    if !trimmed.is_empty() {
        let account_id = parse_subject_i64("account_id", trimmed)?;
        return load_account_by_internal_id(tx, tenant_id, account_id).await;
    }
    load_account_by_owner_asset(tx, tenant_id, organization_id, owner_id, asset_type, now).await
}

async fn load_account_by_owner_asset(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: i64,
    organization_id: i64,
    owner_id: i64,
    asset_type: &CommerceAccountAssetType,
    now: &str,
) -> Result<StoredAccount, CommerceServiceError> {
    if let Some(account) = sqlx::query(
        r#"
        SELECT id, uuid, tenant_id, organization_id, owner_id, asset_code, currency_code,
               available_amount, frozen_amount, pending_amount, status, version
        FROM commerce_account
        WHERE tenant_id = $1 AND organization_id = $2 AND owner_type = $3 AND owner_id = $4 AND asset_code = $5
        LIMIT 1
        "#,
    )
    .bind(tenant_id)
    .bind(organization_id)
    .bind(OWNER_TYPE_USER)
    .bind(owner_id)
    .bind(asset_code_from_type(asset_type))
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| store_error("failed to load account for hold", error))?
    {
        return map_stored_account(&account);
    }

    let account_id = next_entity_id()?;
    let account_uuid = next_entity_uuid();
    let currency_code = match asset_type {
        CommerceAccountAssetType::Cash => "",
        CommerceAccountAssetType::Points => "POINT",
        CommerceAccountAssetType::Token => "TOKEN",
    };
    sqlx::query(
        r#"
        INSERT INTO commerce_account
            (id, uuid, tenant_id, organization_id, owner_type, owner_id, asset_code, currency_code,
             available_amount, frozen_amount, pending_amount, status, version, purpose, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, '0', '0', '0', $9, 0, $10, $11, $12)
        "#,
    )
    .bind(account_id)
    .bind(&account_uuid)
    .bind(tenant_id)
    .bind(organization_id)
    .bind(OWNER_TYPE_USER)
    .bind(owner_id)
    .bind(asset_code_from_type(asset_type))
    .bind(currency_code)
    .bind(ACCOUNT_STATUS_ACTIVE)
    .bind(crate::store::ACCOUNT_PURPOSE_GENERAL)
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to create account for hold", error))?;

    Ok(StoredAccount {
        id: account_id,
        uuid: account_uuid,
        tenant_id,
        organization_id,
        owner_id,
        asset_type: asset_type.clone(),
        currency_code: currency_code.to_owned(),
        available_amount: "0".to_owned(),
        frozen_amount: "0".to_owned(),
        pending_amount: "0".to_owned(),
        status: ACCOUNT_STATUS_ACTIVE,
        version: 0,
    })
}

async fn load_account_by_internal_id(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: i64,
    account_id: i64,
) -> Result<StoredAccount, CommerceServiceError> {
    let row = sqlx::query(
        r#"
        SELECT id, uuid, tenant_id, organization_id, owner_id, asset_code, currency_code,
               available_amount, frozen_amount, pending_amount, status, version
        FROM commerce_account
        WHERE tenant_id = $1 AND id = $2
        LIMIT 1
        "#,
    )
    .bind(tenant_id)
    .bind(account_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| store_error("failed to load account", error))?
    .ok_or_else(|| CommerceServiceError::not_found("account was not found"))?;
    map_stored_account(&row)
}

async fn update_account_balances(
    tx: &mut Transaction<'_, Postgres>,
    account: &StoredAccount,
    available_amount: &str,
    frozen_amount: &str,
    now: &str,
) -> Result<StoredAccount, CommerceServiceError> {
    let next_version = account.version.checked_add(1).ok_or_else(|| {
        CommerceServiceError::storage("commerce account version increment overflow")
    })?;
    let update = sqlx::query(
        r#"
        UPDATE commerce_account
        SET available_amount = $1, frozen_amount = $2, version = $3, updated_at = $4
        WHERE id = $5 AND version = $6
        "#,
    )
    .bind(available_amount)
    .bind(frozen_amount)
    .bind(next_version)
    .bind(now)
    .bind(account.id)
    .bind(account.version)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to update account balances", error))?;
    if update.rows_affected() != 1 {
        return Err(CommerceServiceError::conflict(
            "commerce account balance update was not applied atomically",
        ));
    }
    Ok(StoredAccount {
        available_amount: available_amount.to_owned(),
        frozen_amount: frozen_amount.to_owned(),
        version: next_version,
        ..account.clone()
    })
}

async fn load_hold_by_uuid(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: i64,
    hold_uuid: &str,
) -> Result<AccountHoldItem, CommerceServiceError> {
    let row = sqlx::query(
        r#"
        SELECT id, uuid, tenant_id, organization_id, account_id, owner_id, asset_code, amount,
               settled_amount, released_amount, status, business_type, business_no, source_type,
               source_id, request_no, idempotency_key, expires_at, settled_at, released_at,
               version, created_at, updated_at
        FROM commerce_account_hold
        WHERE tenant_id = $1 AND uuid = $2
        LIMIT 1
        "#,
    )
    .bind(tenant_id)
    .bind(hold_uuid.trim())
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| store_error("failed to load account hold", error))?
    .ok_or_else(|| CommerceServiceError::not_found("account hold was not found"))?;
    map_hold_row(&row)
}

fn map_hold_row(row: &sqlx::postgres::PgRow) -> Result<AccountHoldItem, CommerceServiceError> {
    Ok(AccountHoldItem {
        id: format_i64(integer_cell(row, "id")),
        uuid: string_cell(row, "uuid"),
        tenant_id: format_i64(integer_cell(row, "tenant_id")),
        organization_id: optional_org_string(integer_cell(row, "organization_id")),
        account_id: format_i64(integer_cell(row, "account_id")),
        owner_user_id: format_i64(integer_cell(row, "owner_id")),
        asset_type: asset_type_from_code(&string_cell(row, "asset_code"))?.as_str().to_owned(),
        amount: string_cell(row, "amount"),
        settled_amount: string_cell(row, "settled_amount"),
        released_amount: string_cell(row, "released_amount"),
        status: hold_status_label(integer_cell(row, "status") as i32).to_owned(),
        business_type: string_cell(row, "business_type"),
        business_no: string_cell(row, "business_no"),
        source_type: string_cell(row, "source_type"),
        source_id: format_i64(integer_cell(row, "source_id")),
        request_no: string_cell(row, "request_no"),
        idempotency_key: string_cell(row, "idempotency_key"),
        expires_at: optional_string_cell(row, "expires_at"),
        settled_at: optional_string_cell(row, "settled_at"),
        released_at: optional_string_cell(row, "released_at"),
        version: integer_cell(row, "version"),
        created_at: string_cell(row, "created_at"),
        updated_at: string_cell(row, "updated_at"),
    })
}

fn map_transfer_row(
    transfer_id: i64,
    transfer_uuid: &str,
    tenant_id: i64,
    organization_id: i64,
    from_account_id: i64,
    to_account_id: i64,
    command: &CreateAccountTransferCommand,
    journal_id: i64,
    trace_id: &str,
    created_at: &str,
) -> Result<AccountTransferItem, CommerceServiceError> {
    Ok(AccountTransferItem {
        id: format_i64(transfer_id),
        uuid: transfer_uuid.to_owned(),
        tenant_id: format_i64(tenant_id),
        organization_id: optional_org_string(organization_id),
        from_account_id: format_i64(from_account_id),
        to_account_id: format_i64(to_account_id),
        owner_user_id: command.owner_user_id.clone(),
        asset_type: command.asset_type.as_str().to_owned(),
        amount: command.amount.as_str().to_owned(),
        status: "completed".to_owned(),
        business_type: command.business_type.clone(),
        business_no: command.business_no.clone(),
        request_no: command.request_no.clone(),
        idempotency_key: command.idempotency_key.clone(),
        journal_id: format_i64(journal_id),
        trace_id: trace_id.to_owned(),
        created_at: created_at.to_owned(),
    })
}

async fn try_replay_hold_mutation(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: i64,
    scope: &str,
    idempotency_key: &str,
    request_hash: &str,
) -> Result<Option<HoldMutationOutcome>, CommerceServiceError> {
    let Some(row) = load_idempotency_scoped(tx, tenant_id, scope, idempotency_key).await? else {
        return Ok(None);
    };
    if string_cell(&row, "request_hash") != request_hash {
        return Err(CommerceServiceError::conflict(
            "idempotency key was used with a different request hash",
        ));
    }
    if string_cell(&row, "status") != "COMPLETED" {
        return Ok(None);
    }
    let snapshot: serde_json::Value = serde_json::from_str(&string_cell(&row, "response_snapshot"))
        .map_err(|error| store_error("failed to parse hold idempotency snapshot", error))?;
    let hold_uuid = snapshot
        .get("holdUuid")
        .and_then(|value| value.as_str())
        .ok_or_else(|| CommerceServiceError::storage("hold idempotency snapshot missing holdUuid"))?;
    let hold = load_hold_by_uuid(tx, tenant_id, hold_uuid).await?;
    let account = load_account_by_internal_id(tx, tenant_id, parse_subject_i64("account_id", &hold.account_id)?).await?;
    Ok(Some(HoldMutationOutcome {
        hold,
        account: account.to_wallet_item()?,
        ledger_entry: None,
        replayed: true,
    }))
}

async fn load_transfer_replay(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: i64,
    idempotency_key: &str,
) -> Result<TransferMutationOutcome, CommerceServiceError> {
    let row = sqlx::query(
        r#"
        SELECT id, uuid, tenant_id, organization_id, from_account_id, to_account_id, asset_code,
               amount, status, business_type, business_no, request_no, idempotency_key,
               journal_id, trace_id, created_at
        FROM commerce_account_transfer
        WHERE tenant_id = $1 AND idempotency_key = $2
        LIMIT 1
        "#,
    )
    .bind(tenant_id)
    .bind(idempotency_key)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| store_error("failed to load transfer replay", error))?
    .ok_or_else(|| CommerceServiceError::not_found("transfer replay target was not found"))?;

    let transfer = AccountTransferItem {
        id: format_i64(integer_cell(&row, "id")),
        uuid: string_cell(&row, "uuid"),
        tenant_id: format_i64(integer_cell(&row, "tenant_id")),
        organization_id: optional_org_string(integer_cell(&row, "organization_id")),
        from_account_id: format_i64(integer_cell(&row, "from_account_id")),
        to_account_id: format_i64(integer_cell(&row, "to_account_id")),
        owner_user_id: String::new(),
        asset_type: asset_type_from_code(&string_cell(&row, "asset_code"))?.as_str().to_owned(),
        amount: string_cell(&row, "amount"),
        status: "completed".to_owned(),
        business_type: string_cell(&row, "business_type"),
        business_no: string_cell(&row, "business_no"),
        request_no: string_cell(&row, "request_no"),
        idempotency_key: string_cell(&row, "idempotency_key"),
        journal_id: format_i64(integer_cell(&row, "journal_id")),
        trace_id: string_cell(&row, "trace_id"),
        created_at: string_cell(&row, "created_at"),
    };
    let from_account =
        load_account_by_internal_id(tx, tenant_id, integer_cell(&row, "from_account_id")).await?;
    let to_account =
        load_account_by_internal_id(tx, tenant_id, integer_cell(&row, "to_account_id")).await?;
    Ok(TransferMutationOutcome {
        transfer,
        from_account: from_account.to_wallet_item()?,
        to_account: to_account.to_wallet_item()?,
        debit_entry: load_transfer_ledger_by_journal(
            tx,
            tenant_id,
            integer_cell(&row, "journal_id"),
            from_account.id,
            CommerceLedgerDirection::Debit,
        )
        .await?,
        credit_entry: load_transfer_ledger_by_journal(
            tx,
            tenant_id,
            integer_cell(&row, "journal_id"),
            to_account.id,
            CommerceLedgerDirection::Credit,
        )
        .await?,
        replayed: true,
    })
}

async fn load_transfer_ledger_by_journal(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: i64,
    journal_id: i64,
    account_id: i64,
    direction: CommerceLedgerDirection,
) -> Result<WalletTransactionItem, CommerceServiceError> {
    let row = sqlx::query(
        r#"
        SELECT id, uuid, account_id, tenant_id, organization_id, owner_id, asset_code,
               direction, amount, balance_before, balance_after, business_type, business_no,
               request_no, idempotency_key, created_at
        FROM commerce_account_ledger
        WHERE tenant_id = $1 AND journal_id = $2 AND account_id = $3 AND direction = $4
        LIMIT 1
        "#,
    )
    .bind(tenant_id)
    .bind(journal_id)
    .bind(account_id)
    .bind(direction.as_str())
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| store_error("failed to load transfer ledger replay", error))?
    .ok_or_else(|| CommerceServiceError::invalid_state("transfer ledger replay entry missing"))?;

    WalletTransactionItem::new(
        &format_i64(integer_cell(&row, "id")),
        &string_cell(&row, "uuid"),
        &format_i64(integer_cell(&row, "account_id")),
        &format_i64(integer_cell(&row, "tenant_id")),
        optional_org_string(integer_cell(&row, "organization_id")).as_deref(),
        &format_i64(integer_cell(&row, "owner_id")),
        asset_type_from_code(&string_cell(&row, "asset_code"))?,
        parse_direction(&string_cell(&row, "direction"))?,
        &string_cell(&row, "amount"),
        &string_cell(&row, "balance_before"),
        &string_cell(&row, "balance_after"),
        &string_cell(&row, "business_type"),
        &string_cell(&row, "business_no"),
        &string_cell(&row, "request_no"),
        &string_cell(&row, "idempotency_key"),
        &string_cell(&row, "created_at"),
    )
}

fn parse_direction(value: &str) -> Result<CommerceLedgerDirection, CommerceServiceError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "credit" => Ok(CommerceLedgerDirection::Credit),
        "debit" => Ok(CommerceLedgerDirection::Debit),
        _ => Err(CommerceServiceError::validation("direction is invalid")),
    }
}

async fn load_idempotency_scoped(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: i64,
    scope: &str,
    idempotency_key: &str,
) -> Result<Option<sqlx::postgres::PgRow>, CommerceServiceError> {
    sqlx::query(
        r#"
        SELECT request_hash, status, response_snapshot
        FROM commerce_idempotency_record
        WHERE tenant_id = $1 AND scope = $2 AND idempotency_key = $3
        LIMIT 1
        "#,
    )
    .bind(tenant_id)
    .bind(scope)
    .bind(idempotency_key)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|error| store_error("failed to load idempotency record", error))
}

async fn insert_idempotency_scoped(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: i64,
    scope: &str,
    idempotency_key: &str,
    request_hash: &str,
    target_type: &str,
    now: &str,
) -> Result<(), CommerceServiceError> {
    sqlx::query(
        r#"
        INSERT INTO commerce_idempotency_record
            (id, uuid, tenant_id, scope, idempotency_key, request_hash, target_type, status,
             locked_until, expire_at, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 'LOCKED', $8, $9, $10, $11)
        "#,
    )
    .bind(next_entity_id()?)
    .bind(next_entity_uuid())
    .bind(tenant_id)
    .bind(scope)
    .bind(idempotency_key)
    .bind(request_hash)
    .bind(target_type)
    .bind(now)
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to insert idempotency lock", error))?;
    Ok(())
}

async fn complete_idempotency_scoped(
    tx: &mut Transaction<'_, Postgres>,
    tenant_id: i64,
    scope: &str,
    idempotency_key: &str,
    target_id: i64,
    response_snapshot: &str,
    now: &str,
) -> Result<(), CommerceServiceError> {
    sqlx::query(
        r#"
        UPDATE commerce_idempotency_record
        SET status = 'COMPLETED', target_id = $1, response_snapshot = $2, locked_until = NULL, updated_at = $3
        WHERE tenant_id = $4 AND scope = $5 AND idempotency_key = $6
        "#,
    )
    .bind(target_id)
    .bind(response_snapshot)
    .bind(now)
    .bind(tenant_id)
    .bind(scope)
    .bind(idempotency_key)
    .execute(&mut **tx)
    .await
    .map_err(|error| store_error("failed to complete idempotency record", error))?;
    Ok(())
}

fn map_stored_account(row: &sqlx::postgres::PgRow) -> Result<StoredAccount, CommerceServiceError> {
    crate::postgres_account::map_stored_account(row)
}

fn hold_status_to_code(status: &str) -> i32 {
    match status.trim().to_ascii_lowercase().as_str() {
        "held" => HOLD_STATUS_HELD,
        "settled" => HOLD_STATUS_SETTLED,
        "released" => HOLD_STATUS_RELEASED,
        "expired" => crate::store::HOLD_STATUS_EXPIRED,
        _ => HOLD_STATUS_HELD,
    }
}

fn string_cell(row: &sqlx::postgres::PgRow, name: &str) -> String {
    row.try_get::<String, _>(name).unwrap_or_default()
}

fn optional_string_cell(row: &sqlx::postgres::PgRow, name: &str) -> Option<String> {
    row.try_get::<Option<String>, _>(name).ok().flatten()
}

fn integer_cell(row: &sqlx::postgres::PgRow, name: &str) -> i64 {
    row.try_get::<i64, _>(name).unwrap_or_default()
}

