/// Stellar SEP-10 challenge-response authentication service.
pub mod auth {
    use anyhow::Result;

    /// Build a SEP-10 challenge transaction for the given account.
    pub fn build_challenge(account: &str, home_domain: &str, network_passphrase: &str) -> Result<String> {
        // TODO: Use stellar-sdk to build the challenge XDR
        todo!("implement SEP-10 challenge construction")
    }

    /// Verify a signed SEP-10 challenge and extract the account.
    pub fn verify_challenge(signed_xdr: &str, network_passphrase: &str) -> Result<String> {
        // TODO: Decode XDR, verify signatures, extract account
        todo!("implement SEP-10 verification")
    }
}

/// Index price calculation service.
pub mod index_calc {
    use anyhow::Result;

    /// Calculate the weighted index value from multiple data source values.
    pub fn weighted_average(values: &[(f64, f64)]) -> f64 {
        // values: (price, confidence)
        let total_weight: f64 = values.iter().map(|(_, c)| c).sum();
        if total_weight == 0.0 {
            return 0.0;
        }
        values.iter().map(|(v, c)| v * c).sum::<f64>() / total_weight
    }

    /// Remove outliers beyond N standard deviations.
    pub fn remove_outliers(points: &[f64], stddev_threshold: f64) -> Vec<f64> {
        if points.len() <= 2 {
            return points.to_vec();
        }

        let mean: f64 = points.iter().sum::<f64>() / points.len() as f64;
        let variance: f64 = points.iter().map(|p| (p - mean).powi(2)).sum::<f64>() / points.len() as f64;
        let stddev = variance.sqrt();

        points
            .iter()
            .filter(|p| (*p - mean).abs() <= stddev * stddev_threshold)
            .copied()
            .collect()
    }
}

/// Position health factor calculation.
pub mod position_health {
    /// Calculate health factor for a position.
    ///
    /// - `direction`: "long" or "short"
    /// - `collateral`: USDC deposited
    /// - `size`: position size (collateral * leverage)
    /// - `entry_price`: index value at open
    /// - `current_price`: current index value
    ///
    /// Returns health factor in basis points (10000 = 1.0x).
    pub fn calculate(
        direction: &str,
        collateral: f64,
        size: f64,
        entry_price: f64,
        current_price: f64,
    ) -> f64 {
        if collateral == 0.0 || entry_price == 0.0 {
            return 0.0;
        }

        let pnl = match direction {
            "long" => size * (current_price - entry_price) / entry_price,
            "short" => size * (entry_price - current_price) / entry_price,
            _ => 0.0,
        };

        let equity = collateral + pnl;
        let maintenance_required = collateral * 0.8; // 80% maintenance ratio

        if maintenance_required == 0.0 {
            return f64::MAX;
        }

        equity / maintenance_required
    }

    /// Calculate liquidation price for a position.
    pub fn liquidation_price(direction: &str, entry_price: f64, leverage: u32) -> f64 {
        let maintenance_ratio = 0.8;
        match direction {
            "long" => entry_price * (1.0 - 1.0 / leverage as f64 * maintenance_ratio),
            "short" => entry_price * (1.0 + 1.0 / leverage as f64 * maintenance_ratio),
            _ => 0.0,
        }
    }
}
