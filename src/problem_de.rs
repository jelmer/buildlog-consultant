//! JSON-from-kind dispatcher registrations for the problem
//! types in [`crate::problems`].
//!
//! Each registration teaches [`crate::problem_from_json`] how to
//! reconstruct a `Box<dyn Problem>` from the `(kind, json)` pair
//! produced by `Problem::kind()` + `Problem::json()`. The set
//! covered here is curated for round-trip parity with
//! `Problem::json()` for the kinds the Janitor's followup
//! orchestration cares about — primarily the dependency-related
//! ones that map to `ognibuild::buildlog::problem_to_dependency`.
//!
//! Adding a new kind: pick the matching `Problem` impl, look at
//! its `json()` body, and add a `register_problem_de_fn!` block
//! below that mirrors the field shape exactly. Round-trip
//! coverage is guarded by the `roundtrip_*` tests at the bottom
//! of this file.

use crate::problems::common::*;
use crate::problems::debian::*;

// ---------------------------------------------------------------------------
// common.rs
// ---------------------------------------------------------------------------

crate::register_problem_de_fn!("missing-file", |v| {
    let path: String = v
        .get("path")
        .and_then(|p| p.as_str())
        .ok_or_else(|| serde::de::Error::missing_field("path"))?
        .to_string();
    Ok(Box::new(MissingFile { path: path.into() }) as Box<dyn crate::Problem>)
});

crate::register_problem_de_fn!("command-missing", |v| {
    let cmd = v
        .get("command")
        .and_then(|c| c.as_str())
        .ok_or_else(|| serde::de::Error::missing_field("command"))?
        .to_string();
    Ok(Box::new(MissingCommand(cmd)) as Box<dyn crate::Problem>)
});

crate::register_problem_de_fn!("vcs-control-directory-needed", |v| {
    let vcs: Vec<String> = v
        .get("vcs")
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|s| s.as_str().map(String::from))
                .collect()
        })
        .ok_or_else(|| serde::de::Error::missing_field("vcs"))?;
    Ok(Box::new(VcsControlDirectoryNeeded { vcs }) as Box<dyn crate::Problem>)
});

crate::register_problem_de_fn!("missing-python-module", |v| {
    let module = v
        .get("module")
        .and_then(|m| m.as_str())
        .ok_or_else(|| serde::de::Error::missing_field("module"))?
        .to_string();
    let python_version = v
        .get("python_version")
        .and_then(|x| x.as_i64())
        .map(|n| n as i32);
    let minimum_version = v
        .get("minimum_version")
        .and_then(|x| x.as_str())
        .map(String::from);
    Ok(Box::new(MissingPythonModule {
        module,
        python_version,
        minimum_version,
    }) as Box<dyn crate::Problem>)
});

crate::register_problem_de_fn!("missing-python-distribution", |v| {
    let distribution = v
        .get("distribution")
        .and_then(|m| m.as_str())
        .ok_or_else(|| serde::de::Error::missing_field("distribution"))?
        .to_string();
    let python_version = v
        .get("python_version")
        .and_then(|x| x.as_i64())
        .map(|n| n as i32);
    let minimum_version = v
        .get("minimum_version")
        .and_then(|x| x.as_str())
        .map(String::from);
    Ok(Box::new(MissingPythonDistribution {
        distribution,
        python_version,
        minimum_version,
    }) as Box<dyn crate::Problem>)
});

crate::register_problem_de_fn!("missing-c-header", |v| {
    let header = v
        .get("header")
        .and_then(|m| m.as_str())
        .ok_or_else(|| serde::de::Error::missing_field("header"))?
        .to_string();
    Ok(Box::new(MissingCHeader::new(header)) as Box<dyn crate::Problem>)
});

crate::register_problem_de_fn!("missing-go-package", |v| {
    let package = v
        .get("package")
        .and_then(|m| m.as_str())
        .ok_or_else(|| serde::de::Error::missing_field("package"))?
        .to_string();
    Ok(Box::new(MissingGoPackage { package }) as Box<dyn crate::Problem>)
});

crate::register_problem_de_fn!("missing-node-module", |v| {
    let m = v
        .get("module")
        .and_then(|m| m.as_str())
        .ok_or_else(|| serde::de::Error::missing_field("module"))?
        .to_string();
    Ok(Box::new(MissingNodeModule(m)) as Box<dyn crate::Problem>)
});

crate::register_problem_de_fn!("missing-node-package", |v| {
    let p = v
        .get("package")
        .and_then(|m| m.as_str())
        .ok_or_else(|| serde::de::Error::missing_field("package"))?
        .to_string();
    Ok(Box::new(MissingNodePackage(p)) as Box<dyn crate::Problem>)
});

crate::register_problem_de_fn!("missing-vala-package", |v| {
    let p = v
        .get("package")
        .and_then(|m| m.as_str())
        .ok_or_else(|| serde::de::Error::missing_field("package"))?
        .to_string();
    Ok(Box::new(MissingValaPackage(p)) as Box<dyn crate::Problem>)
});

crate::register_problem_de_fn!("missing-library", |v| {
    let l = v
        .get("library")
        .and_then(|m| m.as_str())
        .ok_or_else(|| serde::de::Error::missing_field("library"))?
        .to_string();
    Ok(Box::new(MissingLibrary(l)) as Box<dyn crate::Problem>)
});

crate::register_problem_de_fn!("missing-haskell-module", |v| {
    let module = v
        .get("module")
        .and_then(|m| m.as_str())
        .ok_or_else(|| serde::de::Error::missing_field("module"))?
        .to_string();
    Ok(Box::new(MissingHaskellModule::new(module)) as Box<dyn crate::Problem>)
});

crate::register_problem_de_fn!("missing-haskell-dependencies", |v| {
    let deps: Vec<String> = v
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|s| s.as_str().map(String::from))
                .collect()
        })
        .or_else(|| {
            // Some json() impls wrap in {"deps": [...]}. Accept that
            // shape too so we round-trip whichever flavour the
            // upstream emitted.
            v.get("deps").and_then(|d| d.as_array()).map(|arr| {
                arr.iter()
                    .filter_map(|s| s.as_str().map(String::from))
                    .collect()
            })
        })
        .ok_or_else(|| serde::de::Error::missing_field("deps"))?;
    Ok(Box::new(MissingHaskellDependencies(deps)) as Box<dyn crate::Problem>)
});

crate::register_problem_de_fn!("missing-pkg-config-package", |v| {
    let module = v
        .get("module")
        .and_then(|m| m.as_str())
        .ok_or_else(|| serde::de::Error::missing_field("module"))?
        .to_string();
    let minimum_version = v
        .get("minimum_version")
        .and_then(|x| x.as_str())
        .map(String::from);
    Ok(Box::new(MissingPkgConfig {
        module,
        minimum_version,
    }) as Box<dyn crate::Problem>)
});

crate::register_problem_de_fn!("missing-autoconf-macro", |v| {
    let m = v
        .get("macro")
        .and_then(|m| m.as_str())
        .ok_or_else(|| serde::de::Error::missing_field("macro"))?
        .to_string();
    let need_rebuild = v
        .get("need_rebuild")
        .and_then(|x| x.as_bool())
        .unwrap_or(false);
    let mut p = MissingAutoconfMacro::new(m);
    p.need_rebuild = need_rebuild;
    Ok(Box::new(p) as Box<dyn crate::Problem>)
});

// ---------------------------------------------------------------------------
// debian.rs
// ---------------------------------------------------------------------------

crate::register_problem_de_fn!("unsatisfied-apt-dependencies", |v| {
    let relations = v
        .get("relations")
        .and_then(|m| m.as_str())
        .ok_or_else(|| serde::de::Error::missing_field("relations"))?
        .to_string();
    Ok(Box::new(UnsatisfiedAptDependencies(relations)) as Box<dyn crate::Problem>)
});

crate::register_problem_de_fn!("apt-package-unknown", |v| {
    let pkg = v
        .get("package")
        .and_then(|m| m.as_str())
        .or_else(|| v.as_str())
        .ok_or_else(|| serde::de::Error::missing_field("package"))?
        .to_string();
    Ok(Box::new(AptPackageUnknown(pkg)) as Box<dyn crate::Problem>)
});

#[cfg(test)]
mod tests {
    use crate::Problem;

    /// Round-trip parity: every Problem::json() output should
    /// deserialize back into a Problem of the same kind via
    /// `problem_from_json`. If a mismatch shows up here it's
    /// because someone added/changed a field in `json()` without
    /// updating the matching deserializer above.
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
