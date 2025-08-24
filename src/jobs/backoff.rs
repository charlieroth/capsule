use rand::Rng;
use std::time::Duration;

/// Calculate exponential backoff delay with jitter
pub fn calculate_backoff_delay(attempt: i32, base_delay_secs: u32) -> Duration {
    let attempt = attempt.max(0) as u32;

    // Cap the exponent to prevent overflow (max ~8.5 hours with 30s base)
    let capped_attempt = attempt.min(10);

    // Calculate base delay: base * 2^attempt
    let base_delay = base_delay_secs.saturating_mul(2_u32.saturating_pow(capped_attempt));

    // Add jitter: ±30% randomness
    let jitter_factor = rand::thread_rng().gen_range(0.7..1.3);
    let delay_with_jitter = (base_delay as f64 * jitter_factor).round() as u64;

    Duration::from_secs(delay_with_jitter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff_progression() {
        let base_delay = 30;

        // Test first few attempts have reasonable progression
        let delay0 = calculate_backoff_delay(0, base_delay);
        let delay1 = calculate_backoff_delay(1, base_delay);
        let delay2 = calculate_backoff_delay(2, base_delay);

        // Delays should be in expected ranges with jitter
        assert!(delay0.as_secs() >= 21 && delay0.as_secs() <= 39); // 30s ±30%
        assert!(delay1.as_secs() >= 42 && delay1.as_secs() <= 78); // 60s ±30% 
        assert!(delay2.as_secs() >= 84 && delay2.as_secs() <= 156); // 120s ±30%
    }

    #[test]
    fn test_backoff_cap() {
        let base_delay = 30;

        // Very high attempt numbers should be capped at attempt 10
        let delay_high = calculate_backoff_delay(20, base_delay);
        let delay_capped = calculate_backoff_delay(10, base_delay);

        // Both should be within reasonable bounds (max delay with jitter should be similar)
        // At attempt 10: base * 2^10 = 30 * 1024 = 30720s with jitter 0.7-1.3 = ~21k-40k
        assert!(delay_high.as_secs() >= 21000 && delay_high.as_secs() <= 40000);
        assert!(delay_capped.as_secs() >= 21000 && delay_capped.as_secs() <= 40000);
    }

    #[test]
    fn test_negative_attempt() {
        let delay = calculate_backoff_delay(-5, 30);
        // Should handle negative attempts gracefully
        assert!(delay.as_secs() >= 21 && delay.as_secs() <= 39);
    }
}
