use actix_web::{http::StatusCode, web, HttpResponse, ResponseError};
use futures::TryStreamExt;
use tera::Tera;

use crate::{auth, repositories::TodoRepository};

#[tracing::instrument(name = "Get all user todos", skip(todos_repository, auth_ctx))]
pub async fn get_all_user_todos<T>(
    tmpl: web::Data<Tera>,
    todos_repository: web::Data<T>,
    auth_ctx: web::ReqData<auth::AuthContext>,
) -> Result<HttpResponse, GetAllUserTodosError>
where
    T: TodoRepository,
{
    let user_id = auth_ctx.principal_id.clone();

    let todos = todos_repository
        .get_ref()
        .get_all_for_user(user_id)
        .try_collect::<Vec<_>>()
        .await?;

    let mut context = tera::Context::new();
    context.insert("todos", &todos);

    let html = tmpl
        .render("todos.html", &context)
        .map_err(|e| anyhow::anyhow!("Failed to render web page: {}", e))
        .map_err(GetAllUserTodosError::UnexpectedError)?;

    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

#[derive(Debug, thiserror::Error)]
pub enum GetAllUserTodosError {
    #[error("Something went wrong")]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for GetAllUserTodosError {
    fn status_code(&self) -> StatusCode {
        match self {
            GetAllUserTodosError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
