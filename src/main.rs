use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::{
    self,
    configuration::get_configuration,
    domain::SubscriberEmail,
    email_client::EmailClient,
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

    let db_pool = PgPool::connect_lazy_with(settings.database.with_db());

    let sender_email = settings.email_client.sender().expect("invalid email");
    let email_client = EmailClient::new(
        settings.email_client.base_url,
        sender_email,
        settings.email_client.auth_token,
    );

    zero2prod::run(listener, db_pool, email_client)?.await
}
