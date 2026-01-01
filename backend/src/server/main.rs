#![deny(clippy::all)]

mod route;

use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::{App, HttpServer, cookie::Key, middleware::Logger, web};
use argon2::Argon2;
use dotenvy_macro::dotenv;
use env_logger::Env;

fn derive_cookie_key(passphrase: &str) -> Key {
    let passphrase = passphrase.as_bytes();
    let mut key_material = [0u8; 64];
    Argon2::default()
        .hash_password_into(
            passphrase,
            b"hdu2025_software_eng_thesis_mgmt/backend",
            &mut key_material,
        )
        .expect("Failed to derive cookie key");
    Key::from(&key_material)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or(dotenv!("RUST_LOG")));

    let pool = backend_database::get_conn_pool();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    derive_cookie_key(dotenv!("COOKIE_PASSPHRASE")),
                )
                .cookie_secure(false)
                .build(),
            )
            .app_data(web::Data::new(pool.clone()))
            .service(route::ping)
    })
    .bind((
        dotenv!("SERVER_HOST"),
        dotenv!("SERVER_PORT").parse().unwrap(),
    ))?
    .run()
    .await
}
