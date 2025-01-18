# -------------------------------
# 1. Create Resource Group
# -------------------------------
resource "azurerm_resource_group" "todo_app_group" {
  name     = var.resource_group_name
  location = var.location
}

# -------------------------------
# 2. Create Cosmos DB Account
# -------------------------------
resource "azurerm_cosmosdb_account" "todo_app_cosmos" {
  name                = var.cosmos_account_name
  location            = azurerm_resource_group.todo_app_group.location
  resource_group_name = azurerm_resource_group.todo_app_group.name
  offer_type          = "Standard"
  kind                = "GlobalDocumentDB"

  consistency_policy {
    consistency_level = "Session"
  }

  geo_location {
    location          = azurerm_resource_group.todo_app_group.location
    failover_priority = 0
  }
}

# -------------------------------
# 3. Create Cosmos SQL Database
# -------------------------------
resource "azurerm_cosmosdb_sql_database" "todo_app_db" {
  name                = var.cosmos_database_name
  resource_group_name = azurerm_resource_group.todo_app_group.name
  account_name        = azurerm_cosmosdb_account.todo_app_cosmos.name
}

# -------------------------------
# 4. Create "todos" Container
# -------------------------------
resource "azurerm_cosmosdb_sql_container" "todos_container" {
  name                = var.cosmos_todos_container_name
  resource_group_name = azurerm_resource_group.todo_app_group.name
  account_name        = azurerm_cosmosdb_account.todo_app_cosmos.name
  database_name       = azurerm_cosmosdb_sql_database.todo_app_db.name
  partition_key_paths = ["/created_by"]

  indexing_policy {
    indexing_mode = "consistent"

    included_path {
      path = "/*"
    }
  }
}

# -------------------------------
# 5. Create Log Analytics Workspace
# -------------------------------
resource "azurerm_log_analytics_workspace" "todo_app_logs" {
  name                = "${var.app_name}-logs"
  location            = azurerm_resource_group.todo_app_group.location
  resource_group_name = azurerm_resource_group.todo_app_group.name
  sku                 = "PerGB2018"
  retention_in_days   = 30
}

# -------------------------------
# 6. Create Application Insights
# -------------------------------
resource "azurerm_application_insights" "todo_app_insights" {
  name                = "${var.app_name}-insights"
  location            = azurerm_resource_group.todo_app_group.location
  resource_group_name = azurerm_resource_group.todo_app_group.name
  application_type    = "web"
  workspace_id        = azurerm_log_analytics_workspace.todo_app_logs.id
}

# -------------------------------
# 7. Create App Service Plan
# -------------------------------
resource "azurerm_service_plan" "todo_app_plan" {
  name                = "${var.app_name}-plan"
  location            = azurerm_resource_group.todo_app_group.location
  resource_group_name = azurerm_resource_group.todo_app_group.name

  sku_name = "S1"
  os_type  = "Linux" # or "Windows" if applicable
}

# -------------------------------
# 8. Create Linux Web App with Authentication
# -------------------------------
resource "azurerm_linux_web_app" "todo_app_service" {
  name                = var.app_service_name
  location            = azurerm_resource_group.todo_app_group.location
  resource_group_name = azurerm_resource_group.todo_app_group.name
  service_plan_id     = azurerm_service_plan.todo_app_plan.id


  app_settings = {
    "APP__TELEMETRY__APP_INSIGHTS_CONNECTION_STRING" = azurerm_application_insights.todo_app_insights.connection_string
    "APP__TELEMETRY__LOG_LEVEL"                      = "info"
    "APP__COSMOS__ACCOUNT"                           = azurerm_cosmosdb_account.todo_app_cosmos.name
    "APP__COSMOS__DATABASE_NAME"                     = azurerm_cosmosdb_sql_database.todo_app_db.name
    "APP__COSMOS__PRIMARY_KEY"                       = azurerm_cosmosdb_account.todo_app_cosmos.primary_key

    # Not used directly by the app, just to link the insights resource to the app service in the portal
    "APPINSIGHTS_INSTRUMENTATIONKEY"        = azurerm_application_insights.todo_app_insights.instrumentation_key
    "APPLICATIONINSIGHTS_CONNECTION_STRING" = azurerm_application_insights.todo_app_insights.connection_string

    "WEBSITES_PORT"                         = "8080"
    "GOOGLE_PROVIDER_AUTHENTICATION_SECRET" = var.google_provider_authentication_secret
  }

  identity {
    type = "SystemAssigned"
  }

  auth_settings_v2 {
    auth_enabled           = true
    require_authentication = true
    unauthenticated_action = "RedirectToLoginPage"
    excluded_paths         = ["/", "/static/*", "/healthcheck"]
    default_provider       = "google"

    google_v2 {
      client_id                  = "165087253946-b5lf8tfk53172mrpcsun321eq30rlodu.apps.googleusercontent.com"
      client_secret_setting_name = "GOOGLE_PROVIDER_AUTHENTICATION_SECRET"
      login_scopes               = ["openid", "profile", "email"]
    }

    login {
      token_store_enabled = true
    }
  }

  site_config {
    application_stack {
      docker_image_name   = var.app_image_name
      docker_registry_url = var.app_image_registry_url
    }
  }
}

# -------------------------------------
# 8. Action Group for Email Notification
# -------------------------------------
resource "azurerm_monitor_action_group" "todo_app_alerts" {
  name                = "todo-app-action-group"
  resource_group_name = azurerm_resource_group.todo_app_group.name
  short_name          = "todoAlerts"

  email_receiver {
    name                    = "Owner"
    email_address           = "bpekala@student.agh.edu.pl"
    use_common_alert_schema = true
  }
}

# -------------------------------
# 9. Create Availability Check
# -------------------------------
resource "azurerm_application_insights_web_test" "healthcheck_test" {
  name                    = "healthcheck-webtest"
  location                = azurerm_application_insights.todo_app_insights.location
  resource_group_name     = azurerm_resource_group.todo_app_group.name
  application_insights_id = azurerm_application_insights.todo_app_insights.id
  kind                    = "ping"
  frequency               = 300
  timeout                 = 30
  enabled                 = true
  geo_locations           = ["us-fl-mia-edge"]

  configuration = <<XML
<WebTest Name="HealthCheckTest" Id="895320c4-c04f-da77-4561-363114e1d42f" Enabled="True" CssProjectStructure="" CssIteration="" Timeout="30" WorkItemIds="" xmlns="http://microsoft.com/schemas/VisualStudio/TeamTest/2010" Description="Healthcheck endpoint test" CredentialUserName="" CredentialPassword="" PreAuthenticate="True" Proxy="default" StopOnError="False" RecordedResultFile="" ResultsLocale="">
  <Items>
    <Request Method="GET" Guid="19046cd4-4a24-033b-072f-95531c30ea28" Version="1.1" Url="https://${azurerm_linux_web_app.todo_app_service.default_hostname}/healthcheck" ThinkTime="0" Timeout="30" ParseDependentRequests="True" FollowRedirects="True" RecordResult="True" Cache="False" ResponseTimeGoal="0" Encoding="utf-8" ExpectedHttpStatusCode="200" ExpectedResponseUrl="" ReportingName="" IgnoreHttpStatusCode="False" />
  </Items>
</WebTest>
XML

  lifecycle {
    ignore_changes = [tags]
  }
}

# -------------------------------------
# 10. Alerting Rule for Availability Test
# -------------------------------------
resource "azurerm_monitor_metric_alert" "alert_availability_errors" {
  name                = "Availability-Alert"
  resource_group_name = azurerm_resource_group.todo_app_group.name
  scopes              = [azurerm_application_insights.todo_app_insights.id]
  description         = "Alert if availability test fails more than 3 times in 30 minutes"
  severity            = 2
  frequency           = "PT1M"
  window_size         = "PT30M"

  criteria {
    metric_namespace = "microsoft.insights/components"
    metric_name      = "availabilityResults/availabilityPercentage"
    aggregation      = "Average"
    operator         = "LessThan"
    threshold        = 99
  }

  action {
    action_group_id = azurerm_monitor_action_group.todo_app_alerts.id
  }
}

# -------------------------------
# 11. Alerting Rule for 5xx Errors
# -------------------------------
resource "azurerm_monitor_metric_alert" "alert_5xx_errors" {
  name                = "5xx-Error-Alert"
  resource_group_name = azurerm_resource_group.todo_app_group.name
  scopes              = [azurerm_linux_web_app.todo_app_service.id]
  description         = "Alert if any 5xx errors occur"
  severity            = 1
  frequency           = "PT1M"
  window_size         = "PT5M"

  criteria {
    metric_namespace = "Microsoft.Web/sites"
    metric_name      = "Http5xx"
    aggregation      = "Total"
    operator         = "GreaterThan"
    threshold        = 0
  }

  action {
    action_group_id = azurerm_monitor_action_group.todo_app_alerts.id
  }
}

# -------------------------------
# 10. Create Storage Account for Azure Function
# -------------------------------
resource "azurerm_storage_account" "function_app_storage" {
  name                     = "${var.app_name}funcstorage"
  resource_group_name      = azurerm_resource_group.todo_app_group.name
  location                 = azurerm_resource_group.todo_app_group.location
  account_tier             = "Standard"
  account_replication_type = "LRS"
}

# -------------------------------
# 11. Create Azure Function (Clear Completed Todos)
# -------------------------------
resource "azurerm_linux_function_app" "clear_completed_todos_function" {
  name                          = "${var.app_name}-clear-todos-func"
  location                      = azurerm_resource_group.todo_app_group.location
  resource_group_name           = azurerm_resource_group.todo_app_group.name
  service_plan_id               = azurerm_service_plan.todo_app_plan.id
  storage_account_name          = azurerm_storage_account.function_app_storage.name
  storage_account_access_key    = azurerm_storage_account.function_app_storage.primary_access_key
  public_network_access_enabled = true

  app_settings = {
    "AzureWebJobsStorage"      = azurerm_storage_account.function_app_storage.primary_blob_connection_string
    "COSMOS_DB_ACCOUNT"        = azurerm_cosmosdb_account.todo_app_cosmos.name
    "COSMOS_DB_KEY"            = azurerm_cosmosdb_account.todo_app_cosmos.primary_key
    "COSMOS_DB_NAME"           = azurerm_cosmosdb_sql_database.todo_app_db.name
    "COSMOS_CONTAINER"         = azurerm_cosmosdb_sql_container.todos_container.name
    "APPLICATIONINSIGHTS_CONNECTION_STRING"      = azurerm_application_insights.todo_app_insights.connection_string
    "APPINSIGHTS_INSTRUMENTATIONKEY"             = azurerm_application_insights.todo_app_insights.instrumentation_key
    "FUNCTIONS_WORKER_RUNTIME" = "python"
    "WEBSITE_RUN_FROM_PACKAGE" = 1
  }

  identity {
    type = "SystemAssigned"
  }

  site_config {
    application_stack {
      python_version = "3.9"
    }
    always_on = false
  }

  lifecycle {
    ignore_changes = [
      app_settings["WEBSITE_RUN_FROM_PACKAGE"],
      app_settings["WEBSITE_ENABLE_SYNC_UPDATE_SITE"]
    ]
  }
}

resource "azurerm_monitor_diagnostic_setting" "func_diag_settings" {
  name                       = "${var.app_name}-func-diagnostic"
  target_resource_id         = azurerm_linux_function_app.clear_completed_todos_function.id
  log_analytics_workspace_id = azurerm_log_analytics_workspace.todo_app_logs.id

  log {
    category = "FunctionAppLogs"
    enabled  = true
  }

  metric {
    category = "AllMetrics"
    enabled  = true
  }
}
