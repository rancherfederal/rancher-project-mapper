use lazy_static::lazy_static;

use guest::prelude::*;
use kubewarden_policy_sdk::wapc_guest as guest;

use k8s_openapi::api::core::v1 as apicore;
use regex::Regex;
use std::collections::BTreeMap;

extern crate kubewarden_policy_sdk as kubewarden;
use kubewarden::{logging, protocol_version_guest, request::ValidationRequest, validate_settings};

mod settings;
use settings::Settings;

use slog::{o, Logger};

lazy_static! {
    static ref LOG_DRAIN: Logger = Logger::root(
        logging::KubewardenDrain::new(),
        o!("policy" => "sample-policy")
    );
}

#[no_mangle]
pub extern "C" fn wapc_init() {
    register_function("validate", validate);
    register_function("validate_settings", validate_settings::<Settings>);
    register_function("protocol_version", protocol_version_guest);
}

fn matches(match_type: &str, match_string: &str, namespace: &str) -> bool {
    match match_type {
        "all" => {
            println!("Match All");
            true
        }
        "regex" => {
            println!("Regex Comparison");
            if match_string == "*" {
                true
            } else {
                let re: Regex = Regex::new(match_string.to_string().as_str()).unwrap();
                re.is_match(namespace)
            }
        }
        "prefix" => {
            println!("Prefix Comparison");
            namespace.starts_with(match_string)
        }
        "exact" => {
            println!("Exact Comparison");
            match_string == namespace
        }
        _ => false,
    }
}

fn validate(payload: &[u8]) -> CallResult {
    let validation_request: ValidationRequest<settings::Settings> =
        ValidationRequest::new(payload)?;
    let stgs = validation_request.settings;

    match serde_json::from_value::<apicore::Namespace>(validation_request.request.object) {
        // NOTE 1
        Ok(mut namespace) => {
            let namespace_name = namespace.metadata.name.clone().unwrap_or_default();
            let metadata = namespace.metadata.clone();
            let mut new_annotations = BTreeMap::new();
            let mut new_labels = BTreeMap::new();
            for (k, v) in metadata.annotations.unwrap_or_default() {
                new_annotations.insert(k, v);
            }

            for (k, v) in metadata.labels.unwrap_or_default() {
                new_labels.insert(k, v);
            }

            for project in stgs.projects.iter() {
                if matches(
                    project.match_type.as_str(),
                    project.namespace_match.as_str(),
                    namespace_name.as_str(),
                ) {
                    new_annotations.insert(
                        String::from("field.cattle.io/projectId"),
                        format!("{}:{}", stgs.cluster_name, project.project_name),
                    );

                    new_labels.insert(
                        String::from("field.cattle.io/projectId"),
                        project.project_name.to_string(),
                    );

                    namespace.metadata.annotations = Some(new_annotations);
                    namespace.metadata.labels = Some(new_labels);

                    let mutated_object = serde_json::to_value(namespace)?;
                    return kubewarden::mutate_request(mutated_object);
                }
            }

            kubewarden::accept_request();
        }
        Err(_) => {
            // We were forwarded a request we cannot unmarshal or
            // understand, just accept it
            kubewarden::accept_request()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use settings::Project;

    use kubewarden_policy_sdk::test::Testcase;

    #[test]
    fn mutate_request_prefix_match() -> Result<(), ()> {
        let cluster_name: String = "foobar".to_string();

        let projects: Vec<Project> = vec![Project {
            match_type: "prefix".into(),
            project_name: "foobar".into(),
            namespace_match: "foo".into(),
        }];

        let request_file = "test_data/namespace-foobar.json";
        let tc = Testcase {
            name: String::from("Namespace Annotation Add"),
            fixture_file: String::from(request_file),
            expected_validation_result: true,
            settings: Settings {
                cluster_name: cluster_name,
                projects: projects,
            },
        };

        let res = tc.eval(validate).unwrap();
        assert!(
            res.mutated_object.is_some(),
            "Expected accepted object to be mutated",
        );

        Ok(())
    }

    #[test]
    fn mutate_request_regex_all_match() -> Result<(), ()> {
        let cluster_name: String = "foobar".to_string();

        let projects: Vec<Project> = vec![Project {
            match_type: "regex".into(),
            project_name: "foobar".into(),
            namespace_match: "*".into(),
        }];

        let request_file = "test_data/namespace-foobar.json";
        let tc = Testcase {
            name: String::from("Namespace Annotation Add"),
            fixture_file: String::from(request_file),
            expected_validation_result: true,
            settings: Settings {
                cluster_name: cluster_name,
                projects: projects,
            },
        };

        let res = tc.eval(validate).unwrap();
        assert!(
            res.mutated_object.is_some(),
            "Expected accepted object to be mutated",
        );

        Ok(())
    }

    #[test]
    fn mutate_request_regex_match() -> Result<(), ()> {
        let cluster_name: String = "foobar".to_string();

        let projects: Vec<Project> = vec![Project {
            match_type: "regex".into(),
            project_name: "foobar".into(),
            namespace_match: "^f[o]+".into(),
        }];

        let request_file = "test_data/namespace-foobar.json";
        let tc = Testcase {
            name: String::from("Namespace Annotation Add"),
            fixture_file: String::from(request_file),
            expected_validation_result: true,
            settings: Settings {
                cluster_name: cluster_name,
                projects: projects,
            },
        };

        let res = tc.eval(validate).unwrap();
        assert!(
            res.mutated_object.is_some(),
            "Expected accepted object to be mutated",
        );

        Ok(())
    }

    #[test]
    fn no_match_no_mutate() -> Result<(), ()> {
        let cluster_name: String = "foobar".to_string();

        let projects: Vec<Project> = vec![Project {
            match_type: "exact".into(),
            project_name: "foobar".into(),
            namespace_match: "feeber".into(),
        }];

        let request_file = "test_data/namespace-foobar.json";
        let tc = Testcase {
            name: String::from("Namespace Not Mutated"),
            fixture_file: String::from(request_file),
            expected_validation_result: true,
            settings: Settings {
                cluster_name: cluster_name,
                projects: projects,
            },
        };

        let res = tc.eval(validate).unwrap();
        assert!(
            res.mutated_object.is_none(),
            "Expected accepted object not to be mutated",
        );

        Ok(())
    }
}
