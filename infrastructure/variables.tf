variable "name_prefix" {
  type        = string
  description = "Base prefix for resource names (letters and numbers only)."
}

variable "environment" {
  type        = string
  description = "Environment name used in resource naming."
  default     = "dev"
}

variable "location" {
  type        = string
  description = "Azure region for all resources."
  default     = "westeurope"
}

variable "plan_sku" {
  type        = string
  description = "SKU for the function app service plan (e.g., EP1 for Elastic Premium)."
  default     = "EP1"
}

variable "acr_sku" {
  type        = string
  description = "Azure Container Registry SKU."
  default     = "Basic"
}

variable "container_image_name" {
  type        = string
  description = "Container image repository name stored in ACR."
  default     = "restful-rust"
}

variable "container_image_tag" {
  type        = string
  description = "Container image tag."
  default     = "latest"
}

variable "container_port" {
  type        = string
  description = "Port exposed by the container image."
  default     = "8080"
}

variable "tags" {
  type        = map(string)
  description = "Optional tags applied to all resources."
  default     = {}
}

