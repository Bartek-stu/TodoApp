variable "resource_group_name" {
  description = "Name of the Azure Resource Group"
  default     = "TodoAppGroup"
}

variable "location" {
  description = "Azure region"
  default     = "polandcentral"
}

variable "app_name" {
  description = "Base name for the app resources"
  default     = "todoapp"
}

variable "app_service_name" {
  description = "Azure App Service name"
  default     = "todoapp2137-service"
}

variable "app_image_name" {
  description = "Docker image name"
  default     = "andrut01/todo_app:latest"
}

variable "app_image_registry_url" {
  description = "URL of Docker Registry"
  default     = "https://index.docker.io"
}

variable "cosmos_account_name" {
  description = "Cosmos DB account name"
  default     = "todoappcosmosacc"
}

variable "cosmos_database_name" {
  description = "Cosmos DB database name"
  default     = "todoappdb"
}

variable "cosmos_todos_container_name" {
  description = "Name of the todos container"
  default     = "todos"
}

variable "google_provider_authentication_secret" {
  description = "Google provider authentication secret"
  type        = string
}
