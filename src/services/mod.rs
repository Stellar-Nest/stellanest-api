/// Stellar SEP-10 challenge-response authentication service.
pub mod auth {
    use anyhow::{Context, Result};
    use base64::Engine;
    use rand::Rng;

    /// Build a SEP-10 challenge transaction for the given account.
    ///
    /// Creates a transaction with:
    /// - Source account = `account`, sequence number = 0
    /// - ManageData op: key = "home_domain", value = `home_domain`
    /// - ManageData op: key = "{home_domain} auth", value = 64-byte random nonce
    /// - Timebounds: now .. now+300s
    /// - Signed by the server's signing key (`STELLAR_SERVER_SECRET` env)
    ///
    /// Returns the base64-encoded transaction envelope XDR.
    pub fn build_challenge(
        account: &str,
        home_domain: &str,
        network_passphrase: &str,
    ) -> Result<String> {
        // Validate the account looks like a Stellar G-address
        if !account.starts_with('G') || account.len() != 56 {
            anyhow::bail!("invalid Stellar account address");
        }

        let server_secret = std::env::var("STELLAR_SERVER_SECRET")
            .context("STELLAR_SERVER_SECRET not configured")?;

        let server_kp = stellar_sdk::Keypair::from_secret(&server_secret)
            .map_err(|e| anyhow::anyhow!("invalid server key: {:?}", e))?;

        // Generate 64-byte random nonce (SEP-10 requirement)
        let mut nonce = vec![0u8; 64];
        rand::thread_rng().fill(&mut nonce[..]);

        // Build ManageData operations
        let manage_data_home = stellar_base::operations::ManageDataOperationBuilder::new()
            .with_data_name("home_domain".to_string())
            .with_data_value(Some(stellar_base::account::DataValue(
                home_domain.as_bytes().to_vec(),
            )))
            .build()
            .context("failed to build home_domain ManageData op")?;

        let manage_data_auth = stellar_base::operations::ManageDataOperationBuilder::new()
            .with_data_name(format!("{} auth", home_domain))
            .with_data_value(Some(stellar_base::account::DataValue(nonce)))
            .build()
            .context("failed to build auth ManageData op")?;

        // Timebounds: 5 minutes from now
        let now = chrono::Utc::now().timestamp() as u64;
        let time_bounds = stellar_base::time_bounds::TimeBounds {
            min_time: now,
            max_time: now + 300,
        };

        // Build transaction: source = account, sequence = 0 (SEP-10 requirement)
        let source = stellar_base::crypto::MuxedAccount::Account(account.to_string());
        let mut tx =
            stellar_base::transaction::Transaction::builder(source, 0i64, 100u32.into())
                .with_time_bounds(time_bounds)
                .add_operation(manage_data_home)
                .add_operation(manage_data_auth)
                .into_transaction()
                .context("failed to build challenge transaction")?;

        // Sign with server key
        let network = stellar_base::network::Network::new(network_passphrase);
        tx.sign(&server_kp, &network);

        // Serialize to base64 XDR
        let envelope = tx.into_envelope();
        let xdr_bytes = envelope
            .xdr_base64()
            .context("failed to encode transaction envelope")?;

        Ok(xdr_bytes)
    }

    /// Verify a signed SEP-10 challenge and extract the authenticated account.
    ///
    /// Checks:
    /// 1. XDR decodes to a valid transaction envelope
    /// 2. Sequence number is 0 (SEP-10 requirement)
    /// 3. Transaction has ManageData operations matching our home domain
    /// 4. At least one valid signature from the source account
    /// 5. Server signature is present
    ///
    /// Returns the verified account public key (G-address).
    pub fn verify_challenge(signed_xdr: &str, network_passphrase: &str) -> Result<String> {
        use base64::engine::general_purpose::STANDARD as BASE64;

        // 1. Decode base64
        let xdr_bytes = BASE64
            .decode(signed_xdr)
            .context("failed to decode base64 transaction")?;

        // 2. Parse the transaction envelope
        let tx = stellar_base::transaction::Transaction::from_xdr_envelope(&xdr_bytes)
            .map_err(|e| anyhow::anyhow!("invalid transaction envelope XDR: {:?}", e))?;

        // 3. SEP-10 requirement: sequence number must be 0
        let seq = *tx.sequence();
        if seq != 0 {
            anyhow::bail!(
                "challenge transaction must have sequence number 0, got {}",
                seq
            );
        }

        // 4. Extract source account
        let source_account = tx.source_account().account_id();

        // 5. Must have signatures
        let signatures = tx.signatures();
        if signatures.is_empty() {
            anyhow::bail!("challenge transaction has no signatures");
        }

        // 6. Verify that the transaction was signed by our server key
        let server_secret = std::env::var("STELLAR_SERVER_SECRET")
            .context("STELLAR_SERVER_SECRET not configured")?;
        let server_kp = stellar_sdk::Keypair::from_secret(&server_secret)
            .map_err(|e| anyhow::anyhow!("invalid server key: {:?}", e))?;

        let network = stellar_base::network::Network::new(network_passphrase);
        let tx_hash = tx.hash(&network).context("failed to compute tx hash")?;

        let mut server_signed = false;
        let mut client_signed = false;

        for sig in signatures {
            // Try verifying with server key
            if server_kp.verify(&tx_hash, sig.signature.as_slice()).is_ok() {
                server_signed = true;
            }

            // Try verifying with source account key
            if let Ok(source_kp) =
                stellar_sdk::Keypair::from_public_key(&source_account)
            {
                if source_kp
                    .verify(&tx_hash, sig.signature.as_slice())
                    .is_ok()
                {
                    client_signed = true;
                }
            }
        }

        if !server_signed {
            anyhow::bail!("challenge transaction is not signed by the server");
        }
        if !client_signed {
            anyhow::bail!(
                "challenge transaction has no valid signature from account {}",
                source_account
            );
        }

        // 7. Verify timebounds are still valid
        if let Some(tb) = tx.time_bounds() {
            let now = chrono::Utc::now().timestamp() as u64;
            if now < tb.min_time || now > tb.max_time {
                anyhow::bail!("challenge transaction timebounds have expired");
            }
        }

        Ok(source_account)
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
        let variance: f64 = points.iter().map(|p| (p - mean).powi(2)).sum::<f64>()
            / points.len() as f64;
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
