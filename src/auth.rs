use crate::config::Config;

pub struct AuthProvider {
    config: Config,
}

impl AuthProvider {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn authenticate(&self, username: &str) -> bool {
        log::info!("Authentication attempt for user: {username}");
        username == self.config.mysql_username
    }

    pub fn default_auth_plugin(&self) -> &str {
        "mysql_native_password"
    }

    pub fn generate_salt(&self) -> [u8; 20] {
        let bs = ";X,po_k}o6^Wz!/kM}Na".as_bytes();
        let mut scramble: [u8; 20] = [0; 20];
        for i in 0..20 {
            scramble[i] = bs[i];
            if scramble[i] == b'\0' || scramble[i] == b'$' {
                scramble[i] += 1;
            }
        }
        scramble
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        Config {
            db_host: "localhost".to_string(),
            db_user: "postgres".to_string(),
            db_password: "password".to_string(),
            db_name: "postgres".to_string(),
            mysql_username: "testuser".to_string(),
            mysql_password: "testpass".to_string(),
            bind_address: "0.0.0.0:3306".to_string(),
        }
    }

    #[test]
    fn test_auth_provider_new() {
        let config = create_test_config();
        let auth_provider = AuthProvider::new(config.clone());

        assert_eq!(auth_provider.config.mysql_username, "testuser");
        assert_eq!(auth_provider.config.mysql_password, "testpass");
    }

    #[test]
    fn test_authenticate_success() {
        let config = create_test_config();
        let auth_provider = AuthProvider::new(config);

        let result = auth_provider.authenticate("testuser");
        assert!(result);
    }

    #[test]
    fn test_authenticate_failure() {
        let config = create_test_config();
        let auth_provider = AuthProvider::new(config);

        let result = auth_provider.authenticate("wronguser");
        assert!(!result);
    }

    #[test]
    fn test_authenticate_empty_username() {
        let config = create_test_config();
        let auth_provider = AuthProvider::new(config);

        let result = auth_provider.authenticate("");
        assert!(!result);
    }

    #[test]
    fn test_default_auth_plugin() {
        let config = create_test_config();
        let auth_provider = AuthProvider::new(config);

        assert_eq!(auth_provider.default_auth_plugin(), "mysql_native_password");
    }

    #[test]
    fn test_generate_salt() {
        let config = create_test_config();
        let auth_provider = AuthProvider::new(config);

        let salt = auth_provider.generate_salt();
        assert_eq!(salt.len(), 20);

        // Test that salt is deterministic (same salt each time)
        let salt2 = auth_provider.generate_salt();
        assert_eq!(salt, salt2);
    }

    #[test]
    fn test_generate_salt_no_null_or_dollar() {
        let config = create_test_config();
        let auth_provider = AuthProvider::new(config);

        let salt = auth_provider.generate_salt();

        // Ensure no null bytes or dollar signs in salt
        for &byte in &salt {
            assert_ne!(byte, b'\0');
            assert_ne!(byte, b'$');
        }
    }
}
