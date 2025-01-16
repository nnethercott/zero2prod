use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::{configuration::get_configuration, run};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let settings = get_configuration("configuration.yaml").expect("couldn't read settings");
    let address = format!("127.0.0.1:{}", settings.application_port);

    let listener = TcpListener::bind(address)?;

    let db_pool = PgPool::connect(&settings.database.connection_string())
        .await
        .expect("couldn't connect to db");

    run(listener, db_pool)?.await
}
