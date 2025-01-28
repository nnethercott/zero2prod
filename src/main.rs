use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::{
    self,
    configuration::get_configuration,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let settings = get_configuration("configuration.yaml").expect("couldn't read settings");
    let app_settings = settings.app;
    let address = format!("{}:{}", app_settings.host, app_settings.port);

    let listener = TcpListener::bind(address)?;

    let db_pool = PgPool::connect_lazy(&settings.database.connection_string())
        .expect("couldn't connect to db");


    zero2prod::run(listener, db_pool)?.await
}
