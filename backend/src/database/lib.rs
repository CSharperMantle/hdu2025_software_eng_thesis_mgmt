pub mod model;
pub mod schema;

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenvy_macro::dotenv;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn get_conn_pool() -> DbPool {
    let manager = ConnectionManager::<PgConnection>::new(dotenv!("DATABASE_URL"));
    Pool::builder()
        .max_size(5)
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool")
}
