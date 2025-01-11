use todo_app::configuration;
use todo_app::startup;
use todo_app::telemetry;

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    let settings = configuration::Settings::load().expect("Failed to load configuration settings");
    let subscriber = telemetry::get_subscriber("todo_app", &settings.telemetry, std::io::stdout);
    telemetry::init_subscriber(subscriber);
    let server = startup::init(settings).expect("Failed to start server");

    server.await
}
