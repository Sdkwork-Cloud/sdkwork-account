use sqlx::SqlitePool;

pub fn account_repository_test_migration_sql() -> &'static str {
    // Path is compile-time embedded; touch this file when test SQL changes.
    include_str!("../test_migrations/0001_account_repository_test.sql")
}

pub async fn account_migrated_sqlite_memory_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("account repository sqlite memory pool");
    for statement in split_sql_statements(account_repository_test_migration_sql()) {
        sqlx::query(&statement)
            .execute(&pool)
            .await
            .unwrap_or_else(|error| {
                panic!("account repository test migration failed on `{statement}`: {error}")
            });
    }
    pool
}

fn split_sql_statements(sql: &str) -> Vec<String> {
    sql.split(';')
        .map(|chunk| {
            chunk
                .lines()
                .filter(|line| {
                    let trimmed = line.trim_start();
                    !trimmed.is_empty() && !trimmed.starts_with("--")
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .map(|statement| statement.trim().to_string())
        .filter(|statement| !statement.is_empty())
        .collect()
}
