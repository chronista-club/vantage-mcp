/// Database migration module
///
/// This module handles database schema migrations.
/// Migrations are embedded in the binary and run automatically on startup.

pub const MIGRATIONS: &str = r#"
-- Migration management is handled by sqlx
-- Place migration files in the migrations/ directory
-- File naming: YYYYMMDDHHMMSS_description.sql
"#;
