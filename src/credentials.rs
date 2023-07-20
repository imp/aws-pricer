use std::fmt;

use aws_credential_types::provider;
use aws_credential_types::provider::ProvideCredentials;
use aws_credential_types::Credentials;

const ACCESS_KEY_ID: &str = "AWS_ACCESS_KEY_ID";
const SECRET_ACCESS_KEY: &str = "AWS_SECRET_ACCESS_KEY";

pub struct ShuttleSecretsAwsCredentials {
    secrets: shuttle_secrets::SecretStore,
}

impl ShuttleSecretsAwsCredentials {
    pub fn new(secrets: shuttle_secrets::SecretStore) -> Self {
        Self { secrets }
    }

    fn credentials(&self) -> provider::Result {
        let access_key_id = self.secrets.get(ACCESS_KEY_ID).ok_or_else(|| {
            provider::error::CredentialsError::invalid_configuration(ACCESS_KEY_ID)
        })?;
        let secret_access_key = self.secrets.get(SECRET_ACCESS_KEY).ok_or_else(|| {
            provider::error::CredentialsError::invalid_configuration(SECRET_ACCESS_KEY)
        })?;

        Ok(Credentials::new(
            access_key_id,
            secret_access_key,
            None,
            None,
            "ShuttleSecretsAwsCredentials",
        ))
    }
}

impl ProvideCredentials for ShuttleSecretsAwsCredentials {
    fn provide_credentials<'a>(&'a self) -> provider::future::ProvideCredentials<'a>
    where
        Self: 'a,
    {
        provider::future::ProvideCredentials::ready(self.credentials())
    }
}

impl fmt::Debug for ShuttleSecretsAwsCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ShuttleSecretsAwsCredentials")
            .field("secrets", &"<SecretStore>")
            .finish()
    }
}
