use rusqlite::{Connection, Result};

const MIGRATIONS: &[(&str, &str)] = &[
    (
        "0001_initial",
        include_str!("../migrations/0001_initial.sql"),
    ),
    (
        "0002_scheduler",
        include_str!("../migrations/0002_scheduler.sql"),
    ),
    (
        "0003_subtitles",
        include_str!("../migrations/0003_subtitles.sql"),
    ),
];

pub fn migrate(connection: &mut Connection) -> Result<()> {
    connection.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS schema_migrations (
          name TEXT PRIMARY KEY,
          applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        ",
    )?;

    let transaction = connection.transaction()?;
    for (name, sql) in MIGRATIONS {
        let applied = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE name = ?1)",
            [name],
            |row| row.get::<_, bool>(0),
        )?;

        if !applied {
            transaction.execute_batch(sql)?;
            transaction.execute("INSERT INTO schema_migrations (name) VALUES (?1)", [name])?;
        }
    }
    transaction.commit()
}
