import re
import pathlib

root = pathlib.Path(__file__).resolve().parents[1]
src = (root / "crates/sdkwork-account-repository-sqlx/src/sqlite_hold.rs").read_text(encoding="utf-8")
dst = src.replace("SqliteCommerceAccountStore", "PostgresCommerceAccountStore")
dst = dst.replace("Transaction<'_, Sqlite>", "Transaction<'_, Postgres>")
dst = dst.replace("sqlx::sqlite::SqliteRow", "sqlx::postgres::PgRow")
dst = dst.replace("crate::sqlite_account::", "crate::postgres_account::")
dst = dst.replace(
    "apply_points_lot_movement_for_hold",
    "apply_points_lot_movement_for_hold_postgres",
)


def repl_query(match: re.Match[str]) -> str:
    sql = match.group(1)
    index = 0

    def sub(_: re.Match[str]) -> str:
        nonlocal index
        index += 1
        return f"${index}"

    return 'r#"' + re.sub(r"\?", sub, sql) + '"#'


dst = re.sub(r'r#"(.*?)"#', repl_query, dst, flags=re.DOTALL)
dst = re.sub(r"\n#\[cfg\(test\)\]\nmod tests \{.*\}\s*\Z", "\n", dst, flags=re.DOTALL)
dst = re.sub(
    r"\npub fn hold_request_hash\(body: &str\) -> Result<CommerceRequestHash, CommerceServiceError> \{.*?\n\}\n",
    "\n",
    dst,
    flags=re.DOTALL,
)
dst = dst.replace("use sqlx::{Row, Sqlite, Transaction}", "use sqlx::{Postgres, Row, Transaction}")
(root / "crates/sdkwork-account-repository-sqlx/src/postgres_hold.rs").write_text(dst, encoding="utf-8")
print("generated postgres_hold.rs")
