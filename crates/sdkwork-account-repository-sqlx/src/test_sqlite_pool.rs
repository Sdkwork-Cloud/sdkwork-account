use sqlx::SqlitePool;

pub fn account_repository_test_migration_sql() -> &'static str {
    include_str!("../test_migrations/0001_account_repository_test.sql")
}

pub async fn account_migrated_sqlite_memory_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("account repository sqlite memory pool");
    sqlx::query(account_repository_test_migration_sql())
        .execute(&pool)
        .await
        .expect("account repository test migration");
    pool
}
