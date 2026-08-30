// 端到端验证：对模拟 v4 的库副本执行 migration_005，验证数据保留
// 临时测试（cargo test 用），验证后删除。
use sqlx::sqlite::SqliteConnectOptions;

#[tokio::test]
async fn test_migration_005_preserves_v4_data() {
    let db_path = r"C:/Users/cicbyte/AppData/Local/Temp/mig005_test.db";
    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(false);
    let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();
    let db = desktop_lib::database::Database::from_pool(pool);

    // 执行迁移（Database::init 会跑全部迁移，005 是关键路径）
    db.run_migrations_for_test().await.unwrap();

    // 验证数据保留
    let projects: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM projects WHERE name = '旧项目A'")
            .fetch_one(db.pool())
            .await
            .unwrap();
    let ds: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM datasources WHERE name = '我的测试库'")
            .fetch_one(db.pool())
            .await
            .unwrap();
    let tables: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM db_tables WHERE name = 'users'")
        .fetch_one(db.pool())
        .await
        .unwrap();
    let cols: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM db_columns WHERE name IN ('id','name')")
        .fetch_one(db.pool())
        .await
        .unwrap();

    assert_eq!(projects, 1, "projects 旧数据应保留");
    assert_eq!(ds, 1, "datasources 旧数据应保留");
    assert_eq!(tables, 1, "db_tables 旧数据应保留");
    assert_eq!(cols, 2, "db_columns 旧数据应保留");

    // 新结构就位
    let has_new: bool = sqlx::query_scalar(
        "SELECT COUNT(*) = 1 FROM pragma_table_info('projects') WHERE name = 'datasource_id'",
    )
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert!(has_new, "projects 应有 datasource_id 新列");

    // 暂存表已清理
    let leftover: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE name LIKE '%__mig005_old'",
    )
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(leftover, 0, "暂存表应已清理");
}
