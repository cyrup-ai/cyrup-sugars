//! Builder pattern primitives and utilities
//!
//! This module provides reusable components for creating fluent typestate builders
//! that leverage all cyrup_sugars features seamlessly.
pub mod chunk_handler;
pub mod llm;
pub use chunk_handler::*;
pub use llm::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap as StdHashMap;
use sugars_collections::{OneOrMany, ZeroOneOrMany};

/// Re-export hashbrown for builder convenience
pub use hashbrown::HashMap;

/// Trait for building configuration objects with validation
pub trait ConfigBuilder<T> {
    /// The error type returned when building fails.
    type Error;

    /// Build the final configuration
    fn build(self) -> Result<T, Self::Error>;

    /// Validate the current state
    fn validate(&self) -> Result<(), Self::Error>;
}

/// Trait for JSON serializable configurations
pub trait JsonConfig: Serialize + for<'de> Deserialize<'de> {
    /// Serialize to pretty JSON string
    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON string
    fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Automatic implementation for all Serialize + Deserialize types
impl<T> JsonConfig for T where T: Serialize + for<'de> Deserialize<'de> {}

/// Helper trait for creating object literal syntax
pub trait ObjectLiteral<K, V> {
    /// Create from key-value pairs
    fn from_pairs<I: IntoIterator<Item = (K, V)>>(pairs: I) -> Self;

    /// Create empty object
    fn empty() -> Self;
}

impl<K, V> ObjectLiteral<K, V> for HashMap<K, V>
where
    K: std::hash::Hash + Eq,
{
    fn from_pairs<I: IntoIterator<Item = (K, V)>>(pairs: I) -> Self {
        pairs.into_iter().collect()
    }

    fn empty() -> Self {
        HashMap::new()
    }
}

impl<K, V> ObjectLiteral<K, V> for StdHashMap<K, V>
where
    K: std::hash::Hash + Eq,
{
    fn from_pairs<I: IntoIterator<Item = (K, V)>>(pairs: I) -> Self {
        pairs.into_iter().collect()
    }

    fn empty() -> Self {
        StdHashMap::new()
    }
}

/// Builder state management
pub mod state {
    use std::marker::PhantomData;

    /// Marker trait for builder states
    pub trait BuilderState {}

    /// Builder is incomplete and missing required fields
    pub struct Incomplete;
    impl BuilderState for Incomplete {}

    /// Builder has all required fields
    pub struct Complete;
    impl BuilderState for Complete {}

    /// Builder with custom validation state
    pub struct Validated<T>(PhantomData<T>);
    impl<T> BuilderState for Validated<T> {}

    /// State transition helpers
    pub trait StateTransition<To: BuilderState> {
        /// The output type after transition.
        type Output;
        /// Performs the state transition.
        fn transition(self) -> Self::Output;
    }
}

/// Common configuration patterns
pub mod patterns {
    use super::*;

    /// Authentication configuration pattern
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct AuthConfig {
        /// Authentication methods (e.g., "jwt", "oauth").
        pub methods: OneOrMany<String>,
        /// Token time-to-live in seconds.
        pub token_ttl: u64,
        /// Authentication providers (e.g., "google", "github").
        pub providers: ZeroOneOrMany<String>,
        /// Additional provider-specific settings.
        pub settings: HashMap<String, serde_json::Value>,
    }

    impl AuthConfig {
        /// Creates a JWT authentication configuration.
        pub fn jwt(secret: &str) -> Self {
            Self {
                methods: OneOrMany::one("jwt".to_string()),
                token_ttl: 3600,
                providers: ZeroOneOrMany::none(),
                settings: {
                    let mut settings = HashMap::new();
                    settings.insert("secret".to_string(), secret.into());
                    settings.insert("algorithm".to_string(), "HS256".into());
                    settings
                },
            }
        }

        /// Creates an OAuth authentication configuration.
        pub fn oauth<P: Into<ZeroOneOrMany<String>>>(providers: P) -> Self {
            Self {
                methods: OneOrMany::many(vec!["oauth".to_string(), "jwt".to_string()])
                    .unwrap_or_else(|_| OneOrMany::one("oauth".to_string())),
                token_ttl: 7200,
                providers: providers.into(),
                settings: HashMap::new(),
            }
        }
    }

    /// Rate limiting configuration pattern
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct RateLimitConfig {
        /// Maximum requests per minute.
        pub requests_per_minute: u32,
        /// Burst size for rate limiting.
        pub burst_size: u32,
        /// Paths to exclude from rate limiting.
        pub exclude_paths: ZeroOneOrMany<String>,
        /// Custom rate limit rules per path.
        pub custom_rules: HashMap<String, u32>,
    }

    impl RateLimitConfig {
        /// Creates a simple rate limit configuration.
        pub fn simple(rpm: u32) -> Self {
            Self {
                requests_per_minute: rpm,
                burst_size: rpm / 10,
                exclude_paths: ZeroOneOrMany::many(vec![
                    "/health".to_string(),
                    "/metrics".to_string(),
                ]),
                custom_rules: HashMap::new(),
            }
        }

        /// Creates a rate limit configuration with custom rules.
        pub fn with_rules(rpm: u32, rules: HashMap<String, u32>) -> Self {
            Self {
                requests_per_minute: rpm,
                burst_size: rpm / 10,
                exclude_paths: ZeroOneOrMany::many(vec![
                    "/health".to_string(),
                    "/metrics".to_string(),
                ]),
                custom_rules: rules,
            }
        }
    }

    /// CORS configuration pattern
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct CorsConfig {
        /// Allowed origins for CORS requests.
        pub allowed_origins: ZeroOneOrMany<String>,
        /// Allowed HTTP methods.
        pub allowed_methods: OneOrMany<String>,
        /// Allowed headers in requests.
        pub allowed_headers: ZeroOneOrMany<String>,
        /// Maximum age for preflight cache.
        pub max_age: u64,
        /// Whether to allow credentials.
        pub credentials: bool,
    }

    impl CorsConfig {
        /// Creates a permissive CORS configuration.
        pub fn permissive() -> Self {
            Self {
                allowed_origins: ZeroOneOrMany::one("*".to_string()),
                allowed_methods: OneOrMany::many(vec![
                    "GET".to_string(),
                    "POST".to_string(),
                    "PUT".to_string(),
                    "DELETE".to_string(),
                    "OPTIONS".to_string(),
                ])
                .unwrap_or_else(|_| OneOrMany::one("GET".to_string())),
                allowed_headers: ZeroOneOrMany::one("*".to_string()),
                max_age: 86400,
                credentials: false,
            }
        }

        /// Creates a strict CORS configuration with specific allowed values.
        pub fn strict<O, M, H>(origins: O, methods: M, headers: H) -> Self
        where
            O: Into<ZeroOneOrMany<String>>,
            M: Into<OneOrMany<String>>,
            H: Into<ZeroOneOrMany<String>>,
        {
            Self {
                allowed_origins: origins.into(),
                allowed_methods: methods.into(),
                allowed_headers: headers.into(),
                max_age: 86400,
                credentials: true,
            }
        }
    }
}

/// Async builder support
pub mod async_support {
    use sugars_async_task::{AsyncTask, NotResult};

    /// Trait for builders that can deploy/execute asynchronously
    pub trait AsyncBuilder<T>
    where
        T: NotResult,
    {
        /// The error type returned when async building fails.
        type Error;

        /// Execute the builder asynchronously
        fn execute(self) -> AsyncTask<T>;

        /// Execute with validation
        fn execute_validated(self) -> AsyncTask<T>
        where
            Self: Sized,
        {
            self.execute()
        }
    }
}

/// Macro helpers for object literal syntax
pub mod macros {
    /// Create HashMap with object literal syntax
    #[macro_export]
    macro_rules! object {
        () => {
            $crate::builders::HashMap::new()
        };
        ($($key:expr => $value:expr),+ $(,)?) => {
            {
                let mut map = $crate::builders::HashMap::new();
                $(
                    map.insert($key, $value);
                )+
                map
            }
        };
    }

    /// Create configuration builder with fluent syntax
    #[macro_export]
    macro_rules! config_builder {
        ($builder_type:ty) => {
            <$builder_type>::new()
        };
        ($builder_type:ty, $($method:ident($($arg:expr),*)),+ $(,)?) => {
            {
                let builder = <$builder_type>::new();
                $(
                    let builder = builder.$method($($arg),*);
                )+
                builder
            }
        };
    }

    pub use config_builder;
    pub use object;
}

/// Feature-gated closure macros for builders
pub mod closure_macros {
    // Re-export closure macros for builder usage
    // Macros are automatically re-exported by macro_export attribute

    /// Builder validation macro
    #[macro_export]
    macro_rules! validate_config {
        ($config:expr, $($field:ident => $validation:expr),+ $(,)?) => {
            {
                let mut errors = Vec::new();
                $(
                    if !($validation) {
                        errors.push(format!("Validation failed for field: {}", stringify!($field)));
                    }
                )+
                if errors.is_empty() {
                    Ok($config)
                } else {
                    Err(errors.join(", "))
                }
            }
        };
    }

    pub use validate_config;
}

pub use patterns::*;
/// Re-export everything for convenience
pub use state::*;

pub use async_support::*;
