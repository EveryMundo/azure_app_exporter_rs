- [Overview](#overview)
- [Example metrics](#example-metrics)
- [Configuration](#configuration)
- [Running the exporter](#running-the-exporter)
- [Using the exporter](#using-the-exporter)
- [How it works](#how-it-works)
- [Metrics exposed by the exporter](#metrics-exposed-by-the-exporter)

# Overview

Expose Prometheus metrics for expiring Azure password credentials. Useful for alerting on a password credential approaching its expiration time.

# Example metrics

```
azure_application_password_remaining_seconds{id="...",app_id="...",app_display_name="",password_key_id="...",password_display_name="",password_end_date_time="2024-01-06 14:43:01 UTC"} 175406
azure_application_password_remaining_seconds{id="...",app_id="...",app_display_name="DATAPLATFORM-PROD",password_key_id="...",password_display_name="Display name",password_end_date_time="2024-07-20 12:58:28 UTC"} 17103533
```

# Configuration

See [./settings_example.toml](./settings_example.toml). There are no command line arguments except `-h / --help` and `show-settings-example`.

# Running the exporter

Create a service principal in Azure with a client secret and the permission `Application.Read.All`. This permission is required because the exporter needs to fetch all applications registered for a given tenant to see the expiration dates for the password credentials assigned to them. Follow this guide for the details <https://learn.microsoft.com/en-us/graph/auth-register-app-v2>.

Copy the `settings_example.toml` to the machine that will host the exporter (usually to `/etc/azure_app_exporter/settings.toml`), and fill in the `[credentials]` header with your `tenant_id`, `client_id` and `client_secret`. These 3 settings are the minimum configuration required. All remaining settings that are not explicitly provided will use the default values shown in the settings example.

Run the exporter after providing a path to the settings file in an env var like so `AZURE_APP_EXPORTER_SETTINGS_PATH=/path/to/settings.toml ./azure_app_exporter`. If the env var is not provided the exporter will try to open `/etc/azure_app_exporter/settings.toml` by default.

After running the exporter, wait a couple of seconds until it creates a token and fetches the applications. View its json line logs on stdout for more info.

# Using the exporter

Once the exporter is up and running, you can interact with it from the following endpoints

- `/metrics` - see the remaining seconds for each password credential among other metrics
- `/api/apps` - show all applications cached in memory
- `/api/apps/:id` - lookup a cached application by its ID
- `/swagger` - interactive API documentation powered by Swagger UI. Allows you to see available endpoints and try them out from your browser. This endpoint can be changed in the settings
- `/openapi.json` - OpenAPI documentation. This endpoint can be changed in the settings

See the Swagger UI for more documentation about each endpoint.

# How it works

After starting the exporter, it first makes a request to `https://login.microsoftonline.com/{tenant_id}/oauth2/v2.0/token` with your `tenant_id`, `client_id` and `client_secret`. It will then get an access token valid for 1 hour which will be cached in memory and used in future requests. This token is automatically refreshed approximately every 54 minutes (90% of the token's validity duration).

After the access token is acquired, the exporter will make a request to `https://graph.microsoft.com/v1.0/applications?$top=999&$select=id,appId,displayName,createdDateTime,passwordCredentials` with the token in an `Authorization: Bearer ...` header. The applications in the response will be cached in memory and automatically refreshed every 15 minutes by default.

Finally, the exporter will automatically update the metrics exported on the `/metrics` endpoint every 1 minute by default.

# Metrics exposed by the exporter

- `azure_app_exporter_azure_api_token_update_duration_seconds` - How many seconds it takes to update the Azure API token
- `azure_app_exporter_azure_applications_update_duration_seconds` - How many seconds it takes to update the in-memory cache of Azure applications
- `azure_app_exporter_azure_application_password_remaining_seconds` - Seconds remaining until the password credential expires
- `azure_app_exporter_requests_total` - Number of HTTP requests processed, partitioned by HTTP method, host, path and status code
- `azure_app_exporter_request_duration_seconds` - The HTTP request latencies in seconds
- `azure_app_exporter_request_size_bytes` - The HTTP request sizes in bytes
- `azure_app_exporter_response_size_bytes` - The HTTP response sizes in bytes
