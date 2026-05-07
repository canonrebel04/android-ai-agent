use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Model pricing information in USD per 1K tokens
#[derive(Debug, Clone, PartialEq)]
pub struct ModelPricing {
    pub input_price: f64,
    pub output_price: f64,
}

/// Static pricing data for popular models
/// Prices are in USD per 1K tokens
static PRICING_DATA: Lazy<HashMap<String, ModelPricing>> = Lazy::new(|| {
    let mut m = HashMap::new();
    
    // Google Gemini
    m.insert(
        "gemini-flash-2.5".to_string(),
        ModelPricing {
            input_price: 0.00001,
            output_price: 0.00003,
        },
    );
    
    // Mistral
    m.insert(
        "mistral-small-3.2".to_string(),
        ModelPricing {
            input_price: 0.0000025,
            output_price: 0.0000085,
        },
    );
    
    // Claude Sonnet
    m.insert(
        "claude-sonnet-4-6".to_string(),
        ModelPricing {
            input_price: 0.000003,
            output_price: 0.000015,
        },
    );
    
    // Claude Opus
    m.insert(
        "claude-opus-4-6".to_string(),
        ModelPricing {
            input_price: 0.000015,
            output_price: 0.000075,
        },
    );
    
    // Local models (free)
    m.insert(
        "local".to_string(),
        ModelPricing {
            input_price: 0.0,
            output_price: 0.0,
        },
    );
    
    m
});

/// Get pricing for a specific model
/// 
/// # Arguments
/// * `model_id` - The model identifier (e.g., "claude-sonnet-4-6")
/// 
/// # Returns
/// Some(ModelPricing) if the model exists, None otherwise
pub fn get_pricing(model_id: &str) -> Option<ModelPricing> {
    PRICING_DATA.get(model_id).cloned()
}

/// Get all pricing data
/// 
/// # Returns
/// A reference to the complete HashMap of model pricing
pub fn get_all_pricing() -> &'static HashMap<String, ModelPricing> {
    &PRICING_DATA
}

/// Format a price as a USD string
/// 
/// # Arguments
/// * `price` - The price in USD
/// 
/// # Returns
/// Formatted string like "$0.000010" or "$0.00"
pub fn format_price(price: f64) -> String {
    if price == 0.0 {
        "$0.00".to_string()
    } else {
        format!("${:.6}", price)
    }
}

/// Estimate the cost for a given number of input and output tokens
/// 
/// # Arguments
/// * `model_id` - The model identifier
/// * `input_tokens` - Number of input tokens (in thousands)
/// * `output_tokens` - Number of output tokens (in thousands)
/// 
/// # Returns
/// Some(total_cost) if the model exists, None otherwise
pub fn estimate_cost(model_id: &str, input_tokens: f64, output_tokens: f64) -> Option<f64> {
    get_pricing(model_id).map(|pricing| {
        (pricing.input_price * input_tokens) + (pricing.output_price * output_tokens)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_pricing_known_model() {
        let pricing = get_pricing("claude-sonnet-4-6");
        assert!(pricing.is_some());
        let pricing = pricing.unwrap();
        assert_eq!(pricing.input_price, 0.000003);
        assert_eq!(pricing.output_price, 0.000015);
    }

    #[test]
    fn test_get_pricing_unknown_model() {
        let pricing = get_pricing("nonexistent-model");
        assert!(pricing.is_none());
    }

    #[test]
    fn test_get_all_pricing() {
        let all = get_all_pricing();
        assert!(all.contains_key("gemini-flash-2.5"));
        assert!(all.contains_key("mistral-small-3.2"));
        assert!(all.contains_key("claude-sonnet-4-6"));
        assert!(all.contains_key("claude-opus-4-6"));
        assert!(all.contains_key("local"));
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn test_format_price() {
        assert_eq!(format_price(0.000003), "$0.000003");
        assert_eq!(format_price(0.0), "$0.00");
        assert_eq!(format_price(0.000015), "$0.000015");
    }

    #[test]
    fn test_estimate_cost() {
        // Test with claude-sonnet-4-6: 1000 input tokens + 500 output tokens
        // Input: 1.0 * 0.000003 = 0.000003
        // Output: 0.5 * 0.000015 = 0.0000075
        // Total: 0.0000105
        let cost = estimate_cost("claude-sonnet-4-6", 1.0, 0.5);
        assert!(cost.is_some());
        assert!((cost.unwrap() - 0.0000105).abs() < f64::EPSILON);
        
        // Test with local model (should be free)
        let cost = estimate_cost("local", 100.0, 100.0);
        assert!(cost.is_some());
        assert_eq!(cost.unwrap(), 0.0);
    }
}
