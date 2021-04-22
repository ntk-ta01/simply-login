use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use crypto::digest::Digest;
use crypto::md5::Md5;
use serde_derive::Deserialize;
use sqlx::mysql::MySqlPoolOptions;
use std::env;

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct FormData {
    UserID: String,
    password: String,
}

#[derive(Debug, sqlx::FromRow)]
struct User {
    name: String,
    digest: String,
}

async fn confirm(
    form: web::Form<FormData>,
    pool: web::Data<sqlx::Pool<sqlx::MySql>>,
) -> impl Responder {
    let mut md5_func = Md5::new();
    md5_func.input(form.password.as_bytes());
    let row = sqlx::query_as::<_, User>(&format!(
        "SELECT * from tbl where name='{}' and digest='{}'",
        form.UserID,
        md5_func.result_str()
    ))
    .fetch_one(&**pool)
    .await;

    match row {
        Ok(user) => HttpResponse::Ok().body(format!("Welcome, {}!", user.name)),
        Err(_) => HttpResponse::Ok().body(format!("Bad UserID or Password")),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database_url: &str = &env::var("DATABASE_URL").unwrap();
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .unwrap(); // sudo /etc/init.d/mysql start してから動く

    HttpServer::new(move || {
        App::new().data(pool.clone()).service(
            web::resource("/login")
                .app_data(web::FormConfig::default().limit(100))
                .route(web::post().to(confirm)),
        )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
