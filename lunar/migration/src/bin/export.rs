use std::env;

#[tokio::main]
async fn main() -> Result<(), sea_orm_migration::prelude::DbErr> {
    let output = env::args()
        .nth(1)
        .expect("usage: migration-export <output-dir>");

    migration::exporter::export_sql(output).await?;

    Ok(())
}