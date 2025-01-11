use actix_web::{http::StatusCode, web, HttpResponse, ResponseError};

use crate::{model::TodoId, repositories::TodoRepository};

#[tracing::instrument(name = "Delete todo", skip(todo_id, todos_repository, auth_ctx))]
pub async fn delete_todo<T>(
    todo_id: web::Path<TodoId>,
    todos_repository: web::Data<T>,
    auth_ctx: web::ReqData<crate::auth::AuthContext>,
) -> Result<HttpResponse, DeleteTodoError>
where
    T: TodoRepository,
{
    let user_id = auth_ctx.principal_id.clone();
    let todo_id = todo_id.into_inner();

    todos_repository
        .get_ref()
        .delete_for_user_by_id(user_id, todo_id)
        .await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteTodoError {
    #[error("Something went wrong")]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for DeleteTodoError {
    fn status_code(&self) -> StatusCode {
        match self {
            DeleteTodoError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
