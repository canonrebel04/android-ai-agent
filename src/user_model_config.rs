use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

/// Configuration for a user's model settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserModelConfig {
    /// Type of provider (e.g., OpenRouter, Anthropic, Google, Mistral, DeepSeek, Local)
    pub provider_type: String,
    /// The model identifier
    pub model_id: String,
    /// API key for authentication
    /// WARNING: This field contains sensitive information. Do not log, print, or expose this value.
    /// Consider using a secure secret management system in production.
    pub api_key: String,
    /// Maximum number of tokens to generate (must be > 0)
    pub max_tokens: u32,
    /// Temperature for sampling (must be between 0.0 and 2.0)
    pub temperature: f32,
}

/// Error type for UserModelConfig validation
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    InvalidTemperature(f32),
    InvalidMaxTokens(u32),
    MissingField(String),
    SerializationError(String),
    DeserializationError(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::InvalidTemperature(temp) => {
                write!(f, "Invalid temperature: {}. Must be between 0.0 and 2.0", temp)
            }
            ConfigError::InvalidMaxTokens(tokens) => {
                write!(f, "Invalid max_tokens: {}. Must be greater than 0", tokens)
            }
            ConfigError::MissingField(field) => {
                write!(f, "Missing required field: {}", field)
            }
            ConfigError::SerializationError(msg) => {
                write!(f, "Serialization error: {}", msg)
            }
            ConfigError::DeserializationError(msg) => {
                write!(f, "Deserialization error: {}", msg)
            }
        }
    }
}

impl Error for ConfigError {}

impl UserModelConfig {
    /// Creates a new UserModelConfig with validation
    pub fn new(
        provider_type: String,
        model_id: String,
        api_key: String,
        max_tokens: u32,
        temperature: f32,
    ) -> Result<Self, ConfigError> {
        if temperature < 0.0 || temperature > 2.0 {
            return Err(ConfigError::InvalidTemperature(temperature));
        }
        if max_tokens == 0 {
            return Err(ConfigError::InvalidMaxTokens(max_tokens));
        }
        Ok(Self {
            provider_type,
            model_id,
            api_key,
            max_tokens,
            temperature,
        })
    }

    /// Validates the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.temperature < 0.0 || self.temperature > 2.0 {
            return Err(ConfigError::InvalidTemperature(self.temperature));
        }
        if self.max_tokens == 0 {
            return Err(ConfigError::InvalidMaxTokens(self.max_tokens));
        }
        Ok(())
    }

    /// Serializes the config to JSON string
    pub fn to_json(&self) -> Result<String, ConfigError> {
        serde_json::to_string(self)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))
    }

    /// Deserializes a UserModelConfig from JSON string
    pub fn from_json(json: &str) -> Result<Self, ConfigError> {
        let config: Self = serde_json::from_str(json)
            .map_err(|e| ConfigError::DeserializationError(e.to_string()))?;
        config.validate()?;
        Ok(config)
    }
}

impl Default for UserModelConfig {
    fn default() -> Self {
        Self {
            provider_type: "OpenRouter".to_string(),
            model_id: "openai/gpt-3.5-turbo".to_string(),
            api_key: "".to_string(),
            max_tokens: 1024,
            temperature: 0.7,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let config = UserModelConfig::default();
        assert_eq!(config.provider_type, "OpenRouter");
        assert_eq!(config.model_id, "openai/gpt-3.5-turbo");
        assert_eq!(config.api_key, "");
        assert_eq!(config.max_tokens, 1024);
        assert_eq!(config.temperature, 0.7);
    }

    #[test]
    fn test_custom_config() {
        let config = UserModelConfig {
            provider_type: "Anthropic".to_string(),
            model_id: "claude-3-sonnet".to_string(),
            api_key: "test-key-123".to_string(),
            max_tokens: 2048,
            temperature: 0.9,
        };
        assert_eq!(config.provider_type, "Anthropic");
        assert_eq!(config.model_id, "claude-3-sonnet");
        assert_eq!(config.api_key, "test-key-123");
        assert_eq!(config.max_tokens, 2048);
        assert_eq!(config.temperature, 0.9);
    }

    #[test]
    fn test_serialization() {
        let config = UserModelConfig {
            provider_type: "Mistral".to_string(),
            model_id: "mistral-tiny".to_string(),
            api_key: "mistral-key".to_string(),
            max_tokens: 512,
            temperature: 0.5,
        };
        let serialized = config.to_json();
        assert!(serialized.is_ok());
        let serialized_str = serialized.expect("Serialization should succeed");
        assert!(serialized_str.contains("Mistral"));
        assert!(serialized_str.contains("mistral-tiny"));
        assert!(serialized_str.contains("mistral-key"));
        assert!(serialized_str.contains("512"));
        assert!(serialized_str.contains("0.5"));
    }

    #[test]
    fn test_deserialization() {
        let json = r#"{
            "provider_type": "Google",
            "model_id": "gemini-pro",
            "api_key": "google-key",
            "max_tokens": 4096,
            "temperature": 0.8
        }"#;
        let config = UserModelConfig::from_json(json);
        assert!(config.is_ok());
        let config = config.expect("Deserialization should succeed");
        assert_eq!(config.provider_type, "Google");
        assert_eq!(config.model_id, "gemini-pro");
        assert_eq!(config.api_key, "google-key");
        assert_eq!(config.max_tokens, 4096);
        assert_eq!(config.temperature, 0.8);
    }

    #[test]
    fn test_roundtrip() {
        let original = UserModelConfig {
            provider_type: "DeepSeek".to_string(),
            model_id: "deepseek-chat".to_string(),
            api_key: "deepseek-key".to_string(),
            max_tokens: 3072,
            temperature: 0.3,
        };
        let serialized = original.to_json();
        assert!(serialized.is_ok());
        let serialized_str = serialized.expect("Serialization should succeed");
        let deserialized = UserModelConfig::from_json(&serialized_str);
        assert!(deserialized.is_ok());
        assert_eq!(original, deserialized.expect("Deserialization should succeed"));
    }

    #[test]
    fn test_local_provider() {
        let config = UserModelConfig {
            provider_type: "Local".to_string(),
            model_id: "llama-3-8b".to_string(),
            api_key: "".to_string(),
            max_tokens: 8192,
            temperature: 0.1,
        };
        assert_eq!(config.provider_type, "Local");
        assert_eq!(config.model_id, "llama-3-8b");
        assert_eq!(config.api_key, "");
        assert_eq!(config.max_tokens, 8192);
        assert_eq!(config.temperature, 0.1);
    }

    // New tests for validation

    #[test]
    fn test_invalid_temperature_too_low() {
        let result = UserModelConfig::new(
            "OpenRouter".to_string(),
            "openai/gpt-3.5-turbo".to_string(),
            "key".to_string(),
            100,
            -0.1,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ConfigError::InvalidTemperature(-0.1));
    }

    #[test]
    fn test_invalid_temperature_too_high() {
        let result = UserModelConfig::new(
            "OpenRouter".to_string(),
            "openai/gpt-3.5-turbo".to_string(),
            "key".to_string(),
            100,
            2.1,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ConfigError::InvalidTemperature(2.1));
    }

    #[test]
    fn test_invalid_temperature_at_boundaries() {
        // Test at exact boundaries (0.0 and 2.0 should be valid)
        let result_low = UserModelConfig::new(
            "OpenRouter".to_string(),
            "openai/gpt-3.5-turbo".to_string(),
            "key".to_string(),
            100,
            0.0,
        );
        assert!(result_low.is_ok());

        let result_high = UserModelConfig::new(
            "OpenRouter".to_string(),
            "openai/gpt-3.5-turbo".to_string(),
            "key".to_string(),
            100,
            2.0,
        );
        assert!(result_high.is_ok());
    }

    #[test]
    fn test_invalid_max_tokens_zero() {
        let result = UserModelConfig::new(
            "OpenRouter".to_string(),
            "openai/gpt-3.5-turbo".to_string(),
            "key".to_string(),
            0,
            0.5,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ConfigError::InvalidMaxTokens(0));
    }

    #[test]
    fn test_valid_max_tokens_one() {
        let result = UserModelConfig::new(
            "OpenRouter".to_string(),
            "openai/gpt-3.5-turbo".to_string(),
            "key".to_string(),
            1,
            0.5,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_missing_field_deserialization() {
        // Test with missing max_tokens field
        let json = r#"{
            "provider_type": "Google",
            "model_id": "gemini-pro",
            "api_key": "google-key",
            "temperature": 0.8
        }"#;
        let result = UserModelConfig::from_json(json);
        assert!(result.is_err());
        // serde will fail to deserialize due to missing field
        match result.unwrap_err() {
            ConfigError::DeserializationError(_) => {}
            _ => panic!("Expected DeserializationError"),
        }
    }

    #[test]
    fn test_missing_field_deserialization_missing_temp() {
        // Test with missing temperature field
        let json = r#"{
            "provider_type": "Google",
            "model_id": "gemini-pro",
            "api_key": "google-key",
            "max_tokens": 100
        }"#;
        let result = UserModelConfig::from_json(json);
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::DeserializationError(_) => {}
            _ => panic!("Expected DeserializationError"),
        }
    }

    #[test]
    fn test_validate_method() {
        let valid_config = UserModelConfig::default();
        assert!(valid_config.validate().is_ok());

        let mut invalid_temp_config = UserModelConfig::default();
        invalid_temp_config.temperature = 2.5;
        assert!(invalid_temp_config.validate().is_err());
        assert_eq!(
            invalid_temp_config.validate().unwrap_err(),
            ConfigError::InvalidTemperature(2.5)
        );

        let mut invalid_tokens_config = UserModelConfig::default();
        invalid_tokens_config.max_tokens = 0;
        assert!(invalid_tokens_config.validate().is_err());
        assert_eq!(
            invalid_tokens_config.validate().unwrap_err(),
            ConfigError::InvalidMaxTokens(0)
        );
    }

    #[test]
    fn test_config_error_display() {
        let err = ConfigError::InvalidTemperature(3.0);
        assert!(err.to_string().contains("Invalid temperature"));
        assert!(err.to_string().contains("3"));

        let err = ConfigError::InvalidMaxTokens(0);
        assert!(err.to_string().contains("Invalid max_tokens"));
        assert!(err.to_string().contains("0"));

        let err = ConfigError::MissingField("api_key".to_string());
        assert!(err.to_string().contains("Missing required field"));
        assert!(err.to_string().contains("api_key"));
    }
}
