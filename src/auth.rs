use crate::config::Config;

pub struct AuthProvider {
    config: Config,
}

impl AuthProvider {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn authenticate(&self, username: &str) -> bool {
        log::info!("Authentication attempt for user: {}", username);
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