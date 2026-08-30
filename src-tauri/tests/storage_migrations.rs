use rusqlite::Connection;

use mytory_yt_dlp_lib::migrate;

#[test]
fn creates_the_local_queue_schema_without_a_cookie_column() {
    let mut connection = Connection::open_in_memory().unwrap();

    migrate(&mut connection).unwrap();
    migrate(&mut connection).unwrap();

    let tables: Vec<String> = connection
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    let job_columns: Vec<String> = connection
        .prepare("SELECT name FROM pragma_table_info('download_jobs') ORDER BY cid")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();

    assert_eq!(
        tables,
        vec!["app_settings", "download_jobs", "schema_migrations"]
    );
    assert!(!job_columns.iter().any(|column| column.contains("cookie")));
}
