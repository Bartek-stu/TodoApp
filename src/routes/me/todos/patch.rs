use actix_web::{http::StatusCode, web, HttpResponse, ResponseError};
use serde::Deserialize;

use crate::{
    auth,
    model::{Todo, TodoId},
    repositories::TodoRepository,
};

#[derive(Deserialize)]
pub struct TodoUpdate {
    done: bool,
}

#[tracing::instrument(
    name = "Update todo",
    skip(todo_id, todo_update, todos_repository, auth_ctx)
)]
pub async fn update_todo<T>(
    todo_id: web::Path<TodoId>,
    todo_update: web::Json<TodoUpdate>,
    todos_repository: web::Data<T>,
    auth_ctx: web::ReqData<auth::AuthContext>,
) -> Result<HttpResponse, UpdateTodoError>
where
    T: TodoRepository,
{
    let user_id = auth_ctx.principal_id.clone();
    let todo_id = todo_id.into_inner();
    let todo = todos_repository
        .get_ref()
        .get_one_for_user(user_id, todo_id)
        .await
        .map_err(UpdateTodoError::UnexpectedError)?;

    if let Some(todo) = todo {
        let todo_update = todo_update.into_inner();
        let updated_todo =
            update_todo_object(todo, todo_update).map_err(UpdateTodoError::ValidationError)?;

        todos_repository
            .get_ref()
            .save(updated_todo)
            .await
            .map_err(UpdateTodoError::UnexpectedError)?;
    }

    Ok(HttpResponse::NoContent().finish())
}

fn update_todo_object(mut current_todo: Todo, todo_update: TodoUpdate) -> anyhow::Result<Todo> {
    if todo_update.done {
        current_todo.mark_as_done();
    } else {
        current_todo.mark_as_unfinished();
    }
    Ok(current_todo)
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateTodoError {
    #[error("{0}")]
    ValidationError(#[source] anyhow::Error),
    #[error("Something went wrong")]
    UnexpectedError(#[source] anyhow::Error),
}

impl ResponseError for UpdateTodoError {
    fn status_code(&self) -> StatusCode {
        match self {
            UpdateTodoError::ValidationError(_) => StatusCode::BAD_REQUEST,
            UpdateTodoError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
