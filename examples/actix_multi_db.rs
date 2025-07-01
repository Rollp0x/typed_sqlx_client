use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use sqlx::{MySql, mysql::MySqlPoolOptions};
use typed_sqlx_client::SqlPool;

// Marker types for different databases and tables
struct MainDb;
struct LogDb;
struct UserTable;
struct LogTable;

type MainDbPool = SqlPool<MySql, MainDb>;
type LogDbPool = SqlPool<MySql, LogDb>;

async fn user_count(main_db: web::Data<MainDbPool>) -> impl Responder {
    let table = main_db.get_table::<UserTable>();
    // Example: count users
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(table.as_ref())
        .await
        .unwrap_or((0,));
    HttpResponse::Ok().body(format!("User count: {}", row.0))
}

async fn log_count(log_db: web::Data<LogDbPool>) -> impl Responder {
    let table = log_db.get_table::<LogTable>();
    // Example: count logs
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM logs")
        .fetch_one(table.as_ref())
        .await
        .unwrap_or((0,));
    HttpResponse::Ok().body(format!("Log count: {}", row.0))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let main_pool = MySqlPoolOptions::new()
        .connect("mysql://user:pass@localhost/main_db")
        .await
        .unwrap();
    let log_pool = MySqlPoolOptions::new()
        .connect("mysql://user:pass@localhost/log_db")
        .await
        .unwrap();

    let main_db = SqlPool::from_pool::<MainDb>(main_pool);
    let log_db = SqlPool::from_pool::<LogDb>(log_pool);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(main_db.clone()))
            .app_data(web::Data::new(log_db.clone()))
            .route("/users/count", web::get().to(user_count))
            .route("/logs/count", web::get().to(log_count))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
