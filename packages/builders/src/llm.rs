//! AI Agent Builder implementations using cyrup_sugars array tuple syntax
//!
//! This module implements the exact syntax shown in the README.md file,
//! demonstrating clean array tuple configuration without exposing macros.

use serde::{Deserialize, Serialize};
use std::collections::HashMap as StdHashMap;

/// Client builder that supports array tuple syntax
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    headers: StdHashMap<String, String>,
    options: StdHashMap<String, String>,
}

impl Client {
    /// Create a new client
    pub fn new() -> Self {
        Self {
            headers: StdHashMap::new(),
            options: StdHashMap::new(),
        }
    }

    /// Set headers using array tuple syntax
    #[cfg(feature = "array-tuples")]
    pub fn with_headers<F>(mut self, f: F) -> Self
    where
        F: FnOnce() -> hashbrown::HashMap<&'static str, &'static str>,
    {
        let headers_map = f();
        for (k, v) in headers_map {
            self.headers.insert(k.to_string(), v.to_string());
        }
        self
    }

    /// Set options using JSON object syntax  
    #[cfg(feature = "array-tuples")]
    pub fn with_options<F>(mut self, f: F) -> Self
    where
        F: FnOnce() -> hashbrown::HashMap<&'static str, &'static str>,
    {
        let options_map = f();
        for (k, v) in options_map {
            self.options.insert(k.to_string(), v.to_string());
        }
        self
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

/// Database connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    pub host: String,
    pub port: String,
    pub database: String,
    pub user: String,
    pub password: Option<String>,
}

impl Database {
    /// Connect to database using JSON object syntax
    #[cfg(feature = "array-tuples")]
    pub fn connect<F>(f: F) -> Self
    where
        F: FnOnce() -> hashbrown::HashMap<&'static str, &'static str>,
    {
        let config = f();
        Self {
            host: config.get("host").unwrap_or(&"localhost").to_string(),
            port: config.get("port").unwrap_or(&"5432").to_string(),
            database: config.get("database").unwrap_or(&"postgres").to_string(),
            user: config.get("user").unwrap_or(&"postgres").to_string(),
            password: config.get("password").map(|s| s.to_string()),
        }
    }
}

/// API client builder with fluent interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiClient {
    endpoint: Option<String>,
    auth: Option<StdHashMap<String, String>>,
    rate_limit: Option<StdHashMap<String, String>>,
}

impl ApiClient {
    /// Create a new API client builder
    pub fn new() -> Self {
        Self {
            endpoint: None,
            auth: None,
            rate_limit: None,
        }
    }

    /// Set the API endpoint
    pub fn endpoint<S: Into<String>>(mut self, url: S) -> Self {
        self.endpoint = Some(url.into());
        self
    }

    /// Set authentication using JSON object syntax
    #[cfg(feature = "array-tuples")]
    pub fn auth<F>(mut self, f: F) -> Self
    where
        F: FnOnce() -> hashbrown::HashMap<&'static str, &'static str>,
    {
        let auth_map = f();
        let mut auth = StdHashMap::new();
        for (k, v) in auth_map {
            auth.insert(k.to_string(), v.to_string());
        }
        self.auth = Some(auth);
        self
    }

    /// Set rate limiting using JSON object syntax
    #[cfg(feature = "array-tuples")]
    pub fn rate_limit<F>(mut self, f: F) -> Self
    where
        F: FnOnce() -> hashbrown::HashMap<&'static str, &'static str>,
    {
        let limit_map = f();
        let mut rate_limit = StdHashMap::new();
        for (k, v) in limit_map {
            rate_limit.insert(k.to_string(), v.to_string());
        }
        self.rate_limit = Some(rate_limit);
        self
    }

    /// Build the final API client
    pub fn build(self) -> Result<BuiltApiClient, String> {
        let endpoint = self.endpoint.ok_or("endpoint is required")?;
        Ok(BuiltApiClient {
            endpoint,
            auth: self.auth.unwrap_or_default(),
            rate_limit: self.rate_limit.unwrap_or_default(),
        })
    }
}

impl Default for ApiClient {
    fn default() -> Self {
        Self::new()
    }
}

/// A built and configured API client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltApiClient {
    pub endpoint: String,
    pub auth: StdHashMap<String, String>,
    pub rate_limit: StdHashMap<String, String>,
}
