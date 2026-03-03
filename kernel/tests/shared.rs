use almond_kernel::kernel::Kernel;
use sea_orm::DatabaseConnection;
use std::fs;
use std::path::Path;
use tokio::sync::OnceCell;

pub fn remove_test_db() -> std::io::Result<()> {
    let path = Path::new("almond.dev.test.db");

    if path.exists() {
        fs::remove_file(path)?;
    }

    Ok(())
}

static DATABASE_PATH: &str = "sqlite://almond.dev.test.db?mode=rwc";
static DB_CONN: OnceCell<DatabaseConnection> = OnceCell::const_new();

pub async fn get_db() -> &'static DatabaseConnection {
    DB_CONN
        .get_or_init(|| async {
            let kernel = Kernel::new(DATABASE_PATH)
                .await
                .expect("error creating kernel");

            kernel
                .run_migrations()
                .await
                .expect("error running migration");

            kernel.connection().clone()
        })
        .await
}
