use lazy_static::lazy_static;

use guest::prelude::*;
use kubewarden_policy_sdk::wapc_guest as guest;

use k8s_openapi::api::core::v1 as apicore;
use std::{collections::BTreeMap, string};
use regex::Regex;

extern crate kubewarden_policy_sdk as kubewarden;
use kubewarden::{logging, protocol_version_guest, request::ValidationRequest, validate_settings};

mod settings;

use slog::{info, o, warn, Logger};

lazy_static! {
    static ref LOG_DRAIN: Logger = Logger::root(
        logging::KubewardenDrain::new(),
        o!("policy" => "sample-policy")
    );
}

#[no_mangle]
pub extern "C" fn wapc_init() {
    // register_function("validate", validate);
    // register_function("validate_settings", validate_settings::<Settings>);
    register_function("protocol_version", protocol_version_guest);
}


fn matches(match_type: &str, match_string: &str, namespace: &str) -> bool {
    match match_type {
        "regex" => {
            println!("Regex Comparison");
            let re = Regex::new(format!(r#"{}"#, regex::escape(match_string)).as_str()).unwrap();
            if re.is_match(namespace) {
                return true;
            }
        },
        "prefix" => {
            println!("Prefix Comparison");
            if namespace.starts_with(match_string) {
                return true;
            }
        },
        "exact" => {
            println!("Exact Comparison");
            if match_string == namespace {
                return true;
            }
        },
        _=> {
            return false;
        }
    }
    return false;
}

fn validate(stgs: settings::Settings, payload: &[u8]) -> CallResult {
    let validation_request: ValidationRequest<settings::Settings> = ValidationRequest::new(payload)?;

    match serde_json::from_value::<apicore::Namespace>(validation_request.request.object) {
        // NOTE 1
        Ok(mut namespace) => {
            let namespace_name = namespace.metadata.name.clone().unwrap_or_default();
                    let metadata = namespace.metadata.clone();
                    let mut new_annotations = BTreeMap::new();
                    let mut new_labels = BTreeMap::new();
                    for (k,v) in metadata.annotations.unwrap_or_default() {
                        new_annotations.insert(k, v);
                    }

                    for (k,v) in metadata.labels.unwrap_or_default() {
                        new_labels.insert(k, v);
                    }

            for project in stgs.projects.iter() {
                if matches(project.project_type.as_str(), project.project_name.as_str(), namespace_name.as_str()) {
                    new_annotations.insert(
                        String::from("field.cattle.io/projectId"),
                        format!("{}:{}", stgs.cluster_name, project.project_name),
                    );
                    
                    new_labels.insert(
                        String::from("field.cattle.io/projectId"),
                        format!("{}", project.project_name),
                    );
                    
                    namespace.metadata.annotations = Some(new_annotations);
                    namespace.metadata.labels = Some(new_labels);

                    let mutated_object = serde_json::to_value(namespace)?;
                    kubewarden::mutate_request(mutated_object);
                }
                break;
            }
            
            return kubewarden::accept_request()
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

    use kubewarden_policy_sdk::test::Testcase;
    use std::collections::HashMap;

    // #[test]
    // fn deny_request_with_istio_disabled_namespace() -> Result<(), ()> {
    //     let excluded_namespaces = vec!["bar".to_string()];

    //     let excluded_pod_labels =
    //         HashMap::from([("istioException".to_string(), "enabled".to_string())]);

    //     let request_file = "test_data/namespace-disabled.json";
    //     let tc = Testcase {
    //         name: String::from("Namespace Creation"),
    //         fixture_file: String::from(request_file),
    //         expected_validation_result: false,
    //         settings: Settings {
    //             excluded_namespaces: excluded_namespaces,
    //             excluded_pod_labels: excluded_pod_labels,
    //         },
    //     };

    //     let res = tc.eval(validate).unwrap();
    //     assert!(
    //         res.mutated_object.is_none(),
    //         "Something mutated with test case: {}",
    //         tc.name,
    //     );

    //     Ok(())
    // }

    // #[test]
    // fn accept_request_with_excluded_namespace() -> Result<(), ()> {
    //     let excluded_namespaces = vec!["foo".to_string()];

    //     let excluded_pod_labels =
    //         HashMap::from([("istioException".to_string(), "enabled".to_string())]);

    //     let request_file = "test_data/namespace-disabled.json";
    //     let tc = Testcase {
    //         name: String::from("Namespace Creation"),
    //         fixture_file: String::from(request_file),
    //         expected_validation_result: true,
    //         settings: Settings {
    //             excluded_namespaces: excluded_namespaces,
    //             excluded_pod_labels: excluded_pod_labels,
    //         },
    //     };

    //     let res = tc.eval(validate).unwrap();
    //     assert!(
    //         res.mutated_object.is_none(),
    //         "Something mutated with test case: {}",
    //         tc.name,
    //     );

    //     Ok(())
    // }

    // #[test]
    // fn deny_request_with_istio_disabled_pod() -> Result<(), ()> {
    //     let excluded_namespaces = vec!["foo".to_string()];

    //     let excluded_pod_labels =
    //         HashMap::from([("istioException".to_string(), "disabled".to_string())]);

    //     let request_file = "test_data/pod-disabled.json";
    //     let tc = Testcase {
    //         name: String::from("Deny - Pod Istio Disabled"),
    //         fixture_file: String::from(request_file),
    //         expected_validation_result: false,
    //         settings: Settings {
    //             excluded_namespaces: excluded_namespaces,
    //             excluded_pod_labels: excluded_pod_labels,
    //         },
    //     };

    //     let res = tc.eval(validate).unwrap();
    //     assert!(
    //         res.mutated_object.is_none(),
    //         "Something mutated with test case: {}",
    //         tc.name,
    //     );

    //     Ok(())
    // }

    // #[test]
    // fn accept_request_with_istio_enabled_pod() -> Result<(), ()> {
    //     let excluded_namespaces = vec!["foo".to_string()];

    //     let excluded_pod_labels =
    //         HashMap::from([("istioException".to_string(), "disabled".to_string())]);

    //     let request_file = "test_data/pod-enabled.json";
    //     let tc = Testcase {
    //         name: String::from("Pod Creation"),
    //         fixture_file: String::from(request_file),
    //         expected_validation_result: true,
    //         settings: Settings {
    //             excluded_namespaces: excluded_namespaces,
    //             excluded_pod_labels: excluded_pod_labels,
    //         },
    //     };

    //     let res = tc.eval(validate).unwrap();
    //     assert!(
    //         res.mutated_object.is_none(),
    //         "Something mutated with test case: {}",
    //         tc.name,
    //     );

    //     Ok(())
    // }
}
