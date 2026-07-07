use chrono::{DateTime, Utc};
use sdkwork_account_service::StoreListPage;
use sdkwork_contract_service::CommerceServiceError;
use sdkwork_utils_rust::OffsetListPageParams;

use super::parse_wallet_transaction_cursor;

/// SQL fetch size for interactive lists (`LIMIT page_size + 1` pattern).
pub fn fetch_limit_for_page(page_size: i64) -> i64 {
    page_size.saturating_add(1).max(1)
}

/// Resolved store-level paging for SQL list queries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ListSqlPaging {
    pub params: OffsetListPageParams,
    pub fetch_limit: i64,
    pub sql_offset: i64,
    pub keyset_before: Option<DateTime<Utc>>,
}

/// Resolves offset or cursor list paging per `PAGINATION_SPEC.md` and `sdkwork-utils-rust`.
pub fn resolve_list_sql_paging(
    page: Option<i64>,
    page_size: Option<i64>,
    cursor: Option<&str>,
) -> Result<ListSqlPaging, CommerceServiceError> {
    let params = OffsetListPageParams::parse(page, page_size);
    let fetch_limit = fetch_limit_for_page(params.page_size);

    if let Some(raw) = cursor.map(str::trim).filter(|value| !value.is_empty()) {
        if let Some(timestamp) = parse_wallet_transaction_cursor(Some(raw))? {
            return Ok(ListSqlPaging {
                params,
                fetch_limit,
                sql_offset: 0,
                keyset_before: Some(timestamp),
            });
        }
    }

    Ok(ListSqlPaging {
        params,
        fetch_limit,
        sql_offset: params.offset,
        keyset_before: None,
    })
}

/// Truncates a limit+1 result set and preserves total count from `COUNT(*) OVER()`.
pub fn finalize_list_page<T>(
    mut items: Vec<T>,
    page_size: i64,
    total_items: i64,
) -> StoreListPage<T> {
    let has_more = page_size > 0 && items.len() as i64 > page_size;
    if has_more {
        items.truncate(page_size as usize);
    }
    StoreListPage {
        items,
        total_items,
        has_more,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finalize_truncates_limit_plus_one_rows() {
        let page = finalize_list_page(vec![1, 2, 3], 2, 10);
        assert_eq!(page.items, vec![1, 2]);
        assert_eq!(page.total_items, 10);
        assert!(page.has_more);
    }

    #[test]
    fn numeric_cursor_is_rejected_for_prelaunch_standard_pagination() {
        let error = resolve_list_sql_paging(Some(3), Some(20), Some("40"))
            .expect_err("numeric cursor must be rejected");
        assert_eq!(error.code(), "validation");
    }
}
