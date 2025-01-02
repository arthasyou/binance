use sea_orm::{Database, DatabaseConnection, DbErr};

pub async fn connect_db(uri: &str) -> Result<DatabaseConnection, DbErr> {
    Database::connect(uri).await
}
