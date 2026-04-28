//! JSON-from-kind dispatcher for `Problem` impls.
//!
//! This module wires up [`problem_from_json`], which reconstructs a
//! `Box<dyn Problem>` from the `(kind, json)` pair produced by
//! `Problem::kind()` + `Problem::json()`. Individual `Problem` impls
//! register themselves with the dispatcher via the
//! [`register_problem_de!`](crate::register_problem_de) /
//! [`register_problem_de_fn!`](crate::register_problem_de_fn) macros,
//! colocated with their `inventory::submit! { ProblemKindInfo {...} }`
//! block in the file that defines the impl.
//!
//! The set covered in-tree is curated for round-trip parity with
//! `Problem::json()` for the kinds the Janitor's followup
//! orchestration cares about — primarily the dependency-related
//! ones that map to `ognibuild::buildlog::problem_to_dependency`.

use crate::Problem;

/// Function pointer that turns a JSON details payload back into a
/// `Box<dyn Problem>`. Registered alongside [`crate::ProblemKindInfo`] so
/// callers can reconstruct a typed problem from
/// `(kind: &str, details: &serde_json::Value)` — the shape the
/// Janitor stores in `run.failure_details` and the Python original
/// reconstructed via `problem_clses[kind].from_json(details)`.
pub type ProblemFromJsonFn = fn(&serde_json::Value) -> Result<Box<dyn Problem>, serde_json::Error>;

/// Pair of `(kind, deserializer)` used to power
/// [`problem_from_json`]. Each `Problem` impl that wants to be
/// reconstructable from JSON registers one of these via
/// `inventory::submit!`. The crate ships registrations for the
/// problem types we know how to round-trip; out-of-tree
/// `Problem` impls can register their own.
///
/// The `register_problem_de!` macro generates this entry from a
/// `serde::Deserialize` impl on the target struct.
pub struct ProblemDeserializer {
    /// The problem kind identifier (matches `ProblemKindInfo.kind`).
    pub kind: &'static str,
    /// Function that takes the JSON details and produces a typed
    /// `Box<dyn Problem>` (or a `serde_json::Error` if the shape
    /// doesn't match).
    pub from_json: ProblemFromJsonFn,
}

inventory::collect!(ProblemDeserializer);

/// Reconstruct a `Box<dyn Problem>` from a `(kind, details)`
/// pair. Returns `None` when no deserializer is registered for
/// `kind`, or when the registered deserializer rejects the JSON
/// shape (the inner error is surfaced via `log::warn!`).
///
/// Mirrors Python's `problem_clses[kind].from_json(details)`.
pub fn problem_from_json(kind: &str, details: &serde_json::Value) -> Option<Box<dyn Problem>> {
    for entry in inventory::iter::<ProblemDeserializer> {
        if entry.kind == kind {
            match (entry.from_json)(details) {
                Ok(p) => return Some(p),
                Err(e) => {
                    log::warn!(
                        "buildlog-consultant: failed to deserialize problem of kind {:?}: {}",
                        kind,
                        e
                    );
                    return None;
                }
            }
        }
    }
    None
}

/// Helper macro for registering a `(kind, struct)` pair with the
/// JSON-from-kind dispatcher. The struct must implement
/// `serde::Deserialize` so the JSON details can round-trip
/// through `serde_json::from_value`. The struct must also
/// implement `Problem` so the boxed result satisfies the trait.
///
/// Use site:
/// ```ignore
/// crate::register_problem_de!(MissingFile, "missing-file");
/// ```
#[macro_export]
macro_rules! register_problem_de {
    ($ty:ty, $kind:expr) => {
        ::inventory::submit! {
            $crate::ProblemDeserializer {
                kind: $kind,
                from_json: |v| {
                    ::serde_json::from_value::<$ty>(v.clone())
                        .map(|p| Box::new(p) as Box<dyn $crate::Problem>)
                },
            }
        }
    };
}

/// Register a deserializer with a custom function — for
/// problems whose JSON shape doesn't trivially round-trip
/// through `serde::Deserialize` (e.g. fields renamed in
/// `json()`, optional fields with defaults, etc.).
#[macro_export]
macro_rules! register_problem_de_fn {
    ($kind:expr, $fn:expr) => {
        ::inventory::submit! {
            $crate::ProblemDeserializer {
                kind: $kind,
                from_json: $fn,
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::Problem;

    /// Round-trip parity: every Problem::json() output should
    /// deserialize back into a Problem of the same kind via
    /// `problem_from_json`. If a mismatch shows up here it's
    /// because someone added/changed a field in `json()` without
    /// updating the matching deserializer registration.
    fn roundtrip<P: Problem + 'static>(p: &P) {
        let kind = p.kind().to_string();
        let json = p.json();
        let back = crate::problem_from_json(&kind, &json)
            .unwrap_or_else(|| panic!("no deserializer for kind {:?}", kind));
        assert_eq!(back.kind(), kind);
        assert_eq!(back.json(), json, "round-trip mismatch for kind {}", kind);
    }

    #[test]
    fn roundtrip_missing_file() {
        roundtrip(&crate::problems::common::MissingFile::new(
            "/usr/bin/foo".into(),
        ));
    }

    #[test]
    fn roundtrip_missing_command() {
        roundtrip(&crate::problems::common::MissingCommand("foo".into()));
    }

    #[test]
    fn roundtrip_vcs_control_directory_needed() {
        roundtrip(&crate::problems::common::VcsControlDirectoryNeeded::new(
            vec!["git", "bzr"],
        ));
    }

    #[test]
    fn roundtrip_missing_python_module() {
        roundtrip(&crate::problems::common::MissingPythonModule::simple(
            "numpy".to_string(),
        ));
    }

    #[test]
    fn roundtrip_missing_python_distribution() {
        roundtrip(&crate::problems::common::MissingPythonDistribution {
            distribution: "twisted".to_string(),
            python_version: Some(3),
            minimum_version: Some("18.0".to_string()),
        });
    }

    #[test]
    fn roundtrip_missing_c_header() {
        roundtrip(&crate::problems::common::MissingCHeader::new(
            "stdio.h".into(),
        ));
    }

    #[test]
    fn roundtrip_missing_go_package() {
        roundtrip(&crate::problems::common::MissingGoPackage {
            package: "github.com/foo/bar".into(),
        });
    }

    #[test]
    fn roundtrip_missing_node_module() {
        roundtrip(&crate::problems::common::MissingNodeModule("foo".into()));
    }

    #[test]
    fn roundtrip_missing_node_package() {
        roundtrip(&crate::problems::common::MissingNodePackage("foo".into()));
    }

    #[test]
    fn roundtrip_missing_vala_package() {
        roundtrip(&crate::problems::common::MissingValaPackage(
            "foo-1.0".into(),
        ));
    }

    #[test]
    fn roundtrip_missing_library() {
        roundtrip(&crate::problems::common::MissingLibrary("foo".into()));
    }

    #[test]
    fn roundtrip_missing_haskell_module() {
        roundtrip(&crate::problems::common::MissingHaskellModule::new(
            "Foo.Bar".into(),
        ));
    }

    #[test]
    fn roundtrip_missing_pkg_config() {
        roundtrip(&crate::problems::common::MissingPkgConfig {
            module: "foo".into(),
            minimum_version: Some("1.0".into()),
        });
    }

    #[test]
    fn roundtrip_missing_python_module_with_versions() {
        roundtrip(&crate::problems::common::MissingPythonModule {
            module: "django".into(),
            python_version: Some(3),
            minimum_version: Some("4.2".into()),
        });
    }

    #[test]
    fn roundtrip_missing_python_distribution_simple() {
        roundtrip(&crate::problems::common::MissingPythonDistribution {
            distribution: "twisted".into(),
            python_version: None,
            minimum_version: None,
        });
    }

    #[test]
    fn roundtrip_missing_pkg_config_no_version() {
        roundtrip(&crate::problems::common::MissingPkgConfig {
            module: "glib-2.0".into(),
            minimum_version: None,
        });
    }

    #[test]
    fn roundtrip_missing_haskell_dependencies() {
        roundtrip(&crate::problems::common::MissingHaskellDependencies(vec![
            "aeson".into(),
            "text".into(),
        ]));
    }

    #[test]
    fn roundtrip_missing_autoconf_macro_default() {
        roundtrip(&crate::problems::common::MissingAutoconfMacro::new(
            "AC_PROG_CC".into(),
        ));
    }

    #[test]
    fn roundtrip_missing_autoconf_macro_need_rebuild() {
        let mut p = crate::problems::common::MissingAutoconfMacro::new("PKG_CHECK_MODULES".into());
        p.need_rebuild = true;
        roundtrip(&p);
    }

    #[test]
    fn roundtrip_unsatisfied_apt_dependencies() {
        roundtrip(&crate::problems::debian::UnsatisfiedAptDependencies(
            "libssl-dev (>= 1.1)".into(),
        ));
    }

    #[test]
    fn roundtrip_apt_package_unknown() {
        roundtrip(&crate::problems::debian::AptPackageUnknown(
            "nonexistent-pkg".into(),
        ));
    }

    #[test]
    fn missing_haskell_dependencies_accepts_bare_array() {
        // The deserializer accepts both {"deps": [...]} (the json() shape)
        // and a bare array (legacy shape). Verify the bare-array path.
        let v = serde_json::json!(["aeson", "text"]);
        let p = crate::problem_from_json("missing-haskell-dependencies", &v)
            .expect("bare array should deserialize");
        assert_eq!(p.json(), serde_json::json!({"deps": ["aeson", "text"]}),);
    }

    #[test]
    fn apt_package_unknown_accepts_bare_string() {
        // The deserializer accepts both {"package": "..."} (the json() shape)
        // and a bare JSON string. Verify the bare-string path.
        let v = serde_json::json!("foo-pkg");
        let p = crate::problem_from_json("apt-package-unknown", &v)
            .expect("bare string should deserialize");
        assert_eq!(p.json(), serde_json::json!({"package": "foo-pkg"}));
    }

    #[test]
    fn missing_file_missing_path_returns_none() {
        // Required field absent — deserializer should fail and the
        // dispatcher should swallow the error into None.
        let v = serde_json::json!({});
        assert!(crate::problem_from_json("missing-file", &v).is_none());
    }

    #[test]
    fn missing_file_wrong_type_returns_none() {
        // Required field is the wrong JSON type — also rejected.
        let v = serde_json::json!({"path": 42});
        assert!(crate::problem_from_json("missing-file", &v).is_none());
    }

    #[test]
    fn missing_python_module_only_required_field() {
        // Optional fields absent (python_version, minimum_version) —
        // deserializer should still succeed.
        let v = serde_json::json!({"module": "numpy"});
        let p = crate::problem_from_json("missing-python-module", &v)
            .expect("optional fields should be optional");
        assert_eq!(p.kind(), "missing-python-module");
        assert_eq!(
            p.json(),
            serde_json::json!({
                "module": "numpy",
                "python_version": null,
                "minimum_version": null,
            }),
        );
    }

    #[test]
    fn missing_autoconf_macro_omits_need_rebuild() {
        // need_rebuild is optional in the deserializer (defaults to false).
        let v = serde_json::json!({"macro": "AC_PROG_CC"});
        let p = crate::problem_from_json("missing-autoconf-macro", &v)
            .expect("need_rebuild should be optional");
        assert_eq!(
            p.json(),
            serde_json::json!({"macro": "AC_PROG_CC", "need_rebuild": false}),
        );
    }

    #[test]
    fn unknown_kind_returns_none() {
        let v = serde_json::json!({"foo": "bar"});
        assert!(crate::problem_from_json("not-a-real-kind", &v).is_none());
    }
}
