use actix_web::{web, App, HttpServer};

mod routes;
use routes::init_routes;

use migration::{Migrator, MigratorTrait};
use sea_orm::Database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();

    // get env vars
    dotenvy::dotenv().ok();

    let db = Database::connect(std::env::var("DATABASE_URL").unwrap()).await.unwrap();

    Migrator::up(&db, None).await.unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .configure(init_routes)
    })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await?;

    Ok(())
}