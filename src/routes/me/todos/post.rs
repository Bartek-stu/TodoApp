use actix_web::{
    http::{header, StatusCode},
    web, HttpResponse, ResponseError,
};
use serde::Deserialize;

use crate::{
    auth,
    model::{Todo, TodoContent},
    repositories::TodoRepository,
};

#[derive(Deserialize)]
pub struct NewTodo {
    content: String,
}

#[tracing::instrument(name = "Create todo", skip(new_todo, todos_repository, auth_ctx))]
pub async fn create_todo<T>(
    new_todo: web::Form<NewTodo>,
    todos_repository: web::Data<T>,
    auth_ctx: web::ReqData<auth::AuthContext>,
) -> Result<HttpResponse, CreateTodoError>
where
    T: TodoRepository,
{
    let user_id = auth_ctx.principal_id.clone();

    let content: TodoContent = new_todo
        .into_inner()
        .content
        .try_into()
        .map_err(CreateTodoError::ValidationError)?;

    let todo = Todo::new(content, user_id);

    todos_repository
        .get_ref()
        .create(todo)
        .await
        .map_err(CreateTodoError::UnexpectedError)?;

    Ok(HttpResponse::SeeOther()
        .append_header((header::LOCATION, "/me/todos"))
        .finish())
}

#[derive(Debug, thiserror::Error)]
pub enum CreateTodoError {
    #[error("{0}")]
    ValidationError(#[source] anyhow::Error),
    #[error("Something went wrong")]
    UnexpectedError(#[source] anyhow::Error),
}

impl ResponseError for CreateTodoError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            CreateTodoError::ValidationError(_) => StatusCode::BAD_REQUEST,
            CreateTodoError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
