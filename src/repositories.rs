use std::marker::PhantomData;

use crate::model::{Todo, TodoId, UserId};
use anyhow::Context;
use azure_data_cosmos::{
    prelude::{CollectionClient, DatabaseClient, GetDocumentResponse, Query},
    CosmosEntity,
};
use futures::{StreamExt, TryStreamExt};
use serde::{de::DeserializeOwned, Serialize};

trait CosmosDocument: CosmosEntity {
    const COLLECTION_NAME: &str;
    type Id: Into<String>;
}

struct CosmosDocumentRepository<T: CosmosEntity + CosmosDocument> {
    collection_client: CollectionClient,
    _marker: PhantomData<T>,
}

impl<T> CosmosDocumentRepository<T>
where
    T: CosmosDocument,
{
    pub fn new(database_client: DatabaseClient) -> Self {
        let collection_client = database_client.collection_client(T::COLLECTION_NAME);
        Self {
            collection_client,
            _marker: PhantomData,
        }
    }

    pub fn collection_client(&self) -> &CollectionClient {
        &self.collection_client
    }
}

impl<T> CosmosDocumentRepository<T>
where
    T: CosmosDocument + Send + DeserializeOwned,
    <T as CosmosEntity>::Entity: Into<String> + Clone,
{
    pub async fn get_by_id(
        &self,
        id: T::Id,
        partition_key: T::Entity,
    ) -> anyhow::Result<Option<T>> {
        let result: GetDocumentResponse<T> = self
            .collection_client
            .document_client(id, &partition_key)?
            .get_document()
            .await?;

        match result {
            GetDocumentResponse::Found(resp) => Ok(Some(resp.document.document)),
            GetDocumentResponse::NotFound(_) => Ok(None),
        }
    }
}

impl<T> CosmosDocumentRepository<T>
where
    T: CosmosDocument + Send + Serialize + 'static,
{
    async fn save(&self, document: T, is_upsert: bool) -> anyhow::Result<()> {
        self.collection_client
            .create_document(document)
            .is_upsert(is_upsert)
            .await
            .context("Failed to store user document")
            .and_then(|_| Ok(()))
    }
}

pub trait TodoRepository {
    fn get_all_for_user(&self, user_id: UserId)
        -> impl StreamExt<Item = anyhow::Result<Todo>> + '_;
    async fn get_one_for_user(
        &self,
        user_id: UserId,
        todo_id: TodoId,
    ) -> anyhow::Result<Option<Todo>>;
    async fn delete_for_user_by_id(&self, user_id: UserId, todo_id: TodoId) -> anyhow::Result<()>;
    async fn create(&self, todo: Todo) -> anyhow::Result<()>;
    async fn save(&self, todo: Todo) -> anyhow::Result<()>;
}

impl CosmosEntity for Todo {
    type Entity = UserId;

    fn partition_key(&self) -> Self::Entity {
        self.created_by()
    }
}

impl CosmosDocument for Todo {
    const COLLECTION_NAME: &str = "todos";
    type Id = TodoId;
}

pub struct CosmosTodoRepository {
    cosmos_repository: CosmosDocumentRepository<Todo>,
}

impl CosmosTodoRepository {
    pub fn new(database_client: DatabaseClient) -> Self {
        let cosmos_repository = CosmosDocumentRepository::new(database_client);
        Self { cosmos_repository }
    }
}

impl TodoRepository for CosmosTodoRepository {
    #[tracing::instrument(name = "Fetch todos from db by user id", skip(self, user_id))]
    fn get_all_for_user(
        &self,
        user_id: UserId,
    ) -> impl StreamExt<Item = anyhow::Result<Todo>> + '_ {
        let statement = format!(
            "SELECT * FROM {} t WHERE t.created_by = '{}'",
            Todo::COLLECTION_NAME,
            user_id
        );
        let query = Query::new(statement);

        self.cosmos_repository
            .collection_client()
            .query_documents(query)
            .query_cross_partition(false)
            .into_stream::<Todo>()
            .map_err(anyhow::Error::from)
            .map_ok(|response| {
                futures::stream::iter(response.results.into_iter().map(|doc| Ok(doc.0)))
            })
            .try_flatten()
    }

    #[tracing::instrument(name = "Fetch one todo for user from db", skip(self, user_id, todo_id))]
    async fn get_one_for_user(
        &self,
        user_id: UserId,
        todo_id: TodoId,
    ) -> anyhow::Result<Option<Todo>> {
        self.cosmos_repository.get_by_id(todo_id, user_id).await
    }

    #[tracing::instrument(
        name = "Delete todo from db by id and user_id",
        skip(self, user_id, todo_id)
    )]
    async fn delete_for_user_by_id(&self, user_id: UserId, todo_id: TodoId) -> anyhow::Result<()> {
        let statement = format!(
            "SELECT * FROM {} t WHERE t.created_by = '{}' AND t.id = '{}' ",
            Todo::COLLECTION_NAME,
            user_id,
            todo_id
        );

        let query = Query::new(statement);

        let mut stream = self
            .cosmos_repository
            .collection_client()
            .query_documents(query)
            .query_cross_partition(false)
            .into_stream::<Todo>();

        while let Some(response) = stream.next().await {
            for document in response?.documents() {
                self.cosmos_repository
                    .collection_client()
                    .document_client(&document.id().to_string(), &document.partition_key())?
                    .delete_document()
                    .await?;
            }
        }

        Ok(())
    }

    #[tracing::instrument(name = "Create new todo in db", skip(self, todo))]
    async fn create(&self, todo: Todo) -> anyhow::Result<()> {
        self.cosmos_repository.save(todo, false).await
    }

    #[tracing::instrument(name = "Save todo in db", skip(self, todo))]
    async fn save(&self, todo: Todo) -> anyhow::Result<()> {
        self.cosmos_repository.save(todo, true).await
    }
}
