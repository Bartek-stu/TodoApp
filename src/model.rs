use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct UserId(String);

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for UserId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Into<String> for UserId {
    fn into(self) -> String {
        self.0
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub struct TodoId(Uuid);

impl TodoId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TodoId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TodoId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Into<String> for TodoId {
    fn into(self) -> String {
        self.to_string()
    }
}

#[derive(Serialize, Deserialize)]
pub struct TodoContent(String);

impl TryFrom<String> for TodoContent {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        const MIN_LEN: usize = 1;
        const MAX_LEN: usize = 500;
        let value = value.trim();

        if value.len() < MIN_LEN {
            Err(anyhow::anyhow!("Todo content cannot be blank"))
        } else if value.len() > MAX_LEN {
            Err(anyhow::anyhow!(
                "Todo content cannot be longer than {} characters",
                MAX_LEN
            ))
        } else {
            Ok(TodoContent(value.to_string()))
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Todo {
    id: TodoId,
    content: TodoContent,
    done: bool,
    created_by: UserId,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl Todo {
    pub fn new(content: TodoContent, created_by: UserId) -> Self {
        Self {
            id: TodoId::new(),
            content,
            done: false,
            created_by,
            created_at: chrono::Utc::now(),
        }
    }

    pub fn id(&self) -> TodoId {
        self.id
    }

    pub fn created_by(&self) -> UserId {
        self.created_by.clone()
    }

    pub fn mark_as_done(&mut self) {
        self.done = true;
    }

    pub fn mark_as_unfinished(&mut self) {
        self.done = false;
    }

    pub fn update_content(&mut self, content: TodoContent) {
        self.content = content;
    }
}
