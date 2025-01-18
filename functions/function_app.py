import datetime
import os
import logging

import azure.functions as func
from azure.cosmos import CosmosClient

app = func.FunctionApp()

@app.function_name(name="mytimer")
@app.timer_trigger(schedule="0 */20 * * * *", arg_name="mytimer", run_on_startup=True,
              use_monitor=False)
def cleanup_function(mytimer: func.TimerRequest) -> None:
    utc_now = datetime.datetime.utcnow()
    logging.info(f"Clear completed todos function ran at {utc_now}")

    endpoint = f"https://{os.getenv('COSMOS_DB_ACCOUNT')}.documents.azure.com:443/"
    key = os.getenv('COSMOS_DB_KEY')
    database_name = os.getenv('COSMOS_DB_NAME')
    container_name = os.getenv('COSMOS_CONTAINER')

    client = CosmosClient(endpoint, credential=key)
    database = client.get_database_client(database_name)
    container = database.get_container_client(container_name)

    seven_days_ago = (utc_now - datetime.timedelta(days=7)).isoformat()

    query = """
    SELECT c.id, c.created_by
    FROM c
    WHERE c.done = true AND c.created_at < @seven_days_ago
    """

    items = container.query_items(
        query=query,
        parameters=[{"name": "@seven_days_ago", "value": seven_days_ago}],
        enable_cross_partition_query=True
    )

    deleted_todos = 0
    for item in items:
        container.delete_item(item=item["id"], partition_key=item["created_by"])
        logging.info(f"Deleted todo with ID: {item['id']} and Partition Key: {item['created_by']}")
        deleted_todos += 1

    logging.info(f"Clear completed todos function finished. {deleted_todos} records deleted.")
