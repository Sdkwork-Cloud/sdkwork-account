use sdkwork_contract_service::{CommerceAccountAssetType, CommerceServiceError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountSummaryQuery {
    pub owner_user_id: String,
    pub organization_id: Option<String>,
    pub tenant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountLedgerQuery {
    pub account_id: Option<String>,
    pub owner_user_id: String,
    pub organization_id: Option<String>,
    pub tenant_id: String,
    /// Owner subject kind (defaults to USER when None).
    pub owner_type: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WalletAccountListQuery {
    pub asset_type: Option<CommerceAccountAssetType>,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub tenant_id: String,
    /// Owner subject kind (defaults to USER when None).
    pub owner_type: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WalletTransactionListQuery {
    pub account_id: Option<String>,
    pub asset_type: Option<CommerceAccountAssetType>,
    pub cursor: Option<String>,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub tenant_id: String,
    /// Owner subject kind (defaults to USER when None).
    pub owner_type: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WalletTransactionDetailQuery {
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub tenant_id: String,
    pub transaction_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WalletOperationQuery {
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub request_no: String,
    pub tenant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BillingHistoryListQuery {
    pub cursor: Option<String>,
    pub history_type: Option<String>,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub status: Option<String>,
    pub tenant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PointsLotListQuery {
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub tenant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountHoldListQuery {
    pub account_id: Option<String>,
    pub asset_type: Option<CommerceAccountAssetType>,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub status: Option<String>,
    pub tenant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountHoldListQueryInput<'a> {
    pub account_id: Option<&'a str>,
    pub asset_type: Option<CommerceAccountAssetType>,
    pub organization_id: Option<&'a str>,
    pub owner_user_id: &'a str,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub status: Option<&'a str>,
    pub tenant_id: &'a str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PointsLotAllocationListQuery {
    pub ledger_entry_id: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub tenant_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountHoldDetailQuery {
    pub hold_id: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub tenant_id: String,
}

impl AccountSummaryQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
    ) -> Result<Self, CommerceServiceError> {
        Ok(Self {
            owner_user_id: required_text("owner_user_id", owner_user_id)?,
            organization_id: optional_text(organization_id),
            tenant_id: required_text("tenant_id", tenant_id)?,
        })
    }
}

impl AccountLedgerQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        account_id: Option<&str>,
    ) -> Result<Self, CommerceServiceError> {
        Ok(Self {
            account_id: optional_text(account_id),
            owner_user_id: required_text("owner_user_id", owner_user_id)?,
            organization_id: optional_text(organization_id),
            tenant_id: required_text("tenant_id", tenant_id)?,
            owner_type: None,
        })
    }
}

impl WalletAccountListQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        asset_type: Option<CommerceAccountAssetType>,
    ) -> Result<Self, CommerceServiceError> {
        Ok(Self {
            asset_type,
            organization_id: optional_text(organization_id),
            owner_user_id: required_text("owner_user_id", owner_user_id)?,
            tenant_id: required_text("tenant_id", tenant_id)?,
            owner_type: None,
        })
    }
}

impl WalletTransactionListQuery {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        account_id: Option<&str>,
        asset_type: Option<CommerceAccountAssetType>,
        page: Option<i64>,
        page_size: Option<i64>,
        cursor: Option<&str>,
    ) -> Result<Self, CommerceServiceError> {
        if let Some(page) = page {
            if page < 1 {
                return Err(CommerceServiceError::validation(
                    "page must be greater than or equal to 1",
                ));
            }
        }
        if let Some(page_size) = page_size {
            if !(1..=200).contains(&page_size) {
                return Err(CommerceServiceError::validation(
                    "page_size must be between 1 and 200",
                ));
            }
        }
        Ok(Self {
            account_id: optional_text(account_id),
            asset_type,
            cursor: optional_text(cursor),
            organization_id: optional_text(organization_id),
            owner_user_id: required_text("owner_user_id", owner_user_id)?,
            page,
            page_size,
            tenant_id: required_text("tenant_id", tenant_id)?,
            owner_type: None,
        })
    }

    /// Set the owner subject kind for non-user transactions (defaults USER).
    pub fn with_owner_type(mut self, owner_type: &str) -> Self {
        self.owner_type = Some(owner_type.to_string());
        self
    }

    pub fn limit(&self) -> i64 {
        self.page_size.unwrap_or(20).clamp(1, 200)
    }

    pub fn offset(&self) -> i64 {
        let page = self.page.unwrap_or(1).max(1);
        (page - 1) * self.limit()
    }
}

impl WalletTransactionDetailQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        transaction_id: &str,
    ) -> Result<Self, CommerceServiceError> {
        Ok(Self {
            organization_id: optional_text(organization_id),
            owner_user_id: required_text("owner_user_id", owner_user_id)?,
            tenant_id: required_text("tenant_id", tenant_id)?,
            transaction_id: required_text("transaction_id", transaction_id)?,
        })
    }
}

impl WalletOperationQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        request_no: &str,
    ) -> Result<Self, CommerceServiceError> {
        Ok(Self {
            organization_id: optional_text(organization_id),
            owner_user_id: required_text("owner_user_id", owner_user_id)?,
            request_no: required_text("request_no", request_no)?,
            tenant_id: required_text("tenant_id", tenant_id)?,
        })
    }
}

impl BillingHistoryListQuery {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        history_type: Option<&str>,
        status: Option<&str>,
        page: Option<i64>,
        page_size: Option<i64>,
        cursor: Option<&str>,
    ) -> Result<Self, CommerceServiceError> {
        if let Some(page) = page {
            if page < 1 {
                return Err(CommerceServiceError::validation(
                    "page must be greater than or equal to 1",
                ));
            }
        }
        if let Some(page_size) = page_size {
            if !(1..=200).contains(&page_size) {
                return Err(CommerceServiceError::validation(
                    "page_size must be between 1 and 200",
                ));
            }
        }
        Ok(Self {
            cursor: optional_text(cursor),
            history_type: optional_text(history_type),
            organization_id: optional_text(organization_id),
            owner_user_id: required_text("owner_user_id", owner_user_id)?,
            page,
            page_size,
            status: optional_text(status),
            tenant_id: required_text("tenant_id", tenant_id)?,
        })
    }

    pub fn limit(&self) -> i64 {
        self.page_size.unwrap_or(20).clamp(1, 200)
    }

    pub fn offset(&self) -> i64 {
        let page = self.page.unwrap_or(1).max(1);
        (page - 1) * self.limit()
    }
}

impl PointsLotListQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Self, CommerceServiceError> {
        if let Some(page) = page {
            if page < 1 {
                return Err(CommerceServiceError::validation(
                    "page must be greater than or equal to 1",
                ));
            }
        }
        if let Some(page_size) = page_size {
            if !(1..=200).contains(&page_size) {
                return Err(CommerceServiceError::validation(
                    "page_size must be between 1 and 200",
                ));
            }
        }
        Ok(Self {
            organization_id: optional_text(organization_id),
            owner_user_id: required_text("owner_user_id", owner_user_id)?,
            page,
            page_size,
            tenant_id: required_text("tenant_id", tenant_id)?,
        })
    }

    pub fn limit(&self) -> i64 {
        self.page_size.unwrap_or(20).clamp(1, 200)
    }

    pub fn offset(&self) -> i64 {
        let page = self.page.unwrap_or(1).max(1);
        (page - 1) * self.limit()
    }
}

impl AccountHoldListQuery {
    pub fn new(input: AccountHoldListQueryInput<'_>) -> Result<Self, CommerceServiceError> {
        if let Some(page) = input.page {
            if page < 1 {
                return Err(CommerceServiceError::validation(
                    "page must be greater than or equal to 1",
                ));
            }
        }
        if let Some(page_size) = input.page_size {
            if !(1..=200).contains(&page_size) {
                return Err(CommerceServiceError::validation(
                    "page_size must be between 1 and 200",
                ));
            }
        }
        Ok(Self {
            account_id: optional_text(input.account_id),
            asset_type: input.asset_type,
            organization_id: optional_text(input.organization_id),
            owner_user_id: required_text("owner_user_id", input.owner_user_id)?,
            page: input.page,
            page_size: input.page_size,
            status: optional_text(input.status),
            tenant_id: required_text("tenant_id", input.tenant_id)?,
        })
    }

    pub fn limit(&self) -> i64 {
        self.page_size.unwrap_or(20).clamp(1, 200)
    }

    pub fn offset(&self) -> i64 {
        let page = self.page.unwrap_or(1).max(1);
        (page - 1) * self.limit()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PointsReconciliationQuery {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub owner_user_id: Option<String>,
}

impl PointsReconciliationQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: Option<&str>,
    ) -> Result<Self, CommerceServiceError> {
        Ok(Self {
            tenant_id: required_text("tenant_id", tenant_id)?,
            organization_id: optional_text(organization_id),
            owner_user_id: optional_text(owner_user_id),
        })
    }
}

impl AccountHoldDetailQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        hold_id: &str,
    ) -> Result<Self, CommerceServiceError> {
        Ok(Self {
            hold_id: required_text("hold_id", hold_id)?,
            organization_id: optional_text(organization_id),
            owner_user_id: required_text("owner_user_id", owner_user_id)?,
            tenant_id: required_text("tenant_id", tenant_id)?,
        })
    }
}

impl PointsLotAllocationListQuery {
    pub fn new(
        tenant_id: &str,
        organization_id: Option<&str>,
        owner_user_id: &str,
        ledger_entry_id: &str,
    ) -> Result<Self, CommerceServiceError> {
        Ok(Self {
            ledger_entry_id: required_text("ledger_entry_id", ledger_entry_id)?,
            organization_id: optional_text(organization_id),
            owner_user_id: required_text("owner_user_id", owner_user_id)?,
            tenant_id: required_text("tenant_id", tenant_id)?,
        })
    }
}

fn required_text(field_name: &str, value: &str) -> Result<String, CommerceServiceError> {
    crate::validation::require_non_empty(field_name, value)?;
    Ok(value.trim().to_string())
}

fn optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}
