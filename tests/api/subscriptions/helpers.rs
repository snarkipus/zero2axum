use surrealdb_migrations::SurrealdbMigrations;
use zero2axum::configuration::Settings;

// region: -- SurrealDB: Initialize & Migration
pub async fn migrate_db(configuration: Settings) -> Result<(), surrealdb::Error> {
    let db_configuration = configuration.database.with_db();

    SurrealdbMigrations::new(db_configuration)
        .up()
        .await
        .expect("Failed to run migrations.");

    Ok(())
}
// endregion: --- SurrealDB: Initialize & Migration
