use crate::auth;
use actix_files::Files;
use actix_web::{dev::Server, middleware, web, App, HttpServer};
use azure_data_cosmos::prelude::{AuthorizationToken, CosmosClient, DatabaseClient};
use secrecy::ExposeSecret;
use tera::Tera;
use tracing_actix_web::TracingLogger;

use crate::{configuration, repositories, routes};

#[tracing::instrument(name = "Initializing server")]
pub fn init(settings: configuration::Settings) -> std::io::Result<Server> {
    let database_client = init_database_client(settings.cosmos);

    let todo_repository = web::Data::new(repositories::CosmosTodoRepository::new(
        database_client.clone(),
    ));

    let tera = Tera::new("templates/**/*").unwrap();

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(todo_repository.clone())
            .app_data(web::Data::new(tera.clone()))
            .service(Files::new("/static", "./static").show_files_listing())
            .route("/", web::get().to(routes::homepage))
            .route("/healthcheck", web::get().to(routes::healthcheck))
            .service(
                web::scope("/me")
                    .wrap(middleware::from_fn(auth::auth_middleware))
                    .route(
                        "todos",
                        web::get().to(routes::me::todos::get_all_user_todos::<
                            repositories::CosmosTodoRepository,
                        >),
                    )
                    .route(
                        "todos",
                        web::post().to(routes::me::todos::create_todo::<
                            repositories::CosmosTodoRepository,
                        >),
                    )
                    .route(
                        "/todos/{todo_id}",
                        web::patch().to(routes::me::todos::update_todo::<
                            repositories::CosmosTodoRepository,
                        >),
                    )
                    .route(
                        "todos/{todo_id}",
                        web::delete().to(routes::me::todos::delete_todo::<
                            repositories::CosmosTodoRepository,
                        >),
                    ),
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run();

    Ok(server)
}

fn init_database_client(database_settings: configuration::CosmosSettings) -> DatabaseClient {
    let auth_token = AuthorizationToken::primary_key(database_settings.primary_key.expose_secret())
        .expect("Invalid cosmos primary key");

    CosmosClient::new(database_settings.account.expose_secret(), auth_token)
        .database_client(database_settings.database_name)
}
