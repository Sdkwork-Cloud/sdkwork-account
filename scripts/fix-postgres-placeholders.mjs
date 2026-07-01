import fs from "node:fs";

const path =
  "crates/sdkwork-account-repository-sqlx/src/postgres_account.rs";
const content = fs.readFileSync(path, "utf8");
const fixed = content.replace(/r#"([\s\S]*?)"#/g, (match, sql) => {
  let index = 0;
  const rewritten = sql.replace(/\?/g, () => `$${++index}`);
  return `r#"${rewritten}"#`;
});
fs.writeFileSync(path, fixed);
