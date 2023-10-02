use crate::LOG_DRAIN;

use serde::{Deserialize, Serialize};
use slog::info;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Project {
    pub match_type: String,
    pub project_name: String,
    pub namespace_match: String,
}

// Describe the settings your policy expects when
// loaded by the policy server.
#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)]
pub(crate) struct Settings {
    pub cluster_name: String,
    pub projects: Vec<Project>,
}

impl kubewarden::settings::Validatable for Settings {
    fn validate(&self) -> Result<(), String> {
        info!(LOG_DRAIN, "starting settings validation");

        // TODO: perform settings validation if applies
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use kubewarden_policy_sdk::settings::Validatable;

    #[test]
    fn validate_settings() -> Result<(), ()> {
        let cluster_name: String = "foobar".to_string();

        let projects: Vec<Project> = vec![Project {
            match_type: "exact".into(),
            project_name: "foobar".into(),
            namespace_match: "foobar".into(),
        }];

        let settings = Settings {
            cluster_name: cluster_name.into(),
            projects: projects,
        };

        assert!(settings.validate().is_ok());
        Ok(())
    }
}
