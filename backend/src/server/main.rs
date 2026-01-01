use actix_web::middleware::Logger;
use actix_web::{App, HttpServer};
use dotenvy_macro::dotenv;
use env_logger::Env;

mod route;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or(dotenv!("RUST_LOG")));

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .service(route::ping)
    })
    .bind((
        dotenv!("SERVER_HOST"),
        dotenv!("SERVER_PORT").parse().unwrap(),
    ))?
    .run()
    .await
}
