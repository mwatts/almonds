use std::io;

use async_trait::async_trait;
use bcrypt::{hash, verify, DEFAULT_COST};

use crate::errors::service_error::ServiceError;

#[derive(Clone)]
pub struct ServiceHelpers {}

impl ServiceHelpers {
    pub fn init() -> Self {
        Self {}
    }
}

#[async_trait]
pub trait ServiceHelpersTrait {
    fn hash_password(&self, raw_password: &str) -> Result<String, ServiceError>;
    fn validate_password(&self, raw_password: &str, hash: &str) -> Result<bool, ServiceError>;
    fn delete_file_if_exists(path: &str) -> io::Result<()>;
}

#[async_trait]
impl ServiceHelpersTrait for ServiceHelpers {
    fn hash_password(&self, raw_password: &str) -> Result<String, ServiceError> {
        hash(raw_password, DEFAULT_COST).map_err(|_| ServiceError::OperationFailed)
    }

    fn validate_password(&self, password: &str, hash: &str) -> Result<bool, ServiceError> {
        verify(password, hash).map_err(|_| ServiceError::OperationFailed)
    }

    fn delete_file_if_exists(path: &str) -> io::Result<()> {
        let p = std::path::Path::new(path);
        if p.exists() {
            std::fs::remove_file(p)?;
        }
        Ok(())
    }
}
