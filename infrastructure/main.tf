terraform {
  required_version = ">= 1.6.0"

  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 3.100"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.6"
    }
  }
}

provider "azurerm" {
  features {}
}

locals {
  prefix       = lower(regexreplace(var.name_prefix, "[^a-z0-9]", ""))
  environment  = lower(regexreplace(var.environment, "[^a-z0-9]", ""))
  base_name    = "${local.prefix}-${local.environment}"
  uniq         = random_string.suffix.result

  resource_group_name     = "${local.base_name}-rg"
  service_plan_name       = "${local.base_name}-plan"
  function_app_name       = "${local.base_name}-func"
  container_registry_name = substr("${local.prefix}${local.uniq}", 0, 50)
  storage_account_name    = substr("${local.prefix}${local.uniq}", 0, 24)

  tags = merge(var.tags, {
    environment = local.environment
  })
}

resource "random_string" "suffix" {
  length  = 6
  upper   = false
  lower   = true
  number  = true
  special = false
}

resource "azurerm_resource_group" "app" {
  name     = local.resource_group_name
  location = var.location
  tags     = local.tags
}

resource "azurerm_storage_account" "functions" {
  name                     = local.storage_account_name
  resource_group_name      = azurerm_resource_group.app.name
  location                 = azurerm_resource_group.app.location
  account_tier             = "Standard"
  account_replication_type = "LRS"
  min_tls_version          = "TLS1_2"
  allow_nested_items_to_be_public = false
  tags                     = local.tags
}

resource "azurerm_service_plan" "functions" {
  name                = local.service_plan_name
  resource_group_name = azurerm_resource_group.app.name
  location            = azurerm_resource_group.app.location
  os_type             = "Linux"
  sku_name            = var.plan_sku
  tags                = local.tags
}

resource "azurerm_container_registry" "acr" {
  name                = local.container_registry_name
  resource_group_name = azurerm_resource_group.app.name
  location            = azurerm_resource_group.app.location
  sku                 = var.acr_sku
  admin_enabled       = false
  tags                = local.tags
}

resource "azurerm_linux_function_app" "app" {
  name                       = local.function_app_name
  resource_group_name        = azurerm_resource_group.app.name
  location                   = azurerm_resource_group.app.location
  service_plan_id            = azurerm_service_plan.functions.id
  storage_account_name       = azurerm_storage_account.functions.name
  storage_account_access_key = azurerm_storage_account.functions.primary_access_key
  functions_extension_version = "~4"
  https_only                 = true
  container_registry_use_managed_identity = true

  identity {
    type = "SystemAssigned"
  }

  site_config {
    always_on = true

    application_stack {
      docker {
        registry_url = azurerm_container_registry.acr.login_server
        image_name   = var.container_image_name
        image_tag    = var.container_image_tag
      }
    }
  }

  app_settings = {
    FUNCTIONS_WORKER_RUNTIME          = "custom"
    WEBSITES_ENABLE_APP_SERVICE_STORAGE = "false"
    WEBSITES_PORT                     = var.container_port
    DOCKER_ENABLE_CI                  = "true"
    DOCKER_REGISTRY_SERVER_URL        = "https://${azurerm_container_registry.acr.login_server}"
    FUNCTIONS_EXTENSION_VERSION       = "~4"
  }

  tags = local.tags
}

resource "azurerm_role_assignment" "acr_pull" {
  principal_id         = azurerm_linux_function_app.app.identity[0].principal_id
  role_definition_name = "AcrPull"
  scope                = azurerm_container_registry.acr.id
}

output "function_app_hostname" {
  description = "Default hostname of the Azure Function."
  value       = azurerm_linux_function_app.app.default_hostname
}

output "resource_group" {
  description = "Resource group name for the deployment."
  value       = azurerm_resource_group.app.name
}

output "container_registry_login_server" {
  description = "Login server for pushing images."
  value       = azurerm_container_registry.acr.login_server
}

