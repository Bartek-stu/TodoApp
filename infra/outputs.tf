output "cosmos_account_name" {
  value = azurerm_cosmosdb_account.todo_app_cosmos.name
}

output "cosmos_primary_key" {
  value     = azurerm_cosmosdb_account.todo_app_cosmos.primary_key
  sensitive = true
}

output "cosmos_database_name" {
  value = azurerm_cosmosdb_sql_database.todo_app_db.name
}

output "app_insights_connection_string" {
  value     = azurerm_application_insights.todo_app_insights.connection_string
  sensitive = true
}
