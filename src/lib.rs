// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use anyhow::{anyhow, Result};

use pdk::hl::*;
use pdk::logger;

use crate::generated::config::Config;

async fn request_filter(request_state: RequestState, config: &Config) -> Flow<()> {
    let headers_state = request_state.into_headers_state().await;
    // path() returns :path pseudo-header value; Envoy includes query string, strip for comparison
    let full_path = headers_state.handler().header(":path").unwrap_or_else(|| headers_state.path());
    let path = full_path.split('?').next().unwrap_or(&full_path);
    logger::debug!("oauth-protected-resource-metadata-policy: Request path: {}", path);
    let well_known_path = config.well_known_path.as_deref().unwrap_or("/.well-known/oauth-protected-resource");

    if path.ends_with(well_known_path) {
        logger::debug!("Serving OAuth protected resource metadata for path: {path}");

        let mut metadata = serde_json::Map::new();
        metadata.insert(
            "resource".to_string(),
            serde_json::Value::String(config.resource_url.clone()),
        );

        if !config.authorization_servers.is_empty() {
            metadata.insert(
                "authorization_servers".to_string(),
                serde_json::Value::Array(
                    config.authorization_servers
                        .iter()
                        .map(|s| serde_json::Value::String(s.clone()))
                        .collect(),
                ),
            );
        }

        if let Some(ref scopes) = config.scopes_supported {
            if !scopes.is_empty() {
                metadata.insert(
                    "scopes_supported".to_string(),
                    serde_json::Value::Array(
                        scopes
                            .iter()
                            .map(|s| serde_json::Value::String(s.clone()))
                            .collect(),
                    ),
                );
            }
        }

        let body = serde_json::to_string(&serde_json::Value::Object(metadata))
            .unwrap_or_else(|_| r#"{"resource":"","error":"serialization failed"}"#.to_string());

        return Flow::Break(
            Response::new(200)
                .with_headers(vec![(
                    "Content-Type".to_string(),
                    "application/json".to_string(),
                )])
                .with_body(body),
        );
    }

    Flow::Continue(())
}

#[entrypoint]
async fn configure(launcher: Launcher, Configuration(bytes): Configuration) -> Result<()> {
    let config: Config = serde_json::from_slice(&bytes).map_err(|err| {
        anyhow!(
            "Failed to parse configuration '{}'. Cause: {}",
            String::from_utf8_lossy(&bytes),
            err
        )
    })?;
    let filter = on_request(|rs| request_filter(rs, &config));
    launcher.launch(filter).await?;
    Ok(())
}
