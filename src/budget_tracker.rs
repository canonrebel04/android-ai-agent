/// Budget tracker: counts tokens per task, estimates cost by tier,
/// and fires alerts when the monthly budget threshold is exceeded.
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

/// Token pricing per 1K tokens (input / output).
#[derive(Debug, Clone)]
pub struct TierPricing {
    pub input_per_1k: f64,
    pub output_per_1k: f64,
}

impl TierPricing {
    pub fn trivial() -> Self {
        Self {
            input_per_1k: 0.00015,
            output_per_1k: 0.0006,
        }
    }
    pub fn standard() -> Self {
        Self {
            input_per_1k: 0.005,
            output_per_1k: 0.015,
        }
    }
    pub fn complex() -> Self {
        Self {
            input_per_1k: 0.01,
            output_per_1k: 0.03,
        }
    }
    pub fn critical() -> Self {
        Self {
            input_per_1k: 0.015,
            output_per_1k: 0.075,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    Trivial,
    Standard,
    Complex,
    Critical,
}

impl Tier {
    pub fn pricing(&self) -> TierPricing {
        match self {
            Tier::Trivial => TierPricing::trivial(),
            Tier::Standard => TierPricing::standard(),
            Tier::Complex => TierPricing::complex(),
            Tier::Critical => TierPricing::critical(),
        }
    }
}

pub struct BudgetTracker {
    monthly_input_tokens: AtomicU64,
    monthly_output_tokens: AtomicU64,
    monthly_cost: Mutex<f64>,
    budget_threshold: Mutex<f64>,
}

impl BudgetTracker {
    pub fn new() -> Self {
        Self {
            monthly_input_tokens: AtomicU64::new(0),
            monthly_output_tokens: AtomicU64::new(0),
            monthly_cost: Mutex::new(0.0),
            budget_threshold: Mutex::new(5.0),
        }
    }

    pub fn record_usage(&self, tier: Tier, input_tokens: u64, output_tokens: u64) {
        let pricing = tier.pricing();
        let cost = (input_tokens as f64 / 1000.0) * pricing.input_per_1k
            + (output_tokens as f64 / 1000.0) * pricing.output_per_1k;

        self.monthly_input_tokens
            .fetch_add(input_tokens, Ordering::SeqCst);
        self.monthly_output_tokens
            .fetch_add(output_tokens, Ordering::SeqCst);
        *self.monthly_cost.lock().unwrap() += cost;
    }

    pub fn monthly_cost(&self) -> f64 {
        *self.monthly_cost.lock().unwrap()
    }

    pub fn monthly_input_tokens(&self) -> u64 {
        self.monthly_input_tokens.load(Ordering::SeqCst)
    }

    pub fn monthly_output_tokens(&self) -> u64 {
        self.monthly_output_tokens.load(Ordering::SeqCst)
    }

    pub fn set_threshold(&self, usd: f64) {
        *self.budget_threshold.lock().unwrap() = usd;
    }

    pub fn is_over_budget(&self) -> bool {
        let cost = self.monthly_cost();
        let threshold = *self.budget_threshold.lock().unwrap();
        if threshold <= 0.0 {
            return false;
        }
        cost > threshold
    }

    pub fn threshold(&self) -> f64 {
        *self.budget_threshold.lock().unwrap()
    }

    pub fn reset_month(&self) {
        self.monthly_input_tokens.store(0, Ordering::SeqCst);
        self.monthly_output_tokens.store(0, Ordering::SeqCst);
        *self.monthly_cost.lock().unwrap() = 0.0;
    }
}

/// Global budget tracker singleton for JNI access.
use std::sync::OnceLock;
static TRACKER: OnceLock<BudgetTracker> = OnceLock::new();

pub fn get_tracker() -> &'static BudgetTracker {
    TRACKER.get_or_init(BudgetTracker::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_usage_increases_cost() {
        let bt = BudgetTracker::new();
        assert_eq!(bt.monthly_cost(), 0.0);
        bt.record_usage(Tier::Standard, 5000, 1000);
        // 5k input * 0.005/1k = 0.025 + 1k output * 0.015/1k = 0.015 = 0.04
        assert!((bt.monthly_cost() - 0.04).abs() < 0.001);
    }

    #[test]
    fn test_budget_threshold_exceeded() {
        let bt = BudgetTracker::new();
        bt.set_threshold(0.01);
        assert!(!bt.is_over_budget());
        bt.record_usage(Tier::Standard, 5000, 1000); // $0.04
        assert!(bt.is_over_budget());
    }

    #[test]
    fn test_zero_threshold_never_over_budget() {
        let bt = BudgetTracker::new();
        bt.set_threshold(0.0);
        bt.record_usage(Tier::Critical, 100_000, 10_000);
        assert!(!bt.is_over_budget());
    }

    #[test]
    fn test_reset_month() {
        let bt = BudgetTracker::new();
        bt.record_usage(Tier::Standard, 5000, 1000);
        assert!(bt.monthly_cost() > 0.0);
        bt.reset_month();
        assert_eq!(bt.monthly_cost(), 0.0);
        assert_eq!(bt.monthly_input_tokens(), 0);
    }

    #[test]
    fn test_token_counting() {
        let bt = BudgetTracker::new();
        bt.record_usage(Tier::Trivial, 100, 50);
        bt.record_usage(Tier::Trivial, 200, 100);
        assert_eq!(bt.monthly_input_tokens(), 300);
        assert_eq!(bt.monthly_output_tokens(), 150);
    }
}
