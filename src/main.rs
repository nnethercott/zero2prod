use secrecy::ExposeSecret;
use zero2prod::{
    self,
    configuration::get_configuration,
    issue_delivery_workers::run_worker_until_stopped,
    telemetry::{get_subscriber, init_subscriber},
    Application,
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let settings = get_configuration().expect("couldn't read settings");
    let application = tokio::spawn(
        Application::build(settings.clone())
            .await?
            .run_until_stopped(),
    );
    
    dbg!(&settings);
    dbg!(&settings.email_client.auth_token.expose_secret());

    let worker = tokio::spawn(run_worker_until_stopped(settings));

    // NOTE: we run until either the app OR the worker finishes !
    tokio::select! {
        _ = application => {},
        _ = worker => {},
    };
    Ok(())
}
