# Azure Functions Terraform

Terraform configuration for running the `restful-rust` container as a custom-handler Azure Function on Linux.

## What gets created
- Resource group, storage account, and Elastic Premium Linux plan sized by `plan_sku`.
- Azure Container Registry (ACR) for image storage with system-assigned managed identity and `AcrPull` role for the function.
- Linux Function App configured for a custom container, listening on `container_port` (defaults to 8080).

## Usage
1) Copy and edit variables:
```bash
cp terraform.tfvars.example terraform.tfvars
# edit terraform.tfvars to set name_prefix, environment, and any tags
```
`name_prefix` should be lowercase letters/numbers only; Terraform adds a random suffix where needed for globally-unique resources like ACR and storage.

2) Deploy:
```bash
terraform init
terraform plan
terraform apply
```

3) Build and push the container image to ACR (after `terraform apply` provides the registry name):
```bash
az acr login --name <acr_name>
docker build -t <acr_name>.azurecr.io/restful-rust:latest ..
docker push <acr_name>.azurecr.io/restful-rust:latest
```
If you push a new tag, update `container_image_tag` and re-apply Terraform or change the setting directly in the Function App.

## Notes
- The Function App uses `FUNCTIONS_WORKER_RUNTIME=custom` with `WEBSITES_ENABLE_APP_SERVICE_STORAGE=false`, suitable for custom containers.
- Plan SKU defaults to `EP1` (Elastic Premium). You can change `plan_sku` to other Premium or Dedicated SKUs if needed.
- All resources inherit tags provided in `tags` along with the `environment` tag.

