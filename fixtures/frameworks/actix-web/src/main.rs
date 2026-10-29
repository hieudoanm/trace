use actix_web::{get, web, App, HttpServer};

#[get("/")]
async fn index() -> &'static str {
    "Hello"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(index))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
