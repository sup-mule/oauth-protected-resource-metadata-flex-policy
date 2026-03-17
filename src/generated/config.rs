use serde::Deserialize;
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(alias = "authorizationServers")]
    pub authorization_servers: Vec<String>,
    #[serde(alias = "resourceUrl")]
    pub resource_url: String,
    #[serde(alias = "scopesSupported")]
    pub scopes_supported: Option<Vec<String>>,
    #[serde(alias = "wellKnownPath")]
    pub well_known_path: Option<String>,
}
