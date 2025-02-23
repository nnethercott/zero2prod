use zero2prod::{
    self, Application, configuration::get_configuration, telemetry::{get_subscriber, init_subscriber}
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let settings = get_configuration().expect("couldn't read settings");
    let application = Application::build(settings).await?;

    application.run_until_stopped().await
}
