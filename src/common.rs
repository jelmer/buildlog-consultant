use crate::lines::Lines;
use crate::problems::common::*;
/// Common code for all environments.
// TODO(jelmer): Right now this is just a straight port from Python. It needs a massive amount of
// refactoring, including a split of the file.
use crate::r#match::{Error, Matcher, MatcherGroup, RegexLineMatcher};
use crate::regex_line_matcher;
use crate::regex_para_matcher;
use crate::{Match, Problem};
use crate::{MultiLineMatch, Origin, SingleLineMatch};
use lazy_regex::{regex_captures, regex_is_match};
use regex::Captures;

fn node_module_missing(c: &Captures) -> Result<Option<Box<dyn Problem>>, Error> {
    if c.get(1).unwrap().as_str().starts_with("/<<PKGBUILDDIR>>/") {
        return Ok(None);
    }
    if c.get(1).unwrap().as_str().starts_with("./") {
        return Ok(None);
    }
    Ok(Some(Box::new(MissingNodeModule(
        c.get(1).unwrap().as_str().to_string(),
    ))))
}

fn file_not_found(c: &Captures) -> Result<Option<Box<dyn Problem>>, Error> {
    let path = c.get(1).unwrap().as_str();
    if path.starts_with('/') && !path.starts_with("/<<PKGBUILDDIR>>") {
        return Ok(Some(Box::new(MissingFile {
            path: std::path::PathBuf::from(path),
        })));
    }
    if let Some(filename) = path.strip_prefix("/<<PKGBUILDDIR>>/") {
        return Ok(Some(Box::new(MissingBuildFile {
            filename: filename.to_string(),
        })));
    }
    if path == ".git/HEAD" {
        return Ok(Some(Box::new(VcsControlDirectoryNeeded {
            vcs: vec!["git".to_string()],
        })));
    }
    if path == "CVS/Root" {
        return Ok(Some(Box::new(VcsControlDirectoryNeeded {
            vcs: vec!["cvs".to_string()],
        })));
    }
    if !path.contains('/') {
        // Maybe a missing command?
        return Ok(Some(Box::new(MissingBuildFile {
            filename: path.to_string(),
        })));
    }
    Ok(None)
}

fn file_not_found_maybe_executable(p: &str) -> Result<Option<Box<dyn Problem>>, Error> {
    if p.starts_with('/') && !p.starts_with("/<<PKGBUILDDIR>>") {
        return Ok(Some(Box::new(MissingFile {
            path: std::path::PathBuf::from(p),
        })));
    }

    if !p.contains('/') {
        // Maybe a missing command?
        return Ok(Some(Box::new(MissingCommandOrBuildFile {
            filename: p.to_string(),
        })));
    }
    Ok(None)
}

fn interpreter_missing(c: &Captures) -> Result<Option<Box<dyn Problem>>, Error> {
    if c.get(1).unwrap().as_str().starts_with('/') {
        if c.get(1).unwrap().as_str().contains("PKGBUILDDIR") {
            return Ok(None);
        }
        return Ok(Some(Box::new(MissingFile {
            path: std::path::PathBuf::from(c.get(1).unwrap().as_str().to_string()),
        })));
    }
    if c.get(1).unwrap().as_str().contains('/') {
        return Ok(None);
    }
    return Ok(Some(Box::new(MissingCommand(
        c.get(1).unwrap().as_str().to_string(),
    ))));
}

fn pkg_config_missing(c: &Captures) -> Result<Option<Box<dyn Problem>>, Error> {
    let expr = c.get(1).unwrap().as_str().split('\t').next().unwrap();
    if let Some((pkg, minimum)) = expr.split_once(">=") {
        return Ok(Some(Box::new(MissingPkgConfig {
            module: pkg.trim().to_string(),
            minimum_version: Some(minimum.trim().to_string()),
        })));
    }
    if !expr.contains(' ') {
        return Ok(Some(Box::new(MissingPkgConfig {
            module: expr.to_string(),
            minimum_version: None,
        })));
    }
    // Hmmm
    Ok(None)
}

fn command_missing(c: &Captures) -> Result<Option<Box<dyn Problem>>, Error> {
    let command = c.get(1).unwrap().as_str();
    if command.contains("PKGBUILDDIR") {
        return Ok(None);
    }
    if command == "./configure" {
        return Ok(Some(Box::new(MissingConfigure)));
    }
    if command.starts_with("./") || command.starts_with("../") {
        return Ok(None);
    }
    if command == "debian/rules" {
        return Ok(None);
    }
    Ok(Some(Box::new(MissingCommand(command.to_string()))))
}

lazy_static::lazy_static! {
    static ref CONFIGURE_LINE_MATCHERS: MatcherGroup = MatcherGroup::new(vec![
        regex_line_matcher!(
            r"^\s*Unable to find (.*) \(http(.*)\)",
            |m| Ok(Some(Box::new(MissingVagueDependency{
                name: m.get(1).unwrap().as_str().to_string(),
                url: Some(m.get(2).unwrap().as_str().to_string()),
                minimum_version: None,
                current_version: None,
            })))
        ),
        regex_line_matcher!(
            r"^\s*Unable to find (.*)\.",
            |m| Ok(Some(Box::new(MissingVagueDependency{
                name: m.get(1).unwrap().as_str().to_string(),
                url: None,
                minimum_version: None,
                current_version: None,
            })))
        ),
    ]);
}

#[derive(Debug, Clone)]
struct MultiLineConfigureErrorMatcher;

impl Matcher for MultiLineConfigureErrorMatcher {
    fn extract_from_lines(
        &self,
        lines: &[&str],
        offset: usize,
    ) -> Result<Option<(Box<dyn Match>, Option<Box<dyn Problem>>)>, Error> {
        if lines[offset].trim_end_matches(|c| c == '\r' || c == '\n') != "configure: error:" {
            return Ok(None);
        }

        let mut relevant_linenos = vec![];
        for (j, line) in lines.enumerate_forward(None).skip(offset + 1) {
            if line.trim().is_empty() {
                continue;
            }
            relevant_linenos.push(j);
            let m = CONFIGURE_LINE_MATCHERS.extract_from_lines(lines, j)?;
            if let Some(m) = m {
                return Ok(Some(m));
            }
        }

        let m = MultiLineMatch::new(
            Origin("configure".into()),
            relevant_linenos.clone(),
            lines
                .iter()
                .enumerate()
                .filter(|(i, _)| relevant_linenos.contains(i))
                .map(|(_, l)| l.to_string())
                .collect(),
        );

        Ok(Some((Box::new(m), None)))
    }
}

#[derive(Debug, Clone)]
struct HaskellMissingDependencyMatcher;

impl Matcher for HaskellMissingDependencyMatcher {
    fn extract_from_lines(
        &self,
        lines: &[&str],
        offset: usize,
    ) -> Result<Option<(Box<dyn Match>, Option<Box<dyn Problem>>)>, Error> {
        if !regex_is_match!(
            r"(.*): Encountered missing or private dependencies:",
            lines[offset].trim_end_matches('\n')
        ) {
            return Ok(None);
        }

        let mut deps = vec![];
        let mut offsets = vec![offset];

        for (offset, line) in lines.enumerate_forward(None).skip(offset + 1) {
            if line.trim().is_empty() {
                break;
            }
            if let Some((dep, _)) = line.trim().split_once(',') {
                deps.push(dep.to_string());
            }
            offsets.push(offset);
        }
        let m = MultiLineMatch {
            origin: Origin("haskell dependencies".into()),
            offsets: offsets.clone(),
            lines: offsets.iter().map(|i| lines[*i].to_string()).collect(),
        };
        let p = MissingHaskellDependencies(deps);
        Ok(Some((Box::new(m), Some(Box::new(p)))))
    }
}

#[derive(Debug, Clone)]
struct SetupPyCommandMissingMatcher;

impl Matcher for SetupPyCommandMissingMatcher {
    fn extract_from_lines(
        &self,
        lines: &[&str],
        offset: usize,
    ) -> Result<Option<(Box<dyn Match>, Option<Box<dyn Problem>>)>, Error> {
        let first_offset = offset;
        let command =
            match regex_captures!(r"error: invalid command \'(.*)\'", lines[offset].trim()) {
                None => return Ok(None),
                Some((_, command)) => command,
            };

        for j in 0..20 {
            let offset = offset - j;
            let line = lines[offset].trim_end_matches('\n');

            if regex_is_match!(
                r"usage: setup.py \[global_opts\] cmd1 \[cmd1_opts\] \[cmd2 \[cmd2_opts\] \.\.\.\]",
                line,
            ) {
                let offsets: Vec<usize> = vec![first_offset];
                let m = MultiLineMatch {
                    origin: Origin("setup.py".into()),
                    offsets,
                    lines: vec![lines[first_offset].to_string()],
                };

                let p = MissingSetupPyCommand(command.to_string());
                return Ok(Some((Box::new(m), Some(Box::new(p)))));
            }
        }

        log::warn!("Unable to find setup.py usage line");
        Ok(None)
    }
}

#[derive(Debug, Clone)]
struct PythonFileNotFoundErrorMatcher;

impl Matcher for PythonFileNotFoundErrorMatcher {
    fn extract_from_lines(
        &self,
        lines: &[&str],
        offset: usize,
    ) -> Result<Option<(Box<dyn Match>, Option<Box<dyn Problem>>)>, Error> {
        if let Some((_, name)) = lazy_regex::regex_captures!(
            r"^(?:E  +)?FileNotFoundError: \[Errno 2\] No such file or directory: \'(.*)\'",
            lines[offset].trim_end_matches('\n')
        ) {
            if offset > 2 && lines[offset - 2].contains("subprocess") {
                return Ok(Some((
                    Box::new(SingleLineMatch {
                        origin: Origin("python".into()),
                        offset,
                        line: lines[offset].to_string(),
                    }),
                    Some(Box::new(MissingCommand(name.to_string()))),
                )));
            } else {
                return Ok(Some((
                    Box::new(SingleLineMatch {
                        origin: Origin("python".into()),
                        offset,
                        line: lines[offset].to_string(),
                    }),
                    file_not_found_maybe_executable(name)?,
                )));
            }
        }

        Ok(None)
    }
}

#[derive(Debug, Clone)]
struct MultiLinePerlMissingModulesErrorMatcher;

impl Matcher for MultiLinePerlMissingModulesErrorMatcher {
    fn extract_from_lines(
        &self,
        lines: &[&str],
        offset: usize,
    ) -> Result<Option<(Box<dyn Match>, Option<Box<dyn Problem>>)>, Error> {
        let line = lines[offset].trim_end_matches(|c| c == '\r' || c == '\n');
        if line != "# The following modules are not available." {
            return Ok(None);
        }
        if lines[offset + 1].trim_end_matches(|c| c == '\r' || c == '\n')
            != "# `perl Makefile.PL | cpanm` will install them:"
        {
            return Ok(None);
        }

        let relevant_linenos = vec![offset, offset + 1, offset + 2];

        let m = MultiLineMatch::new(
            Origin("perl line match".into()),
            relevant_linenos.clone(),
            lines
                .iter()
                .enumerate()
                .filter(|(i, _)| relevant_linenos.contains(i))
                .map(|(_, l)| l.to_string())
                .collect(),
        );

        let problem: Option<Box<dyn Problem>> = Some(Box::new(MissingPerlModule::simple(
            lines[offset + 2].trim(),
        )));

        Ok(Some((Box::new(m), problem)))
    }
}

lazy_static::lazy_static! {
    static ref VIGNETTE_LINE_MATCHERS: MatcherGroup = MatcherGroup::new(vec![
        regex_line_matcher!(r"^([^ ]+) is not available", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
        regex_line_matcher!(r"^The package `(.*)` is required\.", |m| Ok(Some(Box::new(MissingRPackage::simple(m.get(1).unwrap().as_str()))))),
        regex_line_matcher!(r"^Package '(.*)' required.*", |m| Ok(Some(Box::new(MissingRPackage::simple(m.get(1).unwrap().as_str()))))),
        regex_line_matcher!(r"^The '(.*)' package must be installed.*", |m| Ok(Some(Box::new(MissingRPackage::simple(m.get(1).unwrap().as_str()))))),
    ]);
}

#[derive(Debug, Clone)]
struct MultiLineVignetteErrorMatcher;

impl Matcher for MultiLineVignetteErrorMatcher {
    fn extract_from_lines(
        &self,
        lines: &[&str],
        offset: usize,
    ) -> Result<Option<(Box<dyn Match>, Option<Box<dyn Problem>>)>, Error> {
        let header_m =
            regex::Regex::new(r"^Error: processing vignette '(.*)' failed with diagnostics:")
                .unwrap();

        if !header_m.is_match(lines[offset]) {
            return Ok(None);
        }

        if let Some((m, p)) = VIGNETTE_LINE_MATCHERS.extract_from_lines(lines, offset + 1)? {
            return Ok(Some((m, p)));
        }

        Ok(Some((
            Box::new(SingleLineMatch {
                origin: Origin("vignette line match".into()),
                offset: offset + 1,
                line: lines[offset + 1].to_string(),
            }),
            None,
        )))
    }
}

#[derive(Debug, Clone)]
struct AutoconfUnexpectedMacroMatcher;

impl Matcher for AutoconfUnexpectedMacroMatcher {
    fn extract_from_lines(
        &self,
        lines: &[&str],
        offset: usize,
    ) -> Result<Option<(Box<dyn Match>, Option<Box<dyn Problem>>)>, Error> {
        if !regex_is_match!(
            r"\./configure: line [0-9]+: syntax error near unexpected token `.+'",
            lines[offset]
        ) {
            return Ok(None);
        }

        let m = MultiLineMatch::new(
            Origin("autoconf unexpected macro".into()),
            vec![offset, offset + 1],
            vec![lines[offset].to_string(), lines[offset + 1].to_string()],
        );

        let problem = if let Some((_, r#macro)) = regex_captures!(
            r"^\./configure: line [0-9]+: `[\s\t]*([A-Z0-9_]+)\(.*",
            lines[offset + 1]
        ) {
            Some(Box::new(MissingAutoconfMacro {
                r#macro: r#macro.to_string(),
                need_rebuild: true,
            }) as Box<dyn Problem>)
        } else {
            None
        };

        Ok(Some((Box::new(m), problem)))
    }
}

fn maven_missing_artifact(m: &regex::Captures) -> Result<Option<Box<dyn Problem>>, Error> {
    let artifacts = m
        .get(1)
        .unwrap()
        .as_str()
        .split(',')
        .map(|s| s.trim().to_string())
        .collect::<Vec<_>>();
    Ok(Some(Box::new(MissingMavenArtifacts(artifacts))))
}

fn r_missing_package(m: &regex::Captures) -> Result<Option<Box<dyn Problem>>, Error> {
    let fragment = m.get(1).unwrap().as_str();
    let deps = fragment
        .split(",")
        .map(|dep| {
            dep.trim_matches('‘')
                .trim_matches('’')
                .trim_matches('\'')
                .to_string()
        })
        .collect::<Vec<_>>();
    Ok(Some(Box::new(MissingRPackage::simple(&deps[0]))))
}

fn webpack_file_missing(m: &regex::Captures) -> Result<Option<Box<dyn Problem>>, Error> {
    let path = std::path::Path::new(m.get(1).unwrap().as_str());
    let container = std::path::Path::new(m.get(2).unwrap().as_str());
    let path = container.join(path);
    if path.starts_with("/") && !path.as_path().starts_with("/<<PKGBUILDDIR>>") {
        return Ok(Some(Box::new(MissingFile { path })));
    }
    Ok(None)
}

fn ruby_missing_gem(m: &regex::Captures) -> Result<Option<Box<dyn Problem>>, Error> {
    let mut minimum_version = None;
    for grp in m.get(2).unwrap().as_str().split(",") {
        if let Some((cond, val)) = grp.trim().split_once(" ") {
            if cond == ">=" {
                minimum_version = Some(val.to_string());
                break;
            }
            if cond == "~>" {
                minimum_version = Some(val.to_string());
            }
        }
    }
    Ok(Some(Box::new(MissingRubyGem::new(
        m.get(1).unwrap().as_str().to_string(),
        minimum_version,
    ))))
}

const MAVEN_ERROR_PREFIX: &str = "(?:\\[ERROR\\]|\\[\x1b\\[1;31mERROR\x1b\\[m\\]) ";

lazy_static::lazy_static! {
    static ref COMMON_MATCHERS: MatcherGroup = MatcherGroup::new(vec![
        regex_line_matcher!(r"^[^:]+:\d+: (.*): No such file or directory$", |m| file_not_found_maybe_executable(m.get(1).unwrap().as_str())),
        regex_line_matcher!(
        r"^(distutils.errors.DistutilsError|error): Could not find suitable distribution for Requirement.parse\('([^']+)'\)$",
        |c| {
            let req = c.get(2).unwrap().as_str().split(';').next().unwrap();
            Ok(Some(Box::new(MissingPythonDistribution::from_requirement_str(req, None))))
        }),
        regex_line_matcher!(
            r"^We need the Python library (.*) to be installed. Try runnning: python -m ensurepip$",
            |c| Ok(Some(Box::new(MissingPythonDistribution { distribution: c.get(1).unwrap().as_str().to_string(), python_version: None, minimum_version: None })))),
        regex_line_matcher!(
            r"^pkg_resources.DistributionNotFound: The '([^']+)' distribution was not found and is required by the application$",
            |c| Ok(Some(Box::new(MissingPythonDistribution::from_requirement_str(c.get(1).unwrap().as_str(), None))))),
        regex_line_matcher!(
            r"^pkg_resources.DistributionNotFound: The '([^']+)' distribution was not found and is required by (.*)$",
            |c| Ok(Some(Box::new(MissingPythonDistribution::from_requirement_str(c.get(1).unwrap().as_str(), None))))),
        regex_line_matcher!(
            r"^Please install cmake version >= (.*) and re-run setup$",
            |_| Ok(Some(Box::new(MissingCommand("cmake".to_string()))))),
        regex_line_matcher!(
            r"^pluggy.manager.PluginValidationError: Plugin '.*' could not be loaded: \(.* \(/usr/lib/python2.[0-9]/dist-packages\), Requirement.parse\('(.*)'\)\)!$",
            |c| {
                let expr = c.get(1).unwrap().as_str();
                let python_version = Some(2);
                if let Some((pkg, minimum)) = expr.split_once(">=") {
                    Ok(Some(Box::new(MissingPythonModule {
                        module: pkg.trim().to_string(),
                        python_version,
                        minimum_version: Some(minimum.trim().to_string()),
                    })))
                } else if !expr.contains(' ') {
                    Ok(Some(Box::new(MissingPythonModule {
                        module: expr.trim().to_string(),
                        python_version,
                        minimum_version: None,
                    })))
                }
                else {
                    Ok(None)
                }
            }),
        regex_line_matcher!(r"^E ImportError: (.*) could not be imported\.$", |m| Ok(Some(Box::new(MissingPythonModule {
            module: m.get(1).unwrap().as_str().to_string(),
            python_version: None,
            minimum_version: None
        })))),
        regex_line_matcher!(r"^ImportError: could not find any library for ([^ ]+) .*$", |m| Ok(Some(Box::new(MissingLibrary(m.get(1).unwrap().as_str().to_string()))))),
        regex_line_matcher!(r"^ImportError: cannot import name (.*), introspection typelib not found$", |m| Ok(Some(Box::new(MissingIntrospectionTypelib(m.get(1).unwrap().as_str().to_string()))))),
        regex_line_matcher!(r"^ValueError: Namespace (.*) not available$", |m| Ok(Some(Box::new(MissingIntrospectionTypelib(m.get(1).unwrap().as_str().to_string()))))),
        regex_line_matcher!(r"^  namespace '(.*)' ([^ ]+) is being loaded, but >= ([^ ]+) is required$", |m| {
            let package = m.get(1).unwrap().as_str();
            let min_version = m.get(3).unwrap().as_str();

            Ok(Some(Box::new(MissingRPackage {
                package: package.to_string(),
                minimum_version: Some(min_version.to_string()),
            })))
        }),
        regex_line_matcher!("^ImportError: cannot import name '(.*)' from '(.*)'$", |m| {
            let module = m.get(2).unwrap().as_str();
            let name = m.get(1).unwrap().as_str();
            // TODO(jelmer): This name won't always refer to a module
            let name = format!("{}.{}", module, name);
            Ok(Some(Box::new(MissingPythonModule {
                module: name,
                python_version: None,
                minimum_version: None,
            })))
        }),
        regex_line_matcher!("^E       fixture '(.*)' not found$", |m| Ok(Some(Box::new(MissingPytestFixture(m.get(1).unwrap().as_str().to_string()))))),
        regex_line_matcher!("^pytest: error: unrecognized arguments: (.*)$", |m| {
            let args = shlex::split(m.get(1).unwrap().as_str()).unwrap();
            Ok(Some(Box::new(UnsupportedPytestArguments(args))))
        }),
        regex_line_matcher!(
            "^INTERNALERROR> pytest.PytestConfigWarning: Unknown config option: (.*)$",
            |m| Ok(Some(Box::new(UnsupportedPytestConfigOption(m.get(1).unwrap().as_str().to_string()))))),
        regex_line_matcher!("^E   ImportError: cannot import name '(.*)' from '(.*)'", |m| {
            let name = m.get(1).unwrap().as_str();
            let module = m.get(2).unwrap().as_str();
            Ok(Some(Box::new(MissingPythonModule {
                module: format!("{}.{}", module, name),
                python_version: None,
                minimum_version: None,
            })))
        }),
        regex_line_matcher!("^E   ImportError: cannot import name ([^']+)", |m| {
            Ok(Some(Box::new(MissingPythonModule {
                module: m.get(1).unwrap().as_str().to_string(),
                python_version: None,
                minimum_version: None,
            })))
        }),
        regex_line_matcher!(r"^django.core.exceptions.ImproperlyConfigured: Error loading .* module: No module named '(.*)'", |m| {
            Ok(Some(Box::new(MissingPythonModule {
                module: m.get(1).unwrap().as_str().to_string(),
                python_version: None,
                minimum_version: None,
            })))
        }),
        regex_line_matcher!("^E   ImportError: No module named (.*)", |m| {
            Ok(Some(Box::new(MissingPythonModule {
                module: m.get(1).unwrap().as_str().to_string(),
                python_version: None,
                minimum_version: None,
            })))
        }),
        regex_line_matcher!(r"^\s*ModuleNotFoundError: No module named '(.*)'",|m| {
            Ok(Some(Box::new(MissingPythonModule {
                module: m.get(1).unwrap().as_str().to_string(),
                python_version: Some(3),
                minimum_version: None,
            })))
        }),
        regex_line_matcher!(r"^Could not import extension .* \(exception: No module named (.*)\)", |m| {
            Ok(Some(Box::new(MissingPythonModule {
                module: m.get(1).unwrap().as_str().trim().to_string(),
                python_version: None,
                minimum_version: None,
            })))
        }),
        regex_line_matcher!(r"^Could not import (.*)\.", |m| {
            Ok(Some(Box::new(MissingPythonModule {
                module: m.get(1).unwrap().as_str().trim().to_string(),
                python_version: None,
                minimum_version: None,
            })))
        }),
        regex_line_matcher!(r"^(.*): Error while finding module specification for '(.*)' \(ModuleNotFoundError: No module named '(.*)'\)", |m| {
            let exec = m.get(1).unwrap().as_str();
            let python_version = if exec.ends_with("python3") {
                Some(3)
            } else if exec.ends_with("python2") {
                Some(2)
            } else {
                None
            };

            Ok(Some(Box::new(MissingPythonModule {
                module: m.get(3).unwrap().as_str().trim().to_string(),
                python_version,
                minimum_version: None,
            })))}),
        regex_line_matcher!("^E   ModuleNotFoundError: No module named '(.*)'", |m| {
            Ok(Some(Box::new(MissingPythonModule {
                module: m.get(1).unwrap().as_str().to_string(),
                python_version: Some(3),
                minimum_version: None
            })))
        }),
        regex_line_matcher!(r"^/usr/bin/python3: No module named ([^ ]+).*", |m| {
            Ok(Some(Box::new(MissingPythonModule {
                module: m.get(1).unwrap().as_str().to_string(),
                python_version: Some(3),
                minimum_version: None,
            })))
        }),
        regex_line_matcher!(r#"^(.*:[0-9]+|package .*): cannot find package "(.*)" in any of:"#, |m| Ok(Some(Box::new(MissingGoPackage { package: m.get(2).unwrap().as_str().to_string() })))),
        regex_line_matcher!(r#"^ImportError: Error importing plugin ".*": No module named (.*)"#, |m| {
            Ok(Some(Box::new(MissingPythonModule {
                module: m.get(1).unwrap().as_str().to_string(),
                python_version: None,
                minimum_version: None,
            })))
        }),
        regex_line_matcher!(r"^ImportError: No module named (.*)", |m| {
            Ok(Some(Box::new(MissingPythonModule {
                module: m.get(1).unwrap().as_str().to_string(),
                python_version: None,
                minimum_version: None,
            })))
        }),
        regex_line_matcher!(r"^[^:]+:\d+:\d+: fatal error: (.+\.h|.+\.hh|.+\.hpp): No such file or directory", |m| Ok(Some(Box::new(MissingCHeader { header: m.get(1).unwrap().as_str().to_string() })))),
        regex_line_matcher!(r"^[^:]+:\d+:\d+: fatal error: (.+\.xpm): No such file or directory", file_not_found),
        regex_line_matcher!(r".*fatal: not a git repository \(or any parent up to mount point /\)", |_| Ok(Some(Box::new(VcsControlDirectoryNeeded { vcs: vec!["git".to_string()] })))),
        regex_line_matcher!(r".*fatal: not a git repository \(or any of the parent directories\): \.git", |_| Ok(Some(Box::new(VcsControlDirectoryNeeded { vcs: vec!["git".to_string()] })))),
        regex_line_matcher!(r"[^:]+\.[ch]:\d+:\d+: fatal error: (.+): No such file or directory", |m| Ok(Some(Box::new(MissingCHeader { header: m.get(1).unwrap().as_str().to_string() })))),
        regex_line_matcher!("^.*␛\x1b\\[31mERROR:␛\x1b\\[39m Error: Cannot find module '(.*)'", node_module_missing),
    regex_line_matcher!("^\x1b\\[2mError: Cannot find module '(.*)'", node_module_missing),
    regex_line_matcher!("^\x1b\\[1m\x1b\\[31m\\[!\\] \x1b\\[1mError: Cannot find module '(.*)'", node_module_missing),
    regex_line_matcher!("^✖ \x1b\\[31mERROR:\x1b\\[39m Error: Cannot find module '(.*)'", node_module_missing),
    regex_line_matcher!("^\x1b\\[0;31m  Error: To use the transpile option, you must have the '(.*)' module installed",
     node_module_missing),
    regex_line_matcher!(r#"^\[31mError: No test files found: "(.*)"\[39m"#),
    regex_line_matcher!(r#"^\x1b\[31mError: No test files found: "(.*)"\x1b\[39m"#),
    regex_line_matcher!(r"^\s*Error: Cannot find module '(.*)'", node_module_missing),
    regex_line_matcher!(r"^>> Error: Cannot find module '(.*)'", node_module_missing),
    regex_line_matcher!(r"^>> Error: Cannot find module '(.*)' from '.*'", node_module_missing),
    regex_line_matcher!(r"^Error: Failed to load parser '.*' declared in '.*': Cannot find module '(.*)'", |m| Ok(Some(Box::new(MissingNodeModule(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"^    Cannot find module '(.*)' from '.*'", |m| Ok(Some(Box::new(MissingNodeModule(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"^>> Error: Grunt attempted to load a \.coffee file but CoffeeScript was not installed\.", |_| Ok(Some(Box::new(MissingNodePackage("coffeescript".to_string()))))),
    regex_line_matcher!(r"^>> Got an unexpected exception from the coffee-script compiler. The original exception was: Error: Cannot find module '(.*)'", node_module_missing),
    regex_line_matcher!(r"^\s*Module not found: Error: Can't resolve '(.*)' in '(.*)'", node_module_missing),
    regex_line_matcher!(r"^  Module (.*) in the transform option was not found\.", node_module_missing),
    regex_line_matcher!(
        r"^libtool/glibtool not found!",
        |_| Ok(Some(Box::new(MissingVagueDependency::simple("libtool"))))),
    regex_line_matcher!(r"^qmake: could not find a Qt installation of ''", |_| Ok(Some(Box::new(MissingQt)))),
    regex_line_matcher!(r"^Cannot find X include files via .*", |_| Ok(Some(Box::new(MissingX11)))),
    regex_line_matcher!(
        r"^\*\*\* No X11! Install X-Windows development headers/libraries! \*\*\*",
        |_| Ok(Some(Box::new(MissingX11)))
    ),
    regex_line_matcher!(
        r"^configure: error: \*\*\* No X11! Install X-Windows development headers/libraries! \*\*\*",
        |_| Ok(Some(Box::new(MissingX11)))
    ),
    regex_line_matcher!(
        r"^configure: error: The Java compiler javac failed.*",
        |_| Ok(Some(Box::new(MissingCommand("javac".to_string()))))
    ),
    regex_line_matcher!(
        r"^configure: error: No ([^ ]+) command found",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"^ERROR: InvocationError for command could not find executable (.*)",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"^  \*\*\* The (.*) script could not be found\. .*",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r#"^(.*)" command could not be found. (.*)"#,
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"^configure: error: cannot find lib ([^ ]+)",
        |m| Ok(Some(Box::new(MissingLibrary(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r#"^>> Local Npm module "(.*)" not found. Is it installed?"#, node_module_missing),
    regex_line_matcher!(
        r"^npm ERR! CLI for webpack must be installed.",
        |_| Ok(Some(Box::new(MissingNodePackage("webpack-cli".to_string()))))
    ),
    regex_line_matcher!(r"^npm ERR! \[!\] Error: Cannot find module '(.*)'", node_module_missing),
    regex_line_matcher!(
        r#"^npm ERR! >> Local Npm module "(.*)" not found. Is it installed\?"#,
        node_module_missing
    ),
    regex_line_matcher!(r"^npm ERR! Error: Cannot find module '(.*)'", node_module_missing),
    regex_line_matcher!(
        r"^npm ERR! ERROR in Entry module not found: Error: Can't resolve '(.*)' in '.*'",
        node_module_missing
    ),
    regex_line_matcher!(r"^npm ERR! sh: [0-9]+: (.*): not found", command_missing),
    regex_line_matcher!(r"^npm ERR! (.*\.ts)\([0-9]+,[0-9]+\): error TS[0-9]+: Cannot find module '(.*)' or its corresponding type declarations.", |m| Ok(Some(Box::new(MissingNodeModule(m.get(2).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"^npm ERR! Error: spawn (.*) ENOENT", command_missing),

    regex_line_matcher!(
        r"^(\./configure): line \d+: ([A-Z0-9_]+): command not found",
        |m| Ok(Some(Box::new(MissingAutoconfMacro::new(m.get(2).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r"^.*: line \d+: ([^ ]+): command not found", command_missing),
    regex_line_matcher!(r"^.*: line \d+: ([^ ]+): Permission denied"),
    regex_line_matcher!(r"^make\[[0-9]+\]: .*: Permission denied"),
    regex_line_matcher!(r"^/usr/bin/texi2dvi: TeX neither supports -recorder nor outputs \\openout lines in its log file"),
    regex_line_matcher!(r"^/bin/sh: \d+: ([^ ]+): not found", command_missing),
    regex_line_matcher!(r"^sh: \d+: ([^ ]+): not found", command_missing),
    regex_line_matcher!(r"^.*\.sh: \d+: ([^ ]+): not found", command_missing),
    regex_line_matcher!(r"^.*: 1: cd: can't cd to (.*)", |m| Ok(Some(Box::new(DirectoryNonExistant(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"^/bin/bash: (.*): command not found", command_missing),
    regex_line_matcher!(r"^bash: ([^ ]+): command not found", command_missing),
    regex_line_matcher!(r"^env: ‘(.*)’: No such file or directory", interpreter_missing),
    regex_line_matcher!(r"^/bin/bash: .*: (.*): bad interpreter: No such file or directory", interpreter_missing),
    // SH Errors
    regex_line_matcher!(r"^.*: [0-9]+: exec: (.*): not found", command_missing),
    regex_line_matcher!(r"^.*: [0-9]+: (.*): not found", command_missing),
    regex_line_matcher!(r"^/usr/bin/env: [‘'](.*)['’]: No such file or directory", command_missing),
    regex_line_matcher!(r"^make\[[0-9]+\]: (.*): Command not found", command_missing),
    regex_line_matcher!(r"^make: (.*): Command not found", command_missing),
    regex_line_matcher!(r"^make: (.*): No such file or directory", command_missing),
    regex_line_matcher!(r"^xargs: (.*): No such file or directory", command_missing),
    regex_line_matcher!(r"^make\[[0-9]+\]: ([^/ :]+): No such file or directory", command_missing),
    regex_line_matcher!(r"^.*: failed to exec '(.*)': No such file or directory", command_missing),
    regex_line_matcher!(r"^No package '([^']+)' found", pkg_config_missing),
    regex_line_matcher!(r"^--\s* No package '([^']+)' found", pkg_config_missing),
    regex_line_matcher!(
        r"^\-\- Please install Git, make sure it is in your path, and then try again.",
        |_| Ok(Some(Box::new(MissingCommand("git".to_string()))))
    ),
    regex_line_matcher!(
        r#"^\+ERROR:  could not access file "(.*)": No such file or directory"#,
        |m| Ok(Some(Box::new(MissingPostgresExtension(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r#"^configure: error: (Can't|Cannot) find "(.*)" in your PATH.*"#,
        |m| Ok(Some(Box::new(MissingCommand(m.get(2).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"^configure: error: Cannot find (.*) in your system path",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r#"^> Cannot run program "(.*)": error=2, No such file or directory"#,
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r"^(.*) binary '(.*)' not available .*", |m| Ok(Some(Box::new(MissingCommand(m.get(2).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"^An error has occurred: FatalError: git failed\. Is it installed, and are you in a Git repository directory\?",
     |_| Ok(Some(Box::new(MissingCommand("git".to_string()))))),
    regex_line_matcher!("^Please install '(.*)' seperately and try again.", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(
        r"^> A problem occurred starting process 'command '(.*)''", |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"^vcver.scm.git.GitCommandError: 'git .*' returned an error code 127",
        |_| Ok(Some(Box::new(MissingCommand("git".to_string()))))
    ),
    Box::new(MultiLineConfigureErrorMatcher),
    Box::new(MultiLinePerlMissingModulesErrorMatcher),
    Box::new(MultiLineVignetteErrorMatcher),
    regex_line_matcher!(r"^configure: error: No package '([^']+)' found", pkg_config_missing),
    regex_line_matcher!(r"^configure: error: (doxygen|asciidoc) is not available and maintainer mode is enabled", |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"^configure: error: Documentation enabled but rst2html not found.", |_| Ok(Some(Box::new(MissingCommand("rst2html".to_string()))))),
    regex_line_matcher!(r"^cannot run pkg-config to check .* version at (.*) line [0-9]+\.", |_| Ok(Some(Box::new(MissingCommand("pkg-config".to_string()))))),
    regex_line_matcher!(r"^Error: pkg-config not found!", |_| Ok(Some(Box::new(MissingCommand("pkg-config".to_string()))))),
    regex_line_matcher!(r"^\*\*\* pkg-config (.*) or newer\. You can download pkg-config", |m| Ok(Some(Box::new(MissingVagueDependency {
        name: "pkg-config".to_string(),
        minimum_version: Some(m.get(1).unwrap().as_str().to_string()),
        url: None,
        current_version: None
    })))),
    // Tox
    regex_line_matcher!(r"^ERROR: InterpreterNotFound: (.*)", |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"^ERROR: unable to find python", |_| Ok(Some(Box::new(MissingCommand("python".to_string()))))),
    regex_line_matcher!(r"^ ERROR: BLAS not found!", |_| Ok(Some(Box::new(MissingLibrary("blas".to_string()))))),
    Box::new(AutoconfUnexpectedMacroMatcher),
    regex_line_matcher!(r"^\./configure: [0-9]+: \.: Illegal option .*"),
    regex_line_matcher!(r"^Requested '(.*)' but version of ([^ ]+) is ([^ ]+)", pkg_config_missing),
    regex_line_matcher!(r"^.*configure: error: Package requirements \((.*)\) were not met:", pkg_config_missing),
    regex_line_matcher!(r"^configure: error: [a-z0-9_-]+-pkg-config (.*) couldn't be found", pkg_config_missing),
    regex_line_matcher!(r#"^configure: error: C preprocessor "/lib/cpp" fails sanity check"#),
    regex_line_matcher!(r"^configure: error: .*\. Please install (bison|flex)", |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"^configure: error: No C\# compiler found. You need to install either mono \(>=(.*)\) or \.Net", |_| Ok(Some(Box::new(MissingCSharpCompiler)))),
    regex_line_matcher!(r"^configure: error: No C\# compiler found", |_| Ok(Some(Box::new(MissingCSharpCompiler)))),
    regex_line_matcher!(r"^error: can't find Rust compiler", |_| Ok(Some(Box::new(MissingRustCompiler)))),
    regex_line_matcher!(r"^Found no assembler", |_| Ok(Some(Box::new(MissingAssembler)))),
    regex_line_matcher!(r"^error: failed to get `(.*)` as a dependency of package `(.*)`", |m| Ok(Some(Box::new(MissingCargoCrate::simple(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"^configure: error: (.*) requires libkqueue \(or system kqueue\). .*", |_| Ok(Some(Box::new(MissingPkgConfig::simple("libkqueue".to_string()))))),
    regex_line_matcher!(r"^Did not find pkg-config by name 'pkg-config'", |_| Ok(Some(Box::new(MissingCommand("pkg-config".to_string()))))),
    regex_line_matcher!(r"^configure: error: Required (.*) binary is missing. Please install (.*).", |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r#".*meson.build:([0-9]+):([0-9]+): ERROR: Dependency "(.*)" not found"#, |m| Ok(Some(Box::new(MissingPkgConfig::simple(m.get(3).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r".*meson.build:([0-9]+):([0-9]+): ERROR: Problem encountered: No XSLT processor found, .*", |_| Ok(Some(Box::new(MissingVagueDependency::simple("xsltproc"))))),
    regex_line_matcher!(r".*meson.build:([0-9]+):([0-9]+): Unknown compiler\(s\): \[\['(.*)'.*\]", |m| Ok(Some(Box::new(MissingCommand(m.get(3).unwrap().as_str().to_string()))))),
    regex_line_matcher!(".*meson.build:([0-9]+):([0-9]+): ERROR: python3 \"(.*)\" missing", |m| Ok(Some(Box::new(MissingPythonModule {
        module: m.get(3).unwrap().as_str().to_string(),
        python_version: Some(3),
        minimum_version: None,
    })))),
    regex_line_matcher!(".*meson.build:([0-9]+):([0-9]+): ERROR: Program \'(.*)\' not found", |m| Ok(Some(Box::new(MissingCommand(m.get(3).unwrap().as_str().to_string()))))),
    regex_line_matcher!(".*meson.build:([0-9]+):([0-9]+): ERROR: Git program not found, .*", |_| Ok(Some(Box::new(MissingCommand("git".to_string()))))),
    regex_line_matcher!(".*meson.build:([0-9]+):([0-9]+): ERROR: C header \'(.*)\' not found", |m| Ok(Some(Box::new(MissingCHeader::new(m.get(3).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"^configure: error: (.+\.h) could not be found\. Please set CPPFLAGS\.", |m| Ok(Some(Box::new(MissingCHeader::new(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r".*meson.build:([0-9]+):([0-9]+): ERROR: Unknown compiler\(s\): \['(.*)'\]", |m| Ok(Some(Box::new(MissingCommand(m.get(3).unwrap().as_str().to_string()))))),
    regex_line_matcher!(".*meson.build:([0-9]+):([0-9]+): ERROR: Dependency \"(.*)\" not found, tried pkgconfig", |m| Ok(Some(Box::new(MissingPkgConfig::simple(m.get(3).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r#".*meson.build:([0-9]+):([0-9]+): ERROR: Could not execute Vala compiler "(.*)""#, |m| Ok(Some(Box::new(MissingCommand(m.get(3).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r".*meson.build:([0-9]+):([0-9]+): ERROR: python3 is missing modules: (.*)", |m| Ok(Some(Box::new(MissingPythonModule::simple(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r".*meson.build:([0-9]+):([0-9]+): ERROR: Invalid version of dependency, need '([^']+)' \['>=\s*([^']+)'\] found '([^']+)'\.", |m| Ok(Some(Box::new(MissingPkgConfig::new(m.get(3).unwrap().as_str().to_string(), Some(m.get(4).unwrap().as_str().to_string())))))),
    regex_line_matcher!(".*meson.build:([0-9]+):([0-9]+): ERROR: C shared or static library '(.*)' not found", |m| Ok(Some(Box::new(MissingLibrary(m.get(3).unwrap().as_str().to_string()))))),
    regex_line_matcher!(".*meson.build:([0-9]+):([0-9]+): ERROR: C\\+\\++ shared or static library '(.*)' not found", |m| Ok(Some(Box::new(MissingLibrary(m.get(3).unwrap().as_str().to_string()))))),
    regex_line_matcher!(".*meson.build:([0-9]+):([0-9]+): ERROR: Pkg-config binary for machine .* not found. Giving up.", |_| Ok(Some(Box::new(MissingCommand("pkg-config".to_string()))))),
    regex_line_matcher!(".*meson.build([0-9]+):([0-9]+): ERROR: Problem encountered: (.*) require (.*) >= (.*), (.*) which were not found.", |m| Ok(Some(Box::new(MissingVagueDependency{name: m.get(4).unwrap().as_str().to_string(), current_version: None, url: None, minimum_version: Some(m.get(5).unwrap().as_str().to_string())})))),
    regex_line_matcher!(".*meson.build([0-9]+):([0-9]+): ERROR: Problem encountered: (.*) is required to .*", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(4).unwrap().as_str()))))),
    regex_line_matcher!(r"^ERROR: (.*) is not installed\. Install at least (.*) version (.+) to continue\.", |m| Ok(Some(Box::new(MissingVagueDependency {
        name: m.get(1).unwrap().as_str().to_string(),
        minimum_version: Some(m.get(3).unwrap().as_str().to_string()),
        current_version: None,
        url: None,
    })))),
    regex_line_matcher!(r"^configure: error: Library requirements \((.*)\) not met\.", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(r"^configure: error: (.*) is missing -- (.*)", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(r"^configure: error: Cannot find (.*), check (.*)", |m| Ok(Some(Box::new(MissingVagueDependency {
        name: m.get(1).unwrap().as_str().to_string(),
        url: Some(m.get(2).unwrap().as_str().to_string()),
        minimum_version: None,
        current_version: None
    })))),
    regex_line_matcher!(r"^configure: error: \*\*\* Unable to find (.* library)", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(r"^configure: error: unable to find (.*)\.", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(r"^configure: error: Perl Module (.*) not available", |m| Ok(Some(Box::new(MissingPerlModule::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(r"(.*) was not found in your path\. Please install (.*)", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(r"^configure: error: Please install (.*) >= (.*)", |m| Ok(Some(Box::new(MissingVagueDependency {
        name: m.get(1).unwrap().as_str().to_string(),
        minimum_version: Some(m.get(2).unwrap().as_str().to_string()),
        current_version: None,
        url: None
    })))),
    regex_line_matcher!(
        r"^configure: error: the required package (.*) is not installed", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(r"^configure: error: \*\*\* (.*) >= (.*) not installed.*", |m| Ok(Some(Box::new(MissingVagueDependency {
        name: m.get(1).unwrap().as_str().to_string(),
        minimum_version: Some(m.get(2).unwrap().as_str().to_string()),
        current_version: None,
        url: None
    })))),
    regex_line_matcher!(r"^configure: error: you should install (.*) first", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(r"^configure: error: cannot locate (.*) >= (.*)", |m| Ok(Some(Box::new(MissingVagueDependency {
        name: m.get(1).unwrap().as_str().to_string(),
        minimum_version: Some(m.get(2).unwrap().as_str().to_string()),
        current_version: None,
        url: None
    })))),
    regex_line_matcher!(r"^configure: error: !!! Please install (.*) !!!", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(r"^configure: error: (.*) version (.*) or higher is required", |m| Ok(Some(Box::new(MissingVagueDependency {
        name: m.get(1).unwrap().as_str().to_string(),
        minimum_version: Some(m.get(2).unwrap().as_str().to_string()),
        current_version: None,
        url: None
    })))),
    regex_line_matcher!(r"^configure.(ac|in):[0-9]+: error: libtool version (.*) or higher is required", |m| Ok(Some(Box::new(MissingVagueDependency {
        name: m.get(2).unwrap().as_str().to_string(),
        minimum_version: Some(m.get(3).unwrap().as_str().to_string()),
        current_version: None,
        url: None
    })))),
    regex_line_matcher!(r"configure: error: ([^ ]+) ([^ ]+) or better is required.*", |m| Ok(Some(Box::new(MissingVagueDependency {
        name: m.get(1).unwrap().as_str().to_string(),
        minimum_version: Some(m.get(2).unwrap().as_str().to_string()),
        current_version: None,
        url: None
    })))),
    regex_line_matcher!(r"configure: error: ([^ ]+) ([^ ]+) or greater is required.*", |m| Ok(Some(Box::new(MissingVagueDependency {
        name: m.get(1).unwrap().as_str().to_string(),
        minimum_version: Some(m.get(2).unwrap().as_str().to_string()),
        current_version: None,
        url: None
    })))),
    regex_line_matcher!(r"configure: error: ([^ ]+) or greater is required.*", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(
        r"configure: error: (.*) library is required",
        |m| Ok(Some(Box::new(MissingLibrary(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(
        r"configure: error: (.*) library is not installed\.",
        |m| Ok(Some(Box::new(MissingLibrary(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(
        r"configure: error: OpenSSL developer library 'libssl-dev' or 'openssl-devel' not installed; cannot continue.",
        |_m| Ok(Some(Box::new(MissingLibrary("ssl".to_string()))))),
    regex_line_matcher!(
        r"configure: error: \*\*\* Cannot find (.*)",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(
        r"configure: error: (.*) is required to compile .*",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"\s*You must have (.*) installed to compile .*\.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"You must install (.*) to compile (.*)",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"\*\*\* No (.*) found, please in(s?)tall it \*\*\*",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"configure: error: (.*) required, please in(s?)tall it",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"\*\* ERROR \*\* : You must have `(.*)' installed on your system\.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"autogen\.sh: ERROR: You must have `(.*)' installed to compile this package\.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"autogen\.sh: You must have (.*) installed\.", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"\s*Error! You need to have (.*) installed\.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"(configure: error|\*\*Error\*\*): You must have (.*) installed.*",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(2).unwrap().as_str()))))),

    regex_line_matcher!(
        r"configure: error: (.*) is required for building this package.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"configure: error: (.*) is required to build (.*)",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"configure: error: (.*) is required",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"configure: error: (.*) is required for (.*)",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"configure: error: \*\*\* (.*) is required\.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"configure: error: (.*) is required, please get it from (.*)",
        |m| Ok(Some(Box::new(MissingVagueDependency{
            name: m.get(1).unwrap().as_str().to_string(),
            url: Some(m.get(2).unwrap().as_str().to_string()),
            minimum_version: None, current_version: None})))),
    regex_line_matcher!(
        r".*meson.build:\d+:\d+: ERROR: Assert failed: (.*) support explicitly required, but (.*) not found",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"configure: error: .*, (lib[^ ]+) is required",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"dh: Unknown sequence --(.*) \(options should not come before the sequence\)",
        |_| Ok(Some(Box::new(DhWithOrderIncorrect)))),
    regex_line_matcher!(
        r"(dh: |dh_.*: error: )Compatibility levels before ([0-9]+) are no longer supported \(level ([0-9]+) requested\)",
        |m| {
            let l1 = m.get(2).unwrap().as_str().parse().unwrap();
            let l2 = m.get(3).unwrap().as_str().parse().unwrap();
            Ok(Some(Box::new(UnsupportedDebhelperCompatLevel::new(l1, l2))))
        }
    ),
    regex_line_matcher!(r"\{standard input\}: Error: (.*)"),
    regex_line_matcher!(r"dh: Unknown sequence (.*) \(choose from: .*\)"),
    regex_line_matcher!(r".*: .*: No space left on device", |_m| Ok(Some(Box::new(NoSpaceOnDevice)))),
    regex_line_matcher!(r"^No space left on device.", |_m| Ok(Some(Box::new(NoSpaceOnDevice)))),
    regex_line_matcher!(
        r".*Can't locate (.*).pm in @INC \(you may need to install the (.*) module\) \(@INC contains: (.*)\) at .* line [0-9]+\.",
        |m| {
            let path = format!("{}.pm", m.get(1).unwrap().as_str());
            let inc = m.get(3).unwrap().as_str().split(' ').map(|s| s.to_string()).collect::<Vec<_>>();

            Ok(Some(Box::new(MissingPerlModule{ filename: Some(path), module: m.get(2).unwrap().as_str().to_string(), minimum_version: None, inc: Some(inc)})))
        }
    ),
    regex_line_matcher!(
        r".*Can't locate (.*).pm in @INC \(you may need to install the (.*) module\) \(@INC contains: (.*)\)\.",
        |m| {
            let path = format!("{}.pm", m.get(1).unwrap().as_str());
            let inc = m.get(3).unwrap().as_str().split(' ').map(|s| s.to_string()).collect::<Vec<_>>();

            Ok(Some(Box::new(MissingPerlModule{ filename: Some(path), module: m.get(2).unwrap().as_str().to_string(), inc: Some(inc), minimum_version: None })))
        }
    ),
    regex_line_matcher!(
        r"\[DynamicPrereqs\] Can't locate (.*) at inline delegation in .*",
        |m| Ok(Some(Box::new(MissingPerlModule::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r#"Can't locate object method "(.*)" via package "(.*)" \(perhaps you forgot to load "(.*)"\?\) at .*.pm line [0-9]+\."#,
        |m| Ok(Some(Box::new(MissingPerlModule::simple(m.get(2).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r">\(error\): Could not expand \[(.*)'",
        |m| Ok(Some(Box::new(MissingPerlModule::simple(m.get(1).unwrap().as_str().trim().trim_matches('\'')))))),

    regex_line_matcher!(
        r"\[DZ\] could not load class (.*) for license (.*)",
        |m| Ok(Some(Box::new(MissingPerlModule::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"\- ([^\s]+)\s+\.\.\.missing. \(would need (.*)\)",
        |m| Ok(Some(Box::new(MissingPerlModule {
            filename: None,
            module: m.get(1).unwrap().as_str().to_string(),
            inc: None,
            minimum_version: Some(m.get(2).unwrap().as_str().to_string()),
        })))),

    regex_line_matcher!(
        r"Required plugin bundle ([^ ]+) isn't installed.",
        |m| Ok(Some(Box::new(MissingPerlModule::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"Required plugin ([^ ]+) isn't installed.",
        |m| Ok(Some(Box::new(MissingPerlModule::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r".*Can't locate (.*) in @INC \(@INC contains: (.*)\) at .* line .*.",
        |m| {
            let inc = m.get(2).unwrap().as_str().split(' ').map(|s| s.to_string()).collect::<Vec<_>>();
            Ok(Some(Box::new(MissingPerlFile::new(m.get(1).unwrap().as_str().to_string(), Some(inc)))))
        }),

    regex_line_matcher!(
        r"Can't find author dependency ([^ ]+) at (.*) line ([0-9]+).",
        |m| Ok(Some(Box::new(MissingPerlModule::simple(m.get(1).unwrap().as_str()))))),

    regex_line_matcher!(
        r"Can't find author dependency ([^ ]+) version (.*) at (.*) line ([0-9]+).",
        |m| Ok(Some(Box::new(MissingPerlModule {
            filename: None,
            module: m.get(1).unwrap().as_str().to_string(),
            inc: None,
            minimum_version: Some(m.get(2).unwrap().as_str().to_string()),
        })))),
    regex_line_matcher!(
        r"> Could not find (.*)\. Please check that (.*) contains a valid JDK installation.",
        |m| Ok(Some(Box::new(MissingJDKFile::new(m.get(2).unwrap().as_str().to_string(), m.get(1).unwrap().as_str().to_string()))))),

    regex_line_matcher!(
        r"> Could not find (.*)\. Please check that (.*) contains a valid \(and compatible\) JDK installation.",
        |m| Ok(Some(Box::new(MissingJDKFile::new(m.get(2).unwrap().as_str().to_string(), m.get(1).unwrap().as_str().to_string()))))),

    regex_line_matcher!(
        r"> Kotlin could not find the required JDK tools in the Java installation '(.*)' used by Gradle. Make sure Gradle is running on a JDK, not JRE.",
        |m| Ok(Some(Box::new(MissingJDK::new(m.get(1).unwrap().as_str().to_string()))))),

    regex_line_matcher!(
        r"> JDK_5 environment variable is not defined. It must point to any JDK that is capable to compile with Java 5 target \((.*)\)",
        |m| Ok(Some(Box::new(MissingJDK::new(m.get(1).unwrap().as_str().to_string()))))),

    regex_line_matcher!(
        r"ERROR: JAVA_HOME is not set and no 'java' command could be found in your PATH.",
        |_| Ok(Some(Box::new(MissingJRE)))),

    regex_line_matcher!(
        r#"Error: environment variable "JAVA_HOME" must be set to a JDK \(>= v(.*)\) installation directory"#,
        |m| Ok(Some(Box::new(MissingJDK::new(m.get(1).unwrap().as_str().to_string()))))),

    regex_line_matcher!(
        r"(?:/usr/bin/)?install: cannot create regular file '(.*)': No such file or directory",
        file_not_found
    ),
    regex_line_matcher!(
        r"Cannot find source directory \((.*)\)",
        file_not_found
    ),
    regex_line_matcher!(
        r"python[0-9.]*: can't open file '(.*)': \[Errno 2\] No such file or directory",
        file_not_found
    ),
    regex_line_matcher!(
        r"^error: \[Errno 2\] No such file or directory: '(.*)'",
        |m| file_not_found_maybe_executable(m.get(1).unwrap().as_str())
    ),
    regex_line_matcher!(
        r".*:[0-9]+:[0-9]+: ERROR: <ExternalProgram 'python3' -> \['/usr/bin/python3'\]> is not a valid python or it is missing setuptools",
        |_| Ok(Some(Box::new(MissingPythonDistribution {
            distribution: "setuptools".to_string(),
            python_version: Some(3),
            minimum_version: None,
        })))
    ),
    regex_line_matcher!(r"OSError: \[Errno 28\] No space left on device", |_| Ok(Some(Box::new(NoSpaceOnDevice)))),
    // python:setuptools_scm
    regex_line_matcher!(
        r"^LookupError: setuptools-scm was unable to detect version for '.*'\.",
        |_| Ok(Some(Box::new(SetuptoolScmVersionIssue)))
    ),
    regex_line_matcher!(
        r"^LookupError: setuptools-scm was unable to detect version for .*\.",
        |_| Ok(Some(Box::new(SetuptoolScmVersionIssue)))
    ),
    regex_line_matcher!(r"^OSError: 'git' was not found", |_| Ok(Some(Box::new(MissingCommand("git".to_string()))))),
    regex_line_matcher!(r"^OSError: No such file (.*)", |m| file_not_found_maybe_executable(m.get(1).unwrap().as_str())),
    regex_line_matcher!(
        r"^Could not open '(.*)': No such file or directory at /usr/share/perl/[0-9.]+/ExtUtils/MM_Unix.pm line [0-9]+.",
        |m| Ok(Some(Box::new(MissingPerlFile::new(m.get(1).unwrap().as_str().to_string(), None))))
    ),
    regex_line_matcher!(
        r#"^Can't open perl script "(.*)": No such file or directory"#,
        |m| Ok(Some(Box::new(MissingPerlFile::new(m.get(1).unwrap().as_str().to_string(), None))))),
    // Maven
    regex_line_matcher!(
        format!("{}{}", MAVEN_ERROR_PREFIX, r"Failed to execute goal on project .*: \x1b\[1;31mCould not resolve dependencies for project .*: The following artifacts could not be resolved: (.*): Could not find artifact (.*) in (.*) \((.*)\)\x1b\[m -> \x1b\[1m\[Help 1\]\x1b\[m").as_str(), maven_missing_artifact),

    regex_line_matcher!(
        format!("{}{}", MAVEN_ERROR_PREFIX, r"Failed to execute goal on project .*: \x1b\[1;31mCould not resolve dependencies for project .*: Could not find artifact (.*)\x1b\[m .*").as_str(),
        maven_missing_artifact
    ),

    regex_line_matcher!(
        format!("{}{}", MAVEN_ERROR_PREFIX, r"Failed to execute goal on project .*: Could not resolve dependencies for project .*: The following artifacts could not be resolved: (.*): Cannot access central \(https://repo\.maven\.apache\.org/maven2\) in offline mode and the artifact .* has not been downloaded from it before..*").as_str(), maven_missing_artifact
    ),
    regex_line_matcher!(
        format!("{}{}", MAVEN_ERROR_PREFIX, r"Unresolveable build extension: Plugin (.*) or one of its dependencies could not be resolved: Cannot access central \(https://repo.maven.apache.org/maven2\) in offline mode and the artifact .* has not been downloaded from it before. @").as_str(), |m| Ok(Some(Box::new(MissingMavenArtifacts(vec![m.get(1).unwrap().as_str().to_string()]))))),
    regex_line_matcher!(
        format!("{}{}", MAVEN_ERROR_PREFIX, r"Non-resolvable import POM: Cannot access central \(https://repo.maven.apache.org/maven2\) in offline mode and the artifact (.*) has not been downloaded from it before. @ line [0-9]+, column [0-9]+").as_str(), maven_missing_artifact),
    regex_line_matcher!(
        r"\[FATAL\] Non-resolvable parent POM for .*: Cannot access central \(https://repo.maven.apache.org/maven2\) in offline mode and the artifact (.*) has not been downloaded from it before. .*", maven_missing_artifact),
    regex_line_matcher!(
        format!("{}{}", MAVEN_ERROR_PREFIX,r"Plugin (.*) or one of its dependencies could not be resolved: Cannot access central \(https://repo.maven.apache.org/maven2\) in offline mode and the artifact .* has not been downloaded from it before. -> \[Help 1\]").as_str(), |m| Ok(Some(Box::new(MissingMavenArtifacts(vec![m.get(1).unwrap().as_str().to_string()]))))),
    regex_line_matcher!(
        format!("{}{}", MAVEN_ERROR_PREFIX, r"Plugin (.+) or one of its dependencies could not be resolved: Failed to read artifact descriptor for (.*): (.*)").as_str(), |m| Ok(Some(Box::new(MissingMavenArtifacts(vec![m.get(1).unwrap().as_str().to_string()]))))),
    regex_line_matcher!(
        format!("{}{}", MAVEN_ERROR_PREFIX, r"Failed to execute goal on project .*: Could not resolve dependencies for project .*: Cannot access .* \([^\)]+\) in offline mode and the artifact (.*) has not been downloaded from it before. -> \[Help 1\]").as_str(), maven_missing_artifact),
    regex_line_matcher!(
        format!("{}{}", MAVEN_ERROR_PREFIX, r"Failed to execute goal on project .*: Could not resolve dependencies for project .*: Cannot access central \(https://repo.maven.apache.org/maven2\) in offline mode and the artifact (.*) has not been downloaded from it before..*").as_str(), maven_missing_artifact),
    regex_line_matcher!(format!("{}{}", MAVEN_ERROR_PREFIX, "Failed to execute goal (.*) on project (.*): (.*)").as_str(), |_| Ok(None)),
    regex_line_matcher!(
        format!("{}{}", MAVEN_ERROR_PREFIX, r"Error resolving version for plugin \'(.*)\' from the repositories \[.*\]: Plugin not found in any plugin repository -> \[Help 1\]").as_str(),
        |m| Ok(Some(Box::new(MissingMavenArtifacts(vec![m.get(1).unwrap().as_str().to_string()]))))
    ),
    regex_line_matcher!(
        r"E: eatmydata: unable to find '(.*)' in PATH",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"'(.*)' not found in PATH at (.*) line ([0-9]+)\.",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"/usr/bin/eatmydata: [0-9]+: exec: (.*): not found",
        command_missing
    ),
    regex_line_matcher!(
        r"/usr/bin/eatmydata: [0-9]+: exec: (.*): Permission denied",
        |m| Ok(Some(Box::new(NotExecutableFile(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r#"(.*): exec: "(.*)": executable file not found in \$PATH"#,
        |m| Ok(Some(Box::new(MissingCommand(m.get(2).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r#"Can't exec "(.*)": No such file or directory at (.*) line ([0-9]+)\."#,
        command_missing
    ),
    regex_line_matcher!(
        r"dh_missing: (warning: )?(.*) exists in debian/.* but is not installed to anywhere",
        |m| Ok(Some(Box::new(DhMissingUninstalled(m.get(2).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r"dh_link: link destination (.*) is a directory",
                        |m| Ok(Some(Box::new(DhLinkDestinationIsDirectory(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"I/O error : Attempt to load network entity (.*)",
                        |m| Ok(Some(Box::new(MissingXmlEntity::new(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"ccache: error: (.*)",
    |m| Ok(Some(Box::new(CcacheError(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(
        r"dh: The --until option is not supported any longer \(#932537\). Use override targets instead.",
        |_| Ok(Some(Box::new(DhUntilUnsupported::new())))
    ),
    regex_line_matcher!(
        r"dh: unable to load addon (.*): (.*) did not return a true value at \(eval 11\) line ([0-9]+).",
        |m| Ok(Some(Box::new(DhAddonLoadFailure::new(m.get(1).unwrap().as_str().to_string(), m.get(2).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        "ERROR: dependencies (.*) are not available for package [‘'](.*)['’]",
        r_missing_package
    ),
    regex_line_matcher!(
        "ERROR: dependency [‘'](.*)['’] is not available for package [‘'](.*)[’']",
        r_missing_package
    ),
    regex_line_matcher!(
        r"Error in library\(.*\) : there is no package called \'(.*)\'",
        r_missing_package
    ),
    regex_line_matcher!(r"Error in .* : there is no package called \'(.*)\'", r_missing_package),
    regex_line_matcher!(r"there is no package called \'(.*)\'", r_missing_package),
    regex_line_matcher!(
        r"  namespace ‘(.*)’ ([^ ]+) is being loaded, but >= ([^ ]+) is required",
        |m| Ok(Some(Box::new(MissingRPackage{ package: m.get(1).unwrap().as_str().to_string(), minimum_version: Some(m.get(3).unwrap().as_str().to_string())})))
    ),
    regex_line_matcher!(
        r"  namespace ‘(.*)’ ([^ ]+) is already loaded, but >= ([^ ]+) is required",
        |m| Ok(Some(Box::new(MissingRPackage{package: m.get(1).unwrap().as_str().to_string(), minimum_version: Some(m.get(3).unwrap().as_str().to_string())})))
    ),
    regex_line_matcher!(r"b\'convert convert: Unable to read font \((.*)\) \[No such file or directory\]\.\\n\'",
     file_not_found),
    regex_line_matcher!(r"mv: cannot stat \'(.*)\': No such file or directory", file_not_found),
    regex_line_matcher!(r"mv: cannot move \'.*\' to \'(.*)\': No such file or directory", file_not_found),
    regex_line_matcher!(
        r"(/usr/bin/install|mv): will not overwrite just-created \'(.*)\' with \'(.*)\'",
        |_| Ok(None)
    ),
    regex_line_matcher!(r"^IOError: \[Errno 2\] No such file or directory: \'(.*)\'", |m| file_not_found_maybe_executable(m.get(1).unwrap().as_str())),
    regex_line_matcher!(r"^error: \[Errno 2\] No such file or directory: \'(.*)\'", |m| file_not_found_maybe_executable(m.get(1).unwrap().as_str())),
    regex_line_matcher!(r"^E   IOError: \[Errno 2\] No such file or directory: \'(.*)\'", |m| file_not_found_maybe_executable(m.get(1).unwrap().as_str())),
    regex_line_matcher!("FAIL\t(.+\\/.+\\/.+)\t([0-9.]+)s", |_| Ok(None)),
    regex_line_matcher!(
        r#"dh_(.*): Cannot find \(any matches for\) "(.*)" \(tried in (.*)\)"#,
        |m| Ok(Some(Box::new(DebhelperPatternNotFound {
            pattern: m.get(2).unwrap().as_str().to_string(),
            tool: m.get(1).unwrap().as_str().to_string(),
            directories: m.get(3).unwrap().as_str().split(',').map(|s| s.trim().to_string()).collect(),
        })))
    ),
    regex_line_matcher!(
        r#"Can't exec "(.*)": No such file or directory at /usr/share/perl5/Debian/Debhelper/Dh_Lib.pm line [0-9]+."#,
        command_missing
    ),
    regex_line_matcher!(
        r#"Can\'t exec "(.*)": Permission denied at (.*) line [0-9]+\."#,
        |m| Ok(Some(Box::new(NotExecutableFile(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"/usr/bin/fakeroot: [0-9]+: (.*): Permission denied",
        |m| Ok(Some(Box::new(NotExecutableFile(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r".*: error: (.*) command not found", command_missing),
    regex_line_matcher!(r"error: command '(.*)' failed: No such file or directory",
     command_missing),
    regex_line_matcher!(
        r"dh_install: Please use dh_missing --list-missing/--fail-missing instead",
        |_| Ok(None)
    ),

    regex_line_matcher!(
        r#"dh([^:]*): Please use the third-party "pybuild" build system instead of python-distutils"#,
        |_| Ok(None)
    ),
    // A Python error, but not likely to be actionable. The previous line will have the actual line that failed.
    regex_line_matcher!(r"ImportError: cannot import name (.*)", |_| Ok(None)),
    // Rust ?
    regex_line_matcher!(r"\s*= note: /usr/bin/ld: cannot find -l([^ ]+): .*", |m| Ok(Some(Box::new(MissingLibrary(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"\s*= note: /usr/bin/ld: cannot find -l([^ ]+)", |m| Ok(Some(Box::new(MissingLibrary(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"/usr/bin/ld: cannot find -l([^ ]+): .*", |m| Ok(Some(Box::new(MissingLibrary(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"/usr/bin/ld: cannot find -l([^ ]+)", |m| Ok(Some(Box::new(MissingLibrary(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(
        r"Could not find gem \'([^ ]+) \(([^)]+)\)\', which is required by gem.*",
        ruby_missing_gem
    ),
    regex_line_matcher!(
        r"Could not find gem \'([^ \']+)\', which is required by gem.*",
        |m| Ok(Some(Box::new(MissingRubyGem::simple(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"[^:]+:[0-9]+:in \`to_specs\': Could not find \'(.*)\' \(([^)]+)\) among [0-9]+ total gem\(s\) \(Gem::MissingSpecError\)",
        ruby_missing_gem
    ),
    regex_line_matcher!(
        r"[^:]+:[0-9]+:in \`to_specs\': Could not find \'(.*)\' \(([^)]+)\) - .* \(Gem::MissingSpecVersionError\)",
        ruby_missing_gem
    ),
    regex_line_matcher!(
        r"[^:]+:[0-9]+:in \`block in verify_gemfile_dependencies_are_found\!\': Could not find gem \'(.*)\' in any of the gem sources listed in your Gemfile\. \(Bundler::GemNotFound\)",
        |m| Ok(Some(Box::new(MissingRubyGem::simple(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"Exception: (.*) not in path[!.]*",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"Exception: Building sdist requires that ([^ ]+) be installed\.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"[^:]+:[0-9]+:in \`find_spec_for_exe\': can\'t find gem (.*) \(([^)]+)\) with executable (.*) \(Gem::GemNotFoundException\)",
        ruby_missing_gem
    ),
    regex_line_matcher!(
        r".?PHP Fatal error:  Uncaught Error: Class \'(.*)\' not found in (.*):([0-9]+)",
        |m| Ok(Some(Box::new(MissingPhpClass::simple(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r"Caused by: java.lang.ClassNotFoundException: (.*)", |m| Ok(Some(Box::new(MissingJavaClass::simple(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(
        r"\[(.*)\] \t\t:: (.*)\#(.*);\$\{(.*)\}: not found",
        |m| Ok(Some(Box::new(MissingMavenArtifacts(vec![format!("{}:{}:jar:debian", m.get(2).unwrap().as_str(), m.get(3).unwrap().as_str())]))))
    ),
    regex_line_matcher!(
        r"Caused by: java.lang.IllegalArgumentException: Cannot find JAR \'(.*)\' required by module \'(.*)\' using classpath or distribution directory \'(.*)\'",
        |_| Ok(None)
    ),
    regex_line_matcher!(
        r".*\.xml:[0-9]+: Unable to find a javac compiler;",
        |_| Ok(Some(Box::new(MissingJavaClass::simple("com.sun.tools.javac.Main".to_string()))))
    ),
    regex_line_matcher!(
        r#"checking for (.*)\.\.\. configure: error: "Cannot check for existence of module (.*) without pkgconf""#,
        |_| Ok(Some(Box::new(MissingCommand("pkgconf".to_string()))))
    ),
    regex_line_matcher!(
        r"configure: error: Could not find '(.*)' in path\.",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"autoreconf was not found; .*",
        |_| Ok(Some(Box::new(MissingCommand("autoreconf".to_string()))))
    ),
    regex_line_matcher!(r"^g\+\+: error: (.*): No such file or directory", file_not_found),
    regex_line_matcher!(r"strip: \'(.*)\': No such file", file_not_found),
    regex_line_matcher!(
        r"Sprockets::FileNotFound: couldn\'t find file \'(.*)\' with type \'(.*)\'",
        |m| Ok(Some(Box::new(MissingSprocketsFile{ name: m.get(1).unwrap().as_str().to_string(), content_type: m.get(2).unwrap().as_str().to_string()})))
    ),
    regex_line_matcher!(
        r#"xdt-autogen: You must have "(.*)" installed. You can get if from"#,
        |m| Ok(Some(Box::new(MissingXfceDependency::new(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"autogen.sh: You must have GNU autoconf installed.",
        |_| Ok(Some(Box::new(MissingCommand("autoconf".to_string()))))
    ),
    regex_line_matcher!(
        r"\s*You must have (autoconf|automake|aclocal|libtool|libtoolize) installed to compile (.*)\.",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"It appears that Autotools is not correctly installed on this system.",
        |_| Ok(Some(Box::new(MissingCommand("autoconf".to_string()))))
    ),
    regex_line_matcher!(
        r"\*\*\* No autoreconf found \*\*\*",
        |_| Ok(Some(Box::new(MissingCommand("autoreconf".to_string()))))
    ),
    regex_line_matcher!(r"You need to install gnome-common module and make.*", |_| Ok(Some(Box::new(GnomeCommonMissing)))),
    regex_line_matcher!(r"You need to install the gnome-common module and make.*", |_| Ok(Some(Box::new(GnomeCommonMissing)))),
    regex_line_matcher!(
        r"You need to install gnome-common from the GNOME (git|CVS|SVN)",
        |_| Ok(Some(Box::new(GnomeCommonMissing)))
    ),
    regex_line_matcher!(
        r"automake: error: cannot open < (.*): No such file or directory",
        |m| Ok(Some(Box::new(MissingAutomakeInput::new(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"configure(|\.in|\.ac):[0-9]+: error: possibly undefined macro: (.*)",
        |m| Ok(Some(Box::new(MissingAutoconfMacro::new(m.get(2).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"configure.(in|ac):[0-9]+: error: macro (.*) is not defined; is a m4 file missing\?",
        |m| Ok(Some(Box::new(MissingAutoconfMacro::new(m.get(2).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"config.status: error: cannot find input file: `(.*)\'",
        |m| Ok(Some(Box::new(MissingConfigStatusInput::new(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"\*\*\*Error\*\*\*: You must have glib-gettext >= (.*) installed.*",
        |m| Ok(Some(Box::new(MissingGnomeCommonDependency::new("glib-gettext".to_string(), Some(m.get(1).unwrap().as_str().to_string())))))
    ),
    regex_line_matcher!(
        r"ERROR: JAVA_HOME is set to an invalid directory: /usr/lib/jvm/default-java/",
        |_| Ok(Some(Box::new(MissingJVM)))
    ),
    regex_line_matcher!(
        r#"Error: The file "MANIFEST" is missing from this distribution\. The MANIFEST lists all files included in the distribution\."#,
        |_| Ok(Some(Box::new(MissingPerlManifest)))
    ),
    regex_line_matcher!(
        r"dh_installdocs: --link-doc not allowed between (.*) and (.*) \(one is arch:all and the other not\)",
        |_| Ok(None)
    ),
    regex_line_matcher!(
        r"dh: unable to load addon systemd: dh: The systemd-sequence is no longer provided in compat >= 11, please rely on dh_installsystemd instead",
        |_| Ok(None)
    ),
    regex_line_matcher!(
        r"dh: The --before option is not supported any longer \(#932537\). Use override targets instead.",
        |_| Ok(None)
    ),
    regex_line_matcher!(r"\(.*\): undefined reference to `(.*)'", |_| Ok(None)),
    regex_line_matcher!("(.*):([0-9]+): undefined reference to `(.*)'", |_| Ok(None)),
    regex_line_matcher!("(.*):([0-9]+): error: undefined reference to '(.*)'", |_| Ok(None)),
    regex_line_matcher!(
        r"\/usr\/bin\/ld:(.*): multiple definition of `*.\'; (.*): first defined here",
        |_| Ok(None)
    ),
    regex_line_matcher!(r".+\.go:[0-9]+: undefined reference to `(.*)'", |_| Ok(None)),
    regex_line_matcher!(r"ar: libdeps specified more than once", |_| Ok(None)),
    regex_line_matcher!(
        r"\/usr\/bin\/ld: .*\(.*\):\(.*\): multiple definition of `*.\'; (.*):\((.*)\) first defined here",
        |_| Ok(None)
    ),
    regex_line_matcher!(
        r"\/usr\/bin\/ld:(.*): multiple definition of `*.\'; (.*):\((.*)\) first defined here",
        |_| Ok(None)
    ),
    regex_line_matcher!(r"\/usr\/bin\/ld: (.*): undefined reference to `(.*)\'", |_| Ok(None)),
    regex_line_matcher!(r"\/usr\/bin\/ld: (.*): undefined reference to symbol \'(.*)\'", |_| Ok(None)),
    regex_line_matcher!(
        r"\/usr\/bin\/ld: (.*): relocation (.*) against symbol `(.*)\' can not be used when making a shared object; recompile with -fPIC",
        |_| Ok(None)
    ),
    regex_line_matcher!(
        "(.*):([0-9]+): multiple definition of `(.*)'; (.*):([0-9]+): first defined here",
        |_| Ok(None)
    ),
    regex_line_matcher!(
        "(dh.*): debhelper compat level specified both in debian/compat and via build-dependency on debhelper-compat",
        |m| Ok(Some(Box::new(DuplicateDHCompatLevel::new(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        "(dh.*): (error: )?Please specify the compatibility level in debian/compat",
        |m| Ok(Some(Box::new(MissingDHCompatLevel::new(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        "dh_makeshlibs: The udeb (.*) does not contain any shared libraries but --add-udeb=(.*) was passed!?",
        |_| Ok(None)
    ),
    regex_line_matcher!(
        "dpkg-gensymbols: error: some symbols or patterns disappeared in the symbols file: see diff output below",
        |_| Ok(Some(Box::new(DisappearedSymbols)))
    ),
    regex_line_matcher!(
        r"Failed to copy \'(.*)\': No such file or directory at /usr/share/dh-exec/dh-exec-install-rename line [0-9]+.*",
        file_not_found
    ),
    regex_line_matcher!(r"Invalid gemspec in \[.*\]: No such file or directory - (.*)", command_missing),
    regex_line_matcher!(
        r".*meson.build:[0-9]+:[0-9]+: ERROR: Program\(s\) \[\'(.*)\'\] not found or not executable",
        command_missing
    ),
    regex_line_matcher!(
        r".*meson.build:[0-9]+:[0-9]: ERROR: Git program not found\.",
        |_| Ok(Some(Box::new(MissingCommand("git".to_string()))))
    ),
    regex_line_matcher!(
        r"Failed: [pytest] section in setup.cfg files is no longer supported, change to [tool:pytest] instead.",
        |_| Ok(None)
    ),
    regex_line_matcher!(r"cp: cannot stat \'(.*)\': No such file or directory", file_not_found),
    regex_line_matcher!(r"cp: \'(.*)\' and \'(.*)\' are the same file", |_| Ok(None)),
    regex_line_matcher!(r".?PHP Fatal error: (.*)", |_| Ok(None)),
    regex_line_matcher!(r"sed: no input files", |_| Ok(None)),
    regex_line_matcher!(r"sed: can\'t read (.*): No such file or directory", file_not_found),
    regex_line_matcher!(
        r"ERROR in Entry module not found: Error: Can\'t resolve \'(.*)\' in \'(.*)\'",
        webpack_file_missing
    ),
    regex_line_matcher!(
        r".*:([0-9]+): element include: XInclude error : could not load (.*), and no fallback was found",
        |_| Ok(None)
    ),
    regex_line_matcher!(r"E: Child terminated by signal ‘Terminated’",
     |_| Ok(Some(Box::new(Cancelled)))
     ),
    regex_line_matcher!(r"E: Caught signal ‘Terminated’",
     |_| Ok(Some(Box::new(Cancelled)))
     ),
    regex_line_matcher!(r"E: Failed to execute “(.*)”: No such file or directory", command_missing),
    regex_line_matcher!(r"E ImportError: Bad (.*) executable(\.?)", command_missing),
    regex_line_matcher!(r"E: The Debian version .* cannot be used as an ELPA version.", |_| Ok(None)),
    // ImageMagick
    regex_line_matcher!(
        r"convert convert: Image pixel limit exceeded \(see -limit Pixels\) \(-1\).",
        |_| Ok(None)
    ),
    regex_line_matcher!(r"convert convert: Improper image header \(.*\).", |_| Ok(None)),
    regex_line_matcher!(r"convert convert: invalid primitive argument \([0-9]+\).", |_| Ok(None)),
    regex_line_matcher!(r"convert convert: Unexpected end-of-file \(\)\.", |_| Ok(None)),
    regex_line_matcher!(r"convert convert: Unrecognized option \((.*)\)\.", |_| Ok(None)),
    regex_line_matcher!(r"convert convert: Unrecognized channel type \((.*)\)\.", |_| Ok(None)),
    regex_line_matcher!(
        r"convert convert: Unable to read font \((.*)\) \[No such file or directory\].",
        file_not_found
    ),
    regex_line_matcher!(
        r"convert convert: Unable to open file (.*) \[No such file or directory\]\.",
        file_not_found
    ),
    regex_line_matcher!(
        r"convert convert: No encode delegate for this image format \((.*)\) \[No such file or directory\].",
        |m| Ok(Some(Box::new(ImageMagickDelegateMissing::new(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r"ERROR: Sphinx requires at least Python (.*) to run.", |_| Ok(None)),
    regex_line_matcher!(r"Can\'t find (.*) directory in (.*)", |_| Ok(None)),
    regex_line_matcher!(
        r"/bin/sh: [0-9]: cannot create (.*): Directory nonexistent",
        |m|  Ok(Some(Box::new(DirectoryNonExistant(std::path::Path::new(m.get(1).unwrap().as_str()).to_path_buf().parent().unwrap().display().to_string()))))
    ),
    regex_line_matcher!(r"dh: Unknown sequence (.*) \(choose from: .*\)", |_| Ok(None)),
    regex_line_matcher!(r".*\.vala:[0-9]+\.[0-9]+-[0-9]+.[0-9]+: error: (.*)", |_| Ok(None)),
    regex_line_matcher!(
        r"error: Package `(.*)\' not found in specified Vala API directories or GObject-Introspection GIR directories",
        |m| Ok(Some(Box::new(MissingValaPackage(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r".*.scala:[0-9]+: error: (.*)", |_| Ok(None)),
    // JavaScript
    regex_line_matcher!(r"error TS6053: File \'(.*)\' not found.", file_not_found),
    // Mocha
    regex_line_matcher!(r"Error \[ERR_MODULE_NOT_FOUND\]: Cannot find package '(.*)' imported from (.*)", |m| Ok(Some(Box::new(MissingNodePackage(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"\s*Uncaught Error \[ERR_MODULE_NOT_FOUND\]: Cannot find package '(.*)' imported from (.*)",
    |m| Ok(Some(Box::new(MissingNodePackage(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"(.*\.ts)\([0-9]+,[0-9]+\): error TS[0-9]+: (.*)", |_| Ok(None)),
    regex_line_matcher!(r"(.*.nim)\([0-9]+, [0-9]+\) Error: .*", |_| Ok(None)),
    regex_line_matcher!(
        r"dh_installinit: upstart jobs are no longer supported\!  Please remove (.*) and check if you need to add a conffile removal",
        |m| Ok(Some(Box::new(UpstartFilePresent(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"dh_installinit: --no-restart-on-upgrade has been renamed to --no-stop-on-upgrade",
        |_| Ok(None)
    ),
    regex_line_matcher!(r"find: paths must precede expression: .*", |_| Ok(None)),
    regex_line_matcher!(r"find: ‘(.*)’: No such file or directory", file_not_found),
    regex_line_matcher!(r"ninja: fatal: posix_spawn: Argument list too long", |_| Ok(None)),
    regex_line_matcher!("ninja: fatal: chdir to '(.*)' - No such file or directory", |m| Ok(Some(Box::new(DirectoryNonExistant(m.get(1).unwrap().as_str().to_string()))))),
    // Java
    regex_line_matcher!(r"error: Source option [0-9] is no longer supported. Use [0-9] or later.", |_| Ok(None)),
    regex_line_matcher!(
        r"(dh.*|jh_build): -s/--same-arch has been removed; please use -a/--arch instead",
        |_| Ok(None)
    ),
    regex_line_matcher!(
        r"dh_systemd_start: dh_systemd_start is no longer used in compat >= 11, please use dh_installsystemd instead",
        |_| Ok(None)
    ),
    regex_line_matcher!(r"Trying patch (.*) at level 1 \.\.\. 0 \.\.\. 2 \.\.\. failure.", |_| Ok(None)),
    // QMake
    regex_line_matcher!(r"Project ERROR: (.*) development package not found", pkg_config_missing),
    regex_line_matcher!(r"Package \'(.*)\', required by \'(.*)\', not found\n", pkg_config_missing),
    regex_line_matcher!(r"pkg-config cannot find (.*)", pkg_config_missing),
    regex_line_matcher!(
        r"configure: error: .* not found: Package dependency requirement \'([^\']+)\' could not be satisfied.",
        pkg_config_missing
    ),
    regex_line_matcher!(
        r"configure: error: (.*) is required to build documentation",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(r".*:[0-9]+: (.*) does not exist.", file_not_found),
    // uglifyjs
    regex_line_matcher!(r"ERROR: can\'t read file: (.*)", file_not_found),
    regex_line_matcher!(r#"jh_build: Cannot find \(any matches for\) "(.*)" \(tried in .*\)"#, |_| Ok(None)),
    regex_line_matcher!(
        r"--   Package \'(.*)\', required by \'(.*)\', not found",
        |m| Ok(Some(Box::new(MissingPkgConfig::simple(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r".*.rb:[0-9]+:in `require_relative\': cannot load such file -- (.*) \(LoadError\)",
        |_| Ok(None)
    ),
    regex_line_matcher!(
        r"<internal:.*>:[0-9]+:in `require': cannot load such file -- (.*) \(LoadError\)",
        |m| Ok(Some(Box::new(MissingRubyFile::new(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r".*.rb:[0-9]+:in `require\': cannot load such file -- (.*) \(LoadError\)",
        |m| Ok(Some(Box::new(MissingRubyFile::new(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r"LoadError: cannot load such file -- (.*)", |m| Ok(Some(Box::new(MissingRubyFile::new(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"  cannot load such file -- (.*)", |m| Ok(Some(Box::new(MissingRubyFile::new(m.get(1).unwrap().as_str().to_string()))))),
    // TODO(jelmer): This is a fairly generic string; perhaps combine with other checks for ruby?
    regex_line_matcher!(r"File does not exist: ([a-z/]+)$", |m| Ok(Some(Box::new(MissingRubyFile::new(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(
        r".*:[0-9]+:in `do_check_dependencies\': E: dependency resolution check requested but no working gemspec available \(RuntimeError\)",
        |_| Ok(None)
    ),
    regex_line_matcher!(r"rm: cannot remove \'(.*)\': Is a directory", |_| Ok(None)),
    regex_line_matcher!(r"rm: cannot remove \'(.*)\': No such file or directory", file_not_found),
    // Invalid option from Python
    regex_line_matcher!(r"error: option .* not recognized", |_| Ok(None)),
    // Invalid option from go
    regex_line_matcher!(r"flag provided but not defined: .*", |_| Ok(None)),
    regex_line_matcher!(r#"CMake Error: The source directory "(.*)" does not exist."#, |m| Ok(Some(Box::new(DirectoryNonExistant(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r".*: [0-9]+: cd: can\'t cd to (.*)", |m| Ok(Some(Box::new(DirectoryNonExistant(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"/bin/sh: 0: Can\'t open (.*)", |m| file_not_found_maybe_executable(m.get(1).unwrap().as_str())),
    regex_line_matcher!(r"/bin/sh: [0-9]+: cannot open (.*): No such file", |m| file_not_found_maybe_executable(m.get(1).unwrap().as_str())),
    regex_line_matcher!(r".*: line [0-9]+: (.*): No such file or directory", |m| file_not_found_maybe_executable(m.get(1).unwrap().as_str())),
    regex_line_matcher!(r"/bin/sh: [0-9]+: Syntax error: .*", |_| Ok(None)),
    regex_line_matcher!(r"error: No member named \$memberName", |_| Ok(None)),
    regex_line_matcher!(
        r"(?:/usr/bin/)?install: cannot create regular file \'(.*)\': Permission denied",
        |_| Ok(None)
    ),
    regex_line_matcher!(r"(?:/usr/bin/)?install: cannot create directory .(.*).: File exists", |_| Ok(None)),
    regex_line_matcher!(r"/usr/bin/install: missing destination file operand after .*", |_| Ok(None)),
    // Ruby
    regex_line_matcher!(r"rspec .*\.rb:[0-9]+ # (.*)", |_| Ok(None)),
    // help2man
    regex_line_matcher!(r"Addendum (.*) does NOT apply to (.*) \(translation discarded\).", |_| Ok(None)),
    regex_line_matcher!(
        r"dh_installchangelogs: copy\((.*), (.*)\): No such file or directory",
        file_not_found
    ),
    regex_line_matcher!(r"dh_installman: mv (.*) (.*): No such file or directory", file_not_found),
    regex_line_matcher!(r"dh_installman: Could not determine section for (.*)", |_| Ok(None)),
    regex_line_matcher!(
        r"failed to initialize build cache at (.*): mkdir (.*): permission denied",
        |_| Ok(None)
    ),
    regex_line_matcher!(
        r#"Can't exec "(.*)": No such file or directory at (.*) line ([0-9]+)."#,
        command_missing
    ),
    regex_line_matcher!(
        r#"E OSError: No command "(.*)" found on host .*"#,
        command_missing
    ),
    // PHPUnit
    regex_line_matcher!(r#"Cannot open file "(.*)"."#, file_not_found),
    regex_line_matcher!(
        r".*Could not find a JavaScript runtime\. See https://github.com/rails/execjs for a list of available runtimes\..*",
        |_| Ok(Some(Box::new(MissingJavaScriptRuntime)))
    ),
    Box::new(PythonFileNotFoundErrorMatcher),
    // ruby
    regex_line_matcher!(r"Errno::ENOENT: No such file or directory - (.*)", file_not_found),
    regex_line_matcher!(r"(.*.rb):[0-9]+:in `.*\': .* \(.*\) ", |_| Ok(None)),
    // JavaScript
    regex_line_matcher!(r".*: ENOENT: no such file or directory, open \'(.*)\'", file_not_found),
    regex_line_matcher!(r"\[Error: ENOENT: no such file or directory, stat \'(.*)\'\] \{", file_not_found),
    regex_line_matcher!(
        r"(.*):[0-9]+: error: Libtool library used but \'LIBTOOL\' is undefined",
        |_| Ok(Some(Box::new(MissingLibtool)))
    ),
    // libtoolize
    regex_line_matcher!(r"libtoolize:   error: \'(.*)\' does not exist.", file_not_found),
    // Seen in python-cogent
    regex_line_matcher!(
        "(OSError|RuntimeError): (.*) required but not found.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(2).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"RuntimeError: The (.*) executable cannot be found\. Please check if it is in the system path\.",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_lowercase()))))
    ),
    regex_line_matcher!(
        r".*: [0-9]+: cannot open (.*): No such file",
        file_not_found
    ),
    regex_line_matcher!(
        r"Cannot find Git. Git is required for .*",
        |_| Ok(Some(Box::new(MissingCommand("git".to_string()))))
    ),
    regex_line_matcher!(
        r"E ImportError: Bad (.*) executable\.",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        "RuntimeError: (.*) is missing",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"(OSError|RuntimeError): Could not find (.*) library\..*",
        |m| Ok(Some(Box::new(MissingLibrary(m.get(2).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"(OSError|RuntimeError): We need package (.*), but not importable",
        |m| Ok(Some(Box::new(MissingPythonDistribution{ distribution: m.get(2).unwrap().as_str().to_string(), minimum_version: None, python_version: None })))
    ),
    regex_line_matcher!(
        r"(OSError|RuntimeError): No (.*) was found: .*",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(2).unwrap().as_str()))))
    ),

    regex_line_matcher!(
        r"(.*)meson.build:[0-9]+:[0-9]+: ERROR: Meson version is (.+) but project requires >=\s*(.+)",
        |m| Ok(Some(Box::new(MissingVagueDependency{
            name: "meson".to_string(), url: None,
            minimum_version: Some(m.get(3).unwrap().as_str().trim_end_matches('.').to_string()),
            current_version: Some(m.get(2).unwrap().as_str().to_string())}
        )))
    ),

    // Seen in cpl-plugin-giraf
    regex_line_matcher!(
        r"ImportError: Numpy version (.*) or later must be installed to use .*",
        |m| Ok(Some(Box::new(MissingPythonModule{ module: "numpy".to_string(), python_version: None, minimum_version: Some(m.get(1).unwrap().as_str().to_string())})))
    ),
    // Seen in mayavi2
    regex_line_matcher!(r"\w+Numpy is required to build.*", |_| Ok(Some(Box::new(MissingPythonModule::simple("numpy".to_string()))))),
    // autoconf
    regex_line_matcher!(r"configure.ac:[0-9]+: error: required file \'(.*)\' not found", file_not_found),
    regex_line_matcher!(r"/usr/bin/m4:(.*):([0-9]+): cannot open `(.*)\': No such file or directory", |m| Ok(Some(Box::new(MissingFile{path: std::path::PathBuf::from(m.get(3).unwrap().as_str().to_string())})))),
    // automake
    regex_line_matcher!(r"Makefile.am: error: required file \'(.*)\' not found", file_not_found),
    // sphinx
    regex_line_matcher!(r"config directory doesn\'t contain a conf.py file \((.*)\)", |_| Ok(None)),
    // vcversioner
    regex_line_matcher!(
        r"vcversioner: no VCS could be detected in \'/<<PKGBUILDDIR>>\' and \'/<<PKGBUILDDIR>>/version.txt\' isn\'t present.",
        |_| Ok(None)
    ),
    // rst2html (and other Python?)
    regex_line_matcher!(r"  InputError: \[Errno 2\] No such file or directory: \'(.*)\'", file_not_found),
    // gpg
    regex_line_matcher!(r"gpg: can\'t connect to the agent: File name too long", |_| Ok(None)),
    regex_line_matcher!(r"(.*.lua):[0-9]+: assertion failed", |_| Ok(None)),
    regex_line_matcher!(r"\s+\^\-\-\-\-\^ SC[0-4][0-9][0-9][0-9]: .*", |_| Ok(None)),
    regex_line_matcher!(
        r"Error: (.*) needs updating from (.*)\. Run \'pg_buildext updatecontrol\'.",
        |m| Ok(Some(Box::new(NeedPgBuildExtUpdateControl::new(m.get(1).unwrap().as_str().to_string(), m.get(2).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r"Patch (.*) does not apply \(enforce with -f\)", |m| Ok(Some(Box::new(PatchApplicationFailed::new(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(
        r"java.io.FileNotFoundException: ([^ ]+) \(No such file or directory\)",
        file_not_found
    ),
    // Pytest
    regex_line_matcher!(r"INTERNALERROR> PluginValidationError: (.*)", |_| Ok(None)),
    regex_line_matcher!(r"[0-9]+ out of [0-9]+ hunks FAILED -- saving rejects to file (.*\.rej)", |_| Ok(None)),
    regex_line_matcher!(r"pkg_resources.UnknownExtra: (.*) has no such extra feature \'(.*)\'", |_| Ok(None)),
    regex_line_matcher!(
        r"dh_auto_configure: invalid or non-existing path to the source directory: .*",
        |_| Ok(None)
    ),
    // Sphinx
    regex_line_matcher!(
        r"(.*) is no longer a hard dependency since version (.*). Please install it manually.\(pip install (.*)\)",
        |m| Ok(Some(Box::new(MissingPythonModule::simple(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r"There is a syntax error in your configuration file: (.*)", |_| Ok(None)),
    regex_line_matcher!(
        r"E: The Debian version (.*) cannot be used as an ELPA version.",
        |m| Ok(Some(Box::new(DebianVersionRejected::new(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r#""(.*)" is not exported by the ExtUtils::MakeMaker module"#, |_| Ok(None)),
    regex_line_matcher!(
        r"E: Please add appropriate interpreter package to Build-Depends, see pybuild\(1\) for details\..*",
        |_| Ok(Some(Box::new(DhAddonLoadFailure::new("pybuild".to_string(), "Debian/Debhelper/Buildsystem/pybuild.pm".to_string()))))
    ),
    regex_line_matcher!(r"dpkg: error: .*: No space left on device", |_| Ok(Some(Box::new(NoSpaceOnDevice)))),
    regex_line_matcher!(
        r"You need the GNU readline library\(ftp://ftp.gnu.org/gnu/readline/\s+\) to build",
        |_| Ok(Some(Box::new(MissingLibrary("readline".to_string()))))
    ),
    regex_line_matcher!(
        r"configure: error: Could not find lib(.*)",
        |m| Ok(Some(Box::new(MissingLibrary(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"    Could not find module ‘(.*)’",
        |m| Ok(Some(Box::new(MissingHaskellModule::new(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r"E: session: (.*): Chroot not found", |m| Ok(Some(Box::new(ChrootNotFound::new(m.get(1).unwrap().as_str().to_string()))))),
    Box::new(HaskellMissingDependencyMatcher),
    Box::new(SetupPyCommandMissingMatcher),
    Box::new(CMakeErrorMatcher),
    regex_line_matcher!
    (
        r"error: failed to select a version for the requirement `(.*)`",
        |m| {
                let (crate_name, requirement) = match m.get(1).unwrap().as_str().split_once(" ") {
                    Some((cratename, requirement)) => (cratename.to_string(), Some(requirement.to_string())),
                    None => (m.get(1).unwrap().as_str().to_string(), None),
                };
                Ok(Some(Box::new(MissingCargoCrate {
                    crate_name,
                    requirement,
                }))
            )
        }
    ),
    regex_line_matcher!(r"^Environment variable \$SOURCE_DATE_EPOCH: No digits were found: $"),
    regex_line_matcher!(
        r"\[ERROR\] LazyFont - Failed to read font file (.*) \<java.io.FileNotFoundException: (.*) \(No such file or directory\)\>java.io.FileNotFoundException: (.*) \(No such file or directory\)",
        |m| Ok(Some(Box::new(MissingFile::new(m.get(1).unwrap().as_str().into()))))
    ),
    regex_line_matcher!(r"qt.qpa.xcb: could not connect to display", |_m| Ok(Some(Box::new(MissingXDisplay)))),
    regex_line_matcher!(
        r"\(.*:[0-9]+\): Gtk-WARNING \*\*: [0-9]{2}:[0-9]{2}:[0-9]{2}\.[0-9]{3}: cannot open display: ",
        |_m| Ok(Some(Box::new(MissingXDisplay)))
    ),
    regex_line_matcher!(
        r"\s*Package (.*) was not found in the pkg-config search path.",
        |m| Ok(Some(Box::new(MissingPkgConfig::simple(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"Can't open display",
        |_m| Ok(Some(Box::new(MissingXDisplay)))
    ),
    regex_line_matcher!(
        r"Can't open (.+): No such file or directory.*",
        file_not_found
    ),
    regex_line_matcher!(
        r"pkg-config does not know (.*) at .*\.",
        |m| Ok(Some(Box::new(MissingPkgConfig::simple(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"\*\*\* Please install (.*) \(atleast version (.*)\) or adjust",
        |m| Ok(Some(Box::new(MissingPkgConfig{
            module: m.get(1).unwrap().as_str().to_string(),
            minimum_version: Some(m.get(2).unwrap().as_str().to_string())
        })))
    ),
    regex_line_matcher!(
        r"go runtime is required: https://golang.org/doc/install",
        |_m| Ok(Some(Box::new(MissingGoRuntime)))
    ),
    regex_line_matcher!(
        r"\%Error: '(.*)' must be installed to build",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r#"configure: error: "Could not find (.*) in PATH"#,
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r"Could not find executable (.*)", |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(
        r#"go: .*: Get \"(.*)\": x509: certificate signed by unknown authority"#,
        |m| Ok(Some(Box::new(UnknownCertificateAuthority(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r#".*.go:[0-9]+:[0-9]+: .*: Get \"(.*)\": x509: certificate signed by unknown authority"#,
        |m| Ok(Some(Box::new(UnknownCertificateAuthority(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"fatal: unable to access '(.*)': server certificate verification failed. CAfile: none CRLfile: none",
        |m| Ok(Some(Box::new(UnknownCertificateAuthority(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"curl: \(77\) error setting certificate verify locations:  CAfile: (.*) CApath: (.*)",
        |m| Ok(Some(Box::new(MissingFile::new(m.get(1).unwrap().as_str().to_string().into()))))
    ),
    regex_line_matcher!(
        r"\t\(Do you need to predeclare (.*)\?\)",
        |m| Ok(Some(Box::new(MissingPerlPredeclared(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r#"Bareword \"(.*)\" not allowed while \"strict subs\" in use at Makefile.PL line ([0-9]+)."#,
        |m| Ok(Some(Box::new(MissingPerlPredeclared(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r#"String found where operator expected at Makefile.PL line ([0-9]+), near "([a-z0-9_]+).*""#,
        |m| Ok(Some(Box::new(MissingPerlPredeclared(m.get(2).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r"  vignette builder 'knitr' not found", |_| Ok(Some(Box::new(MissingRPackage::simple("knitr"))))),
    regex_line_matcher!(
        r"fatal: unable to auto-detect email address \(got \'.*\'\)",
        |_m| Ok(Some(Box::new(MissingGitIdentity)))
    ),
    regex_line_matcher!(
        r"E       fatal: unable to auto-detect email address \(got \'.*\'\)",
        |_m| Ok(Some(Box::new(MissingGitIdentity)))
    ),
    regex_line_matcher!(r"gpg: no default secret key: No secret key", |_m| Ok(Some(Box::new(MissingSecretGpgKey)))),
    regex_line_matcher!(
        r"ERROR: FAILED--Further testing stopped: Test requires module \'(.*)\' but it\'s not found",
        |m| Ok(Some(Box::new(MissingPerlModule::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r#"(subprocess.CalledProcessError|error): Command \'\[\'/usr/bin/python([0-9.]*)\', \'-m\', \'pip\', \'--disable-pip-version-check\', \'wheel\', \'--no-deps\', \'-w\', .*, \'([^-][^\']+)\'\]\' returned non-zero exit status 1."#,
        |m| {
            let python_version = m.get(2).filter(|x| !x.is_empty()).map(|pv| pv.as_str().split_once('.').map_or(pv.as_str(), |x| x.0).parse().unwrap());
            Ok(Some(Box::new(MissingPythonDistribution::from_requirement_str(
                m.get(3).unwrap().as_str(), python_version
            ))))
        }
    ),
    regex_line_matcher!(
        r"vcversioner: \[\'git\', .*, \'describe\', \'--tags\', \'--long\'\] failed and \'(.*)/version.txt\' isn\'t present\.",
        |_m| Ok(Some(Box::new(MissingVcVersionerVersion)))
    ),
    regex_line_matcher!(
        r"vcversioner: no VCS could be detected in '(.*)' and '(.*)/version.txt' isn't present\.",
        |_m| Ok(Some(Box::new(MissingVcVersionerVersion)))
    ),
    regex_line_matcher!(
        r"You don't have a working TeX binary \(tex\) installed anywhere in",
        |_m| Ok(Some(Box::new(MissingCommand("tex".to_string()))))
    ),
    regex_line_matcher!(
        r"# Module \'(.*)\' is not installed",
        |m| Ok(Some(Box::new(MissingPerlModule::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r#"Base class package "(.*)" is empty."#,
        |m| Ok(Some(Box::new(MissingPerlModule::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"    \!  (.*::.*) is not installed",
        |m| Ok(Some(Box::new(MissingPerlModule::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"Cannot find (.*) in @INC at (.*) line ([0-9]+)\.",
        |m| Ok(Some(Box::new(MissingPerlModule::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"(.*::.*) (.*) is required to configure our .* dependency, please install it manually or upgrade your CPAN/CPANPLUS",
        |m| Ok(Some(Box::new(MissingPerlModule::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"configure: error: Missing lib(.*)\.",
        |m| Ok(Some(Box::new(MissingLibrary(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"OSError: (.*): cannot open shared object file: No such file or directory",
        |m| Ok(Some(Box::new(MissingFile::new(m.get(1).unwrap().as_str().into()))))
    ),
    regex_line_matcher!(
        r#"The "(.*)" executable has not been found\."#,
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"  '\! LaTeX Error: File `(.*)' not found.'",
        |m| Ok(Some(Box::new(MissingLatexFile(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"\! LaTeX Error: File `(.*)\' not found\.",
        |m| Ok(Some(Box::new(MissingLatexFile(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r#"(\!|.*:[0-9]+:) Package fontspec Error: The font \"(.*)\" cannot be found\."#,
        |m| Ok(Some(Box::new(MissingFontspec(m.get(2).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r"  vignette builder \'(.*)\' not found", |m| Ok(Some(Box::new(MissingRPackage::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(
        r"Error: package [‘'](.*)[’'] (.*) was found, but >= (.*) is required by [‘'](.*)[’']",
        |m| Ok(Some(Box::new(MissingRPackage {
            package: m.get(1).unwrap().as_str().to_string(),
            minimum_version: Some(m.get(3).unwrap().as_str().to_string()),
        })))
    ),
    regex_line_matcher!(r"\s*there is no package called \'(.*)\'", |m| Ok(Some(Box::new(MissingRPackage::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(
        r"Error in .*: there is no package called ‘(.*)’",
        |m| Ok(Some(Box::new(MissingRPackage::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"Exception: cannot execute command due to missing interpreter: (.*)",
        command_missing
    ),
    regex_line_matcher!(
        r"E: Build killed with signal TERM after ([0-9]+) minutes of inactivity",
        |m| Ok(Some(Box::new(InactiveKilled(m.get(1).unwrap().as_str().parse().unwrap()))))
    ),
    regex_line_matcher!(
        r#"\[.*Authority\] PAUSE credentials not found in "config.ini" or "dist.ini" or "~/.pause"\! Please set it or specify an authority for this plugin. at inline delegation in Dist::Zilla::Plugin::Authority for logger->log_fatal \(attribute declared in /usr/share/perl5/Dist/Zilla/Role/Plugin.pm at line [0-9]+\) line [0-9]+\."#, |_m| Ok(Some(Box::new(MissingPauseCredentials)))
    ),
    regex_line_matcher!(
        r"npm ERR\! ERROR: \[Errno 2\] No such file or directory: \'(.*)\'",
        file_not_found
    ),
    regex_line_matcher!(
        r"\*\*\* error: gettext infrastructure mismatch: using a Makefile\.in\.in from gettext version ([0-9.]+) but the autoconf macros are from gettext version ([0-9.]+)",
        |m| Ok(Some(Box::new(MismatchGettextVersions{
            makefile_version: m.get(1).unwrap().as_str().to_string(),
            autoconf_version: m.get(2).unwrap().as_str().to_string(),
        })))
    ),
    regex_line_matcher!(
        r"You need to install the (.*) package to use this program\.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(r"You need to install (.*)", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(
        r"configure: error: You don't seem to have the (.*) library installed\..*",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"configure: error: You need (.*) installed",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"open3: exec of cme (.*) failed: No such file or directory at .*/Dist/Zilla/Plugin/Run/Role/Runner.pm line [0-9]+\.",
        |m| Ok(Some(Box::new(MissingPerlModule::simple(&format!("App::Cme::Command::{}", m.get(1).unwrap().as_str())))))
    ),
    regex_line_matcher!(
        r"pg_ctl: cannot be run as (.*)",
        |m| Ok(Some(Box::new(InvalidCurrentUser(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"([^ ]+) \(for section ([^ ]+)\) does not appear to be installed",
        |m| Ok(Some(Box::new(MissingPerlModule::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"(.*) version (.*) required--this is only version (.*) at .*\.pm line [0-9]+\.",
        |m| Ok(Some(Box::new(MissingPerlModule {
            module: m.get(1).unwrap().as_str().to_string(),
            minimum_version: Some(m.get(2).unwrap().as_str().to_string()),
            inc: None,
            filename: None,
        })))
    ),
    regex_line_matcher!(
        r"Bailout called\.  Further testing stopped:  YOU ARE MISSING REQUIRED MODULES: \[ ([^,]+)(.*) \]:",
        |m| Ok(Some(Box::new(MissingPerlModule::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r#"CMake Error: CMake was unable to find a build program corresponding to "(.*)".  CMAKE_MAKE_PROGRAM is not set\.  You probably need to select a different build tool\."#,
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"Dist currently only works with Git or Mercurial repos",
        |_| Ok(Some(Box::new(VcsControlDirectoryNeeded::new(vec!["git", "hg"]))))
    ),
    regex_line_matcher!(
        r"GitHubMeta: need a .git\/config file, and you don\'t have one",
        |_| Ok(Some(Box::new(VcsControlDirectoryNeeded::new(vec!["git"]))))
    ),
    regex_line_matcher!(
        r"Exception: Versioning for this project requires either an sdist tarball, or access to an upstream git repository\. It's also possible that there is a mismatch between the package name in setup.cfg and the argument given to pbr\.version\.VersionInfo\. Project name .* was given, but was not able to be found\.",
        |_| Ok(Some(Box::new(VcsControlDirectoryNeeded::new(vec!["git"]))))
    ),
    regex_line_matcher!(
        r"configure: error: no suitable Python interpreter found",
        |_| Ok(Some(Box::new(MissingCommand("python".to_string()))))
    ),
    regex_line_matcher!(r#"Could not find external command "(.*)""#, |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(
        r"  Failed to find (.*) development headers\.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"\*\*\* \Subdirectory \'(.*)\' does not yet exist. Use \'./gitsub.sh pull\' to create it, or set the environment variable GNULIB_SRCDIR\.",
        |m| Ok(Some(Box::new(MissingGnulibDirectory(m.get(1).unwrap().as_str().into()))))
    ),
    regex_line_matcher!(
        r"configure: error: Cap\'n Proto compiler \(capnp\) not found.",
        |_| Ok(Some(Box::new(MissingCommand("capnp".to_string()))))
    ),
    regex_line_matcher!(
        r"lua: (.*):(\d+): module \'(.*)\' not found:",
        |m| Ok(Some(Box::new(MissingLuaModule(m.get(3).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r"Unknown key\(s\) in sphinx_gallery_conf:"),
    regex_line_matcher!(r"(.+\.gir):In (.*): error: (.*)"),
    regex_line_matcher!(r"(.+\.gir):[0-9]+\.[0-9]+-[0-9]+\.[0-9]+: error: (.*)"),
    regex_line_matcher!(r"psql:.*\.sql:[0-9]+: ERROR:  (.*)"),
    regex_line_matcher!(r"intltoolize: \'(.*)\' is out of date: use \'--force\' to overwrite"),
    regex_line_matcher!(
        r"E: pybuild pybuild:[0-9]+: cannot detect build system, please use --system option or set PYBUILD_SYSTEM env\. variable"
    ),
    regex_line_matcher!(
        r"--   Requested \'(.*) >= (.*)\' but version of (.*) is (.*)",
        |m| Ok(Some(Box::new(MissingPkgConfig{
            module: m.get(1).unwrap().as_str().to_string(),
            minimum_version: Some(m.get(2).unwrap().as_str().to_string()),
        })))
    ),
    regex_line_matcher!(
        r".*Could not find (.*) lib/headers, please set .* or ensure (.*).pc is in PKG_CONFIG_PATH\.",
        |m| Ok(Some(Box::new(MissingPkgConfig::simple(m.get(2).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"go: go.mod file not found in current directory or any parent directory; see \'go help modules\'",
        |_| Ok(Some(Box::new(MissingGoModFile)))
    ),
    regex_line_matcher!(
        r"go: cannot find main module, but found Gopkg.lock in (.*)",
        |_| Ok(Some(Box::new(MissingGoModFile)))
    ),
    regex_line_matcher!(r"go: updates to go.mod needed; to update it:", |_| Ok(Some(Box::new(OutdatedGoModFile)))),
    regex_line_matcher!(r"(c\+\+|collect2|cc1|g\+\+): fatal error: .*"),
    regex_line_matcher!(r"fatal: making (.*): failed to create tests\/decode.trs"),
    // ocaml
    regex_line_matcher!(r"Please specify at most one of .*"),
    // Python lint
    regex_line_matcher!(r".*\.py:[0-9]+:[0-9]+: [A-Z][0-9][0-9][0-9] .*"),
    regex_line_matcher!(
        r#"PHPUnit requires the "(.*)" extension\."#,
        |m| Ok(Some(Box::new(MissingPHPExtension(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r#"     \[exec\] PHPUnit requires the "(.*)" extension\."#,
        |m| Ok(Some(Box::new(MissingPHPExtension(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r".*/gnulib-tool: \*\*\* minimum supported autoconf version is (.*)\. ",
        |m| Ok(Some(Box::new(MinimumAutoconfTooOld(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"configure.(ac|in):[0-9]+: error: Autoconf version (.*) or higher is required",
        |m| Ok(Some(Box::new(MissingVagueDependency {
            name: "autoconf".to_string(),
            url: None,
            minimum_version: Some(m.get(2).unwrap().as_str().to_string()),
            current_version: None,
        })))
    ),
    regex_line_matcher!(
        r#"# Error: The file "(MANIFEST|META.yml)" is missing from this distribution\\. .*"#,
        |m| Ok(Some(Box::new(MissingPerlDistributionFile(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r"^  ([^ ]+) does not exist$", file_not_found),
    regex_line_matcher!(
        r"\s*> Cannot find \'\.git\' directory",
        |_m| Ok(Some(Box::new(VcsControlDirectoryNeeded::new(vec!["git"]))))
    ),
    regex_line_matcher!(
        r"Unable to find the \'(.*)\' executable\. .*",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"\[@RSRCHBOY\/CopyrightYearFromGit\]  -  412 No \.git subdirectory found",
        |_m| Ok(Some(Box::new(VcsControlDirectoryNeeded::new(vec!["git"]))))
    ),
    regex_line_matcher!(
        r"Couldn\'t find version control data \(git/hg/bzr/svn supported\)",
        |_m| Ok(Some(Box::new(VcsControlDirectoryNeeded::new(vec!["git", "hg", "bzr", "svn"]))))
    ),
    regex_line_matcher!(
        r"RuntimeError: Unable to determine package version. No local Git clone detected, and no version file found at .*",
        |_m| Ok(Some(Box::new(VcsControlDirectoryNeeded::new(vec!["git"]))))
    ),
    regex_line_matcher!(
        r#""(.*)" failed to start: "No such file or directory" at .*.pm line [0-9]+\."#,
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r"Can\'t find ([^ ]+)\.", |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(r"Error: spawn (.*) ENOENT", |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(
        r"E ImportError: Failed to initialize: Bad (.*) executable\.",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r#"ESLint couldn\'t find the config "(.*)" to extend from\. Please check that the name of the config is correct\."#
    ),
    regex_line_matcher!(
        r#"E OSError: no library called "cairo-2" was found"#,
        |m| Ok(Some(Box::new(MissingLibrary(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"ERROR: \[Errno 2\] No such file or directory: '(.*)'",
 |m| file_not_found_maybe_executable(m.get(1).unwrap().as_str())
    ),
    regex_line_matcher!(
        r"error: \[Errno 2\] No such file or directory: '(.*)'",
 |m| file_not_found_maybe_executable(m.get(1).unwrap().as_str())
    ),
    regex_line_matcher!(
        r"We need the Python library (.+) to be installed\. .*",
        |m| Ok(Some(Box::new(MissingPythonDistribution::simple(m.get(1).unwrap().as_str()))))
    ),
    // Waf
    regex_line_matcher!(
        r"Checking for header (.+\.h|.+\.hpp)\s+: not found ",
        |m| Ok(Some(Box::new(MissingCHeader::new(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"000: File does not exist (.*)",
        file_not_found
    ),
    regex_line_matcher!(
        r"ERROR: Coverage for lines \(([0-9.]+)%\) does not meet global threshold \(([0-9]+)%\)",
        |m| Ok(Some(Box::new(CodeCoverageTooLow{
            actual: m.get(1).unwrap().as_str().parse().unwrap(),
            required: m.get(2).unwrap().as_str().parse().unwrap()})))
    ),
    regex_line_matcher!(
        r"Error \[ERR_REQUIRE_ESM\]: Must use import to load ES Module: (.*)",
        |m| Ok(Some(Box::new(ESModuleMustUseImport(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r".* (/<<BUILDDIR>>/.*): No such file or directory", file_not_found),
    regex_line_matcher!(
        r"Cannot open file `(.*)' in mode `(.*)' \(No such file or directory\)",
        file_not_found
    ),
    regex_line_matcher!(r"[^:]+: cannot stat \'(.*)\': No such file or directory", file_not_found),
    regex_line_matcher!(r"cat: (.*): No such file or directory", file_not_found),
    regex_line_matcher!(r"ls: cannot access \'(.*)\': No such file or directory", file_not_found),
    regex_line_matcher!(
        r"Problem opening (.*): No such file or directory at (.*) line ([0-9]+)\.",
        file_not_found
    ),
    regex_line_matcher!(r"/bin/bash: (.*): No such file or directory", file_not_found),
    regex_line_matcher!(
        r#"\(The package "(.*)" was not found when loaded as a Node module from the directory ".*"\.\)"#,
        |m| Ok(Some(Box::new(MissingNodePackage(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r"\+\-\- UNMET DEPENDENCY (.*)", |m| Ok(Some(Box::new(MissingNodePackage(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(
        r"Project ERROR: Unknown module\(s\) in QT: (.*)",
        |m| Ok(Some(Box::new(MissingQtModules(m.get(1).unwrap().as_str().split_whitespace().map(|s| s.to_string()).collect()))))
    ),
    regex_line_matcher!(
        r"(.*):(\d+):(\d+): ERROR: Vala compiler \'.*\' can not compile programs",
        |_| Ok(Some(Box::new(ValaCompilerCannotCompile)))
    ),
    regex_line_matcher!(
        r"(.*):(\d+):(\d+): ERROR: Problem encountered: Cannot load ([^ ]+) library\. (.*)",
        |m| Ok(Some(Box::new(MissingLibrary(m.get(4).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"go: (.*)@(.*): missing go.sum entry; to add it:",
        |m| Ok(Some(Box::new(MissingGoSumEntry {
            package: m.get(1).unwrap().as_str().to_string(),
            version: m.get(2).unwrap().as_str().to_string(),
        })))
    ),
    regex_line_matcher!(
        r"E: pybuild pybuild:(.*): configure: plugin (.*) failed with: PEP517 plugin dependencies are not available\. Please Build-Depend on (.*)\.",
        |m| Ok(Some(Box::new(MissingDebianBuildDep(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"^make\[[0-9]+\]: \*\*\* No rule to make target '(.*)', needed by '(.*)'\.  Stop\.$",
        |m| Ok(Some(Box::new(MissingMakeTarget::new(m.get(1).unwrap().as_str(), Some(m.get(2).unwrap().as_str())))))
    ),
    regex_line_matcher!(r#"make: \*\*\* No rule to make target \'(.*)\'\.  Stop\."#, |m| Ok(Some(Box::new(MissingMakeTarget::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(
        r"make\[[0-9]+\]: \*\*\* No rule to make target \'(.*)\'\.  Stop\.", |m| Ok(Some(Box::new(MissingMakeTarget::simple(m.get(1).unwrap().as_str()))))),
    // ADD NEW REGEXES ABOVE THIS LINE
    regex_line_matcher!(
        r#"configure: error: Can not find "(.*)" .* in your PATH"#,
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    // Intentionally at the bottom of the list.
    regex_line_matcher!(
        r"([^ ]+) package not found\. Please install from (https://[^ ]+)",
        |m| Ok(Some(Box::new(MissingVagueDependency {name:m.get(1).unwrap().as_str().to_string(),url:Some(m.get(2).unwrap().as_str().to_string()), minimum_version: None, current_version: None })))
    ),
    regex_line_matcher!(
        r"([^ ]+) package not found\. Please use \'pip install .*\' first",
        |m| Ok(Some(Box::new(MissingPythonDistribution::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(r".*: No space left on device", |_m| Ok(Some(Box::new(NoSpaceOnDevice)))),
    regex_line_matcher!(r".*(No space left on device).*", |_m| Ok(Some(Box::new(NoSpaceOnDevice)))),
    regex_line_matcher!(
        r"ocamlfind: Package `(.*)\' not found",
        |m| Ok(Some(Box::new(MissingOCamlPackage(m.get(1).unwrap().as_str().to_string()))))
    ),
    // Not a very unique ocaml-specific pattern :(
    regex_line_matcher!(r#"Error: Library "(.*)" not found."#, |m| Ok(Some(Box::new(MissingOCamlPackage(m.get(1).unwrap().as_str().to_string()))))),
    // ADD NEW REGEXES ABOVE THIS LINE
    // Intentionally at the bottom of the list, since they're quite broad.
    regex_line_matcher!(
        r"configure: error: ([^ ]+) development files not found",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"Exception: ([^ ]+) development files not found\..*",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"Exception: Couldn\'t find (.*) source libs\!",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        "configure: error: '(.*)' command was not found",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"configure: error: (.*) not present.*",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"configure: error: (.*) >= (.*) not found",
        |m| Ok(Some(Box::new(MissingVagueDependency {
            name: m.get(1).unwrap().as_str().to_string(),
            minimum_version: Some(m.get(2).unwrap().as_str().to_string()),
            url: None, current_version: None
        })))
    ),
    regex_line_matcher!(
        r"configure: error: (.*) headers (could )?not (be )?found",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"configure: error: (.*) ([0-9].*) (could )?not (be )?found",
        |m| Ok(Some(Box::new(MissingVagueDependency {
            name: m.get(1).unwrap().as_str().to_string(),
            minimum_version: Some(m.get(2).unwrap().as_str().to_string()),
            url: None, current_version: None
        })))
    ),
    regex_line_matcher!(
        r"configure: error: (.*) (could )?not (be )?found",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"configure: error: (.*) ([0-9.]+) is required to build.*",
        |m| Ok(Some(Box::new(MissingVagueDependency {name:m.get(1).unwrap().as_str().to_string(),minimum_version:Some(m.get(2).unwrap().as_str().to_string()),url:None, current_version: None })))
    ),
    regex_line_matcher!(
        ".*meson.build:([0-9]+):([0-9]+): ERROR: Problem encountered: (.*) (.*) or later required",
        |m| Ok(Some(Box::new(MissingVagueDependency {
            name: m.get(3).unwrap().as_str().to_string(),
            minimum_version: Some(m.get(4).unwrap().as_str().to_string()),
                url: None, current_version: None
        })))
    ),
    regex_line_matcher!(
        r"configure: error: Please install (.*) from (http:\/\/[^ ]+)",
        |m| Ok(Some(Box::new(MissingVagueDependency {
            name: m.get(1).unwrap().as_str().to_string(),
            url: Some(m.get(2).unwrap().as_str().to_string()),
            minimum_version: None, current_version: None
        })))
    ),
    regex_line_matcher!(
        r"configure: error: Required package (.*) (is ?)not available\.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"Error\! You need to have (.*) \((.*)\) around.",
        |m| Ok(Some(Box::new(MissingVagueDependency {
            name: m.get(1).unwrap().as_str().to_string(),
            minimum_version: Some(m.get(2).unwrap().as_str().to_string()),
            url: None, current_version: None
        })))
    ),
    regex_line_matcher!(
        r"configure: error: You don\'t have (.*) installed",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"configure: error: Could not find a recent version of (.*)",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"configure: error: Unable to locate (.*)",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"configure: error: Missing the (.* library)",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"configure: error: (.*) requires (.* libraries), .*",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(2).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"configure: error: (.*) requires ([^ ]+)\.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(2).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"(.*) cannot be discovered in ([^ ]+)",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"configure: error: Missing required program '(.*)'.*",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"configure: error: Missing (.*)\.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"configure: error: Unable to find (.*), please install (.*)",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(2).unwrap().as_str()))))
    ),
    regex_line_matcher!(r"configure: error: (.*) Not found", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(
        r"configure: error: You need to install (.*)",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"configure: error: (.*) \((.*)\) not found\.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(2).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"configure: error: (.*) libraries are required for compilation",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"configure: error: .*Make sure you have (.*) installed\.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"error: Cannot find (.*) in the usual places. .*",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r#"Makefile:[0-9]+: \*\*\* "(.*) was not found"\.  Stop\."#,
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r#"Makefile:[0-9]+: \*\*\* \"At least (.*) version (.*) is needed to build (.*)\.".  Stop\."#,
        |m| Ok(Some(Box::new(MissingVagueDependency {
            name: m.get(1).unwrap().as_str().to_string(),
            minimum_version: Some(m.get(2).unwrap().as_str().to_string()),
            url: None, current_version: None
        })))
    ),
    regex_line_matcher!(r"([a-z0-9A-Z]+) not found", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(r"ERROR:  Unable to locate (.*)\.", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(
        "\x1b\\[1;31merror: (.*) not found\x1b\\[0;32m",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"You do not have (.*) correctly installed\. .*",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"Error: (.*) is not available on your system",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"ERROR: (.*) (.*) or later is required",
        |m| Ok(Some(Box::new(MissingVagueDependency {
            name: m.get(1).unwrap().as_str().to_string(),
            minimum_version: Some(m.get(2).unwrap().as_str().to_string()),
            url: None,
            current_version: None
        })))
    ),
    regex_line_matcher!(
        r"configure: error: .*Please install the \'(.*)\' package\.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"Error: Please install ([^ ]+) package",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(r"configure: error: <(.*\.h)> is required", |m| Ok(Some(Box::new(MissingCHeader::new(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(
        r"configure: error: ([^ ]+) is required",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"configure: error: you should install ([^ ]+) first",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"configure: error: .*You need (.*) installed.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(r"To build (.*) you need (.*)", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(r".*Can\'t ([^\. ]+)\. (.*)", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(
        r"([^ ]+) >= (.*) is required",
        |m| Ok(Some(Box::new(MissingVagueDependency {
            name: m.get(1).unwrap().as_str().to_string(),
            minimum_version: Some(m.get(1).unwrap().as_str().to_string()),
            current_version: None,
            url: None
        })))
    ),
    regex_line_matcher!(
        r".*: ERROR: (.*) needs to be installed to run these tests",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(r"ERROR: Unable to locate (.*)\.", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(
        r"ERROR: Cannot find command \'(.*)\' - do you have \'(.*)\' installed and in your PATH\?",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"ValueError: no ([^ ]+) installed, .*",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"This project needs (.*) in order to build\. .*",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(r"ValueError: Unable to find (.+)", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(r"([^ ]+) executable not found\. .*", |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(
        r"ERROR: InvocationError for command could not find executable (.*)",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"E ImportError: Unable to find ([^ ]+) shared library",
        |m| Ok(Some(Box::new(MissingLibrary(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"\s*([^ ]+) library not found on the system",
        |m| Ok(Some(Box::new(MissingLibrary(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r"\s*([^ ]+) library not found(\.?)", |m| Ok(Some(Box::new(MissingLibrary(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(
        r".*Please install ([^ ]+) libraries\.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"Error: Please install (.*) package",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"Please get ([^ ]+) from (www\..*)\.",
        |m| Ok(Some(Box::new(MissingVagueDependency {
            name: m.get(1).unwrap().as_str().to_string(),
            url: Some(m.get(2).unwrap().as_str().to_string()),
            minimum_version: None, current_version: None
        })))
    ),
    regex_line_matcher!(
        r"Please install ([^ ]+) so that it is on the PATH and try again\.",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(
        r"configure: error: No (.*) binary found in (.*)",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r"Could not find ([A-Za-z-]+)$", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(
        r"No ([^ ]+) includes and libraries found",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"Required library (.*) not found\.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(r"Missing ([^ ]+) boost library, .*", |m| Ok(Some(Box::new(MissingLibrary(m.get(1).unwrap().as_str().to_string()))))),
    regex_line_matcher!(
        r"configure: error: ([^ ]+) needed\!",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"\*\*\* (.*) not found, please install it \*\*\*",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"configure: error: could not find ([^ ]+)",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"([^ ]+) is required for ([^ ]+)\.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"configure: error: \*\*\* No ([^.])\! Install (.*) development headers/libraries! \*\*\*",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"configure: error: \'(.*)\' cannot be found",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"No (.*) includes and libraries found",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"\s*No (.*) version could be found in your system\.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(r"You need (.+)", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    regex_line_matcher!(
        r"configure: error: ([^ ]+) is needed",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"configure: error: Cannot find ([^ ]+)\.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"configure: error: ([^ ]+) requested but not installed\.",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"We need the Python library (.+) to be installed\..*",
        |m| Ok(Some(Box::new(MissingPythonDistribution::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"(.*) uses (.*) \(.*\) for installation but (.*) was not found",
        |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
    ),
    regex_line_matcher!(
        r"ERROR: could not locate the \'([^ ]+)\' utility",
        |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
    ),
    regex_line_matcher!(r"Can\'t find (.*) libs. Exiting", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    ]);
}

lazy_static::lazy_static! {
    static ref CMAKE_ERROR_MATCHERS: MatcherGroup = MatcherGroup::new(vec![
        regex_para_matcher!(r"Could NOT find (.*) \(missing:\s(.*)\)\s\(found\ssuitable\sversion\s.*",
            |m| Ok(Some(Box::new(MissingCMakeComponents{
                name: m.get(1).unwrap().as_str().to_string(),
                components: m.get(2).unwrap().as_str().split_whitespace().map(|s| s.to_string()).collect()})))
        ),
        regex_para_matcher!(r"\s*--\s+Package \'(.*)\', required by \'(.*)\', not found",
            |m| Ok(Some(Box::new(MissingPkgConfig::simple(m.get(1).unwrap().as_str().to_string()))))
        ),
        regex_para_matcher!(r#"Could not find a package configuration file provided by\s"(.*)" \(requested\sversion\s(.*)\)\swith\sany\s+of\s+the\s+following\snames:\n\n(  .*\n)+\n.*$"#,
            |m| {
                let package = m.get(1).unwrap().as_str().to_string();
                let version = m.get(2).unwrap().as_str().to_string();
                let _names = m.get(3).unwrap().as_str().split_whitespace().map(|s| s.to_string()).collect::<Vec<_>>();
                Ok(Some(Box::new(MissingCMakeConfig{
                    name: package,
                    version: Some(version),
                })))
            }
        ),
        regex_para_matcher!(
            r"Could NOT find (.*) \(missing: (.*)\)",
            |m| {
                let name = m.get(1).unwrap().as_str().to_string();
                let components = m.get(2).unwrap().as_str().split_whitespace().map(|s| s.to_string()).collect();
                Ok(Some(Box::new(MissingCMakeComponents {
                    name,
                    components,
                })))
            }
        ),
        regex_para_matcher!(
            r#"The (.+) compiler\n\n  "(.*)"\n\nis not able to compile a simple test program\.\n\nIt fails with the following output:\n\n(.*)\n\nCMake will not be able to correctly generate this project.\n$"#,
            |m| {
                let compiler_output = textwrap::dedent(m.get(3).unwrap().as_str());
                let (_match, error) = find_build_failure_description(compiler_output.split_inclusive('\n').collect());
                Ok(error)
            }
        ),
        regex_para_matcher!(
            r#"Could NOT find (.*): Found unsuitable version \"(.*)\",\sbut\srequired\sis\sexact version \"(.*)\" \(found\s(.*)\)"#,
            |m| {
                let package = m.get(1).unwrap().as_str().to_string();
                let version_found = m.get(2).unwrap().as_str().to_string();
                let exact_version_needed = m.get(3).unwrap().as_str().to_string();
                let path = m.get(4).unwrap().as_str().to_string();

                Ok(Some(Box::new(CMakeNeedExactVersion {
                    package,
                    version_found,
                    exact_version_needed,
                    path: std::path::PathBuf::from(path),
                })))
            }
        ),
        regex_para_matcher!(
            r"(.*) couldn't be found \(missing: .*_LIBRARIES .*_INCLUDE_DIR\)",
            |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
        ),
        regex_para_matcher!(
            r#"Could NOT find (.*): Found unsuitable version \"(.*)\",\sbut\srequired\sis\sat\sleast\s\"(.*)\" \(found\s(.*)\)"#,
            |m| Ok(Some(Box::new(MissingPkgConfig{
                module: m.get(1).unwrap().as_str().to_string(),
                minimum_version: Some(m.get(3).unwrap().as_str().to_string())})))
        ),
        regex_para_matcher!(
            r#"The imported target \"(.*)\" references the file\n\n\s*"(.*)"\n\nbut this file does not exist\.(.*)"#,
            |m| Ok(Some(Box::new(MissingFile::new(m.get(2).unwrap().as_str().to_string().into()))))
        ),
        regex_para_matcher!(
            r#"Could not find a configuration file for package "(.*)"\sthat\sis\scompatible\swith\srequested\sversion\s"(.*)"\."#,
            |m| Ok(Some(Box::new(MissingCMakeConfig {
                name: m.get(1).unwrap().as_str().to_string(),
                version: Some(m.get(2).unwrap().as_str().to_string())})))
        ),
        regex_para_matcher!(
            r#".*Could not find a package configuration file provided by "(.*)"\s+with\s+any\s+of\s+the\s+following\s+names:\n\n(  .*\n)+\n.*$"#,
            |m| Ok(Some(Box::new(CMakeFilesMissing{ filenames: m.get(2).unwrap().as_str().split_whitespace().map(|s| s.to_string()).collect(), version: None })))
        ),
        regex_para_matcher!(
            r#".*Could not find a package configuration file provided by "(.*)"\s\(requested\sversion\s(.+\))\swith\sany\sof\sthe\sfollowing\snames:\n\n(  .*\n)+\n.*$"#, |m| {
                let package = m.get(1).unwrap().as_str().to_string();
                let versions = m.get(2).unwrap().as_str().to_string();
                let _names = m.get(3).unwrap().as_str().split_whitespace().map(|s| s.to_string()).collect::<Vec<_>>();
                Ok(Some(Box::new(MissingCMakeConfig {
                    name: package,
                    version: Some(versions),
                })))
            }
        ),
        regex_para_matcher!(
            r#"No CMAKE_(.*)_COMPILER could be found.\n\nTell CMake where to find the compiler by setting either\sthe\senvironment\svariable\s"(.*)"\sor\sthe\sCMake\scache\sentry\sCMAKE_(.*)_COMPILER\sto\sthe\sfull\spath\sto\sthe\scompiler,\sor\sto\sthe\scompiler\sname\sif\sit\sis\sin\sthe\sPATH.\n"#,
            |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_lowercase()))))
        ),
        regex_para_matcher!(r#"file INSTALL cannot find\s"(.*)".\n"#, |m| Ok(Some(Box::new(MissingFile::new(m.get(1).unwrap().as_str().into()))))),
        regex_para_matcher!(
            r#"file INSTALL cannot copy file\n"(.*)"\sto\s"(.*)":\sNo space left on device.\n"#,
            |_m| Ok(Some(Box::new(NoSpaceOnDevice)))
        ),
        regex_para_matcher!(
            r"patch: \*\*\*\* write error : No space left on device", |_| Ok(Some(Box::new(NoSpaceOnDevice)))
        ),
        regex_para_matcher!(
            r".*\(No space left on device\)", |_| Ok(Some(Box::new(NoSpaceOnDevice)))
        ),
        regex_para_matcher!(r#"file INSTALL cannot copy file\n"(.*)"\nto\n"(.*)"\.\n"#),
        regex_para_matcher!(
            r#"Missing (.*)\.  Either your\nlib(.*) version is too old, or lib(.*) wasn\'t found in the place you\nsaid."#,
            |m| Ok(Some(Box::new(MissingLibrary(m.get(1).unwrap().as_str().to_string()))))
        ),
        regex_para_matcher!(
            r"need (.*) of version (.*)",
            |m| Ok(Some(Box::new(MissingVagueDependency{
                name: m.get(1).unwrap().as_str().to_string(),
                minimum_version: Some(m.get(2).unwrap().as_str().to_string()),
                url: None,
                current_version: None
            })))
        ),
        regex_para_matcher!(
            r"\*\*\* (.*) is required to build (.*)\n",
            |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
        ),
        regex_para_matcher!(r"\[([^ ]+)\] not found", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
        regex_para_matcher!(r"([^ ]+) not found", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
        regex_para_matcher!(r"error: could not find git .*", |_m| Ok(Some(Box::new(MissingCommand("git".to_string()))))),
        regex_para_matcher!(
            r"Could not find \'(.*)\' executable[\!,].*",
            |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
        ),
        regex_para_matcher!(
            r"Could not find (.*)_STATIC_LIBRARIES using the following names: ([a-zA-z0-9_.]+)",
            |m| Ok(Some(Box::new(MissingStaticLibrary{
                library: m.get(1).unwrap().as_str().to_string(),
                filename: m.get(2).unwrap().as_str().to_string()})))
        ),
        regex_para_matcher!(
            "include could not find (requested|load) file:\n\n  (.*)\n",
            |m| {
                let mut path = m.get(2).unwrap().as_str().to_string();
                if !path.ends_with(".cmake") {
                    path += ".cmake";
                }
                Ok(Some(Box::new(CMakeFilesMissing{filenames:vec![path], version: None })))
            }
        ),
        regex_para_matcher!(r"(.*) and (.*) are required", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
        regex_para_matcher!(
            r"Please check your (.*) installation",
            |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
        ),
        regex_para_matcher!(r"Python module (.*) not found\!", |m| Ok(Some(Box::new(MissingPythonModule::simple(m.get(1).unwrap().as_str().to_string()))))),
        regex_para_matcher!(r"\s*could not find ([^\s]+)$", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
        regex_para_matcher!(
            r"Please install (.*) before installing (.*)\.",
            |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
        ),
        regex_para_matcher!(
            r"Please get (.*) from (www\..*)",
            |m| Ok(Some(Box::new(MissingVagueDependency {
                name: m.get(1).unwrap().as_str().to_string(),
                url: Some(m.get(2).unwrap().as_str().to_string()),
                minimum_version: None,
                current_version: None
            })))
        ),
        regex_para_matcher!(
            r#"Found unsuitable Qt version "" from NOTFOUND, this code requires Qt 4.x"#,
            |_| Ok(Some(Box::new(MissingQt)))
        ),
        regex_para_matcher!(
            r"(.*) executable not found\! Please install (.*)\.",
            |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))
        ),
        regex_para_matcher!(r"(.*) tool not found", |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))),
        regex_para_matcher!(
            r"--   Requested \'(.*) >= (.*)\' but version of (.*) is (.*)",
            |m| Ok(Some(Box::new(MissingPkgConfig{
                module: m.get(1).unwrap().as_str().to_string(),
                minimum_version: Some(m.get(2).unwrap().as_str().to_string())
            })))
        ),
        regex_para_matcher!(r"--   No package \'(.*)\' found", |m| Ok(Some(Box::new(MissingPkgConfig{minimum_version: None, module: m.get(1).unwrap().as_str().to_string()})))),
        regex_para_matcher!(r"([^ ]+) library not found\.", |m| Ok(Some(Box::new(MissingLibrary(m.get(1).unwrap().as_str().to_string()))))),
        regex_para_matcher!(
            r"Please install (.*) so that it is on the PATH and try again\.",
            command_missing
        ),
        regex_para_matcher!(
            r"-- Unable to find git\.  Setting git revision to \'unknown\'\.",
            |_| Ok(Some(Box::new(MissingCommand("git".to_string()))))
        ),
        regex_para_matcher!(
            r"(.*) must be installed before configuration \& building can proceed",
            |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
        ),
        regex_para_matcher!(
            r"(.*) development files not found\.",
            |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
        ),
        regex_para_matcher!(
            r".* but no (.*) dev libraries found",
            |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
        ),
        regex_para_matcher!(
            r"Failed to find (.*) \(missing: .*\)",
            |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
        ),
        regex_para_matcher!(
            r"Couldn\'t find ([^ ]+) development files\..*",
            |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
        ),
        regex_para_matcher!(
            r"Could not find required (.*) package\!",
            |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
        ),
        regex_para_matcher!(
            r"Cannot find (.*), giving up\. .*",
            |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
        ),
        regex_para_matcher!(
            r"Cannot find (.*)\. (.*) is required for (.*)",
            |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
        ),
        regex_para_matcher!(
            r"The development\sfiles\sfor\s(.*)\sare\srequired\sto\sbuild (.*)\.",
            |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
        ),
        regex_para_matcher!(
            r"Required library (.*) not found\.",
            |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
        ),
        regex_para_matcher!(
            r"(.*) required to compile (.*)",
            |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
        ),
        regex_para_matcher!(
            r"(.*) requires (.*) ([0-9].*) or newer. See (https://.*)\s*",
            |m| Ok(Some(Box::new(MissingVagueDependency {
                name: m.get(2).unwrap().as_str().to_string(),
                minimum_version: Some(m.get(3).unwrap().as_str().to_string()),
                url: Some(m.get(4).unwrap().as_str().to_string()),
                current_version: None
            })))
        ),
        regex_para_matcher!(
            r"(.*) requires (.*) ([0-9].*) or newer.\s*",
            |m| Ok(Some(Box::new(MissingVagueDependency{
                name: m.get(2).unwrap().as_str().to_string(),
                minimum_version: Some(m.get(3).unwrap().as_str().to_string()),
                url: None,
                current_version: None
            })))
        ),
        regex_para_matcher!(r"(.*) requires (.*) to build", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(2).unwrap().as_str()))))),
        regex_para_matcher!(r"(.*) library missing", |m| Ok(Some(Box::new(MissingLibrary(m.get(1).unwrap().as_str().to_string()))))),
        regex_para_matcher!(r"(.*) requires (.*)", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(2).unwrap().as_str()))))),
        regex_para_matcher!(r"Could not find ([A-Za-z-]+)", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
        regex_para_matcher!(r"(.+) is required for (.*)\.", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
        regex_para_matcher!(
            r"No (.+) version could be found in your system\.",
            |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
        ),
        regex_para_matcher!(
            r"([^ ]+) >= (.*) is required",
            |m| Ok(Some(Box::new(MissingVagueDependency {
                name: m.get(1).unwrap().as_str().to_string(),
                minimum_version: Some(m.get(2).unwrap().as_str().to_string()),
                current_version: None,
                url: None
            })))
        ),
        regex_para_matcher!(r"\s*([^ ]+) is required", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
        regex_para_matcher!(r"([^ ]+) binary not found\!", |m| Ok(Some(Box::new(MissingCommand(m.get(1).unwrap().as_str().to_string()))))),
        regex_para_matcher!(r"error: could not find git for clone of .*", |_m| Ok(Some(Box::new(MissingCommand("git".to_string()))))),
        regex_para_matcher!(r"Did not find ([^\s]+)", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
        regex_para_matcher!(
            r"Could not find the ([^ ]+) external dependency\.",
            |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))
        ),
        regex_para_matcher!(r"Couldn\'t find (.*)", |m| Ok(Some(Box::new(MissingVagueDependency::simple(m.get(1).unwrap().as_str()))))),
    ]);
}

#[derive(Debug, Clone)]
struct CMakeErrorMatcher;

// Function to extract error lines and corresponding line numbers
fn extract_cmake_error_lines<'a>(lines: &'a [&'a str], i: usize) -> (Vec<usize>, String) {
    let mut linenos = vec![i];
    let mut error_lines = vec![];

    // Iterate over the lines starting from index i + 1
    for (j, line) in lines.iter().enumerate().skip(i + 1) {
        let trimmed = line.trim_end_matches('\n');
        if !trimmed.is_empty() && !line.starts_with(' ') {
            break;
        }
        error_lines.push(*line);
        linenos.push(j);
    }

    // Remove trailing empty lines from error_lines and linenos
    while let Some(last_line) = error_lines.last() {
        if last_line.trim_end_matches('\n').is_empty() {
            error_lines.pop();
            linenos.pop();
        } else {
            break;
        }
    }

    // Dedent the error_lines using textwrap::dedent
    let dedented_string = textwrap::dedent(&error_lines.join(""));
    (linenos, dedented_string)
}

impl Matcher for CMakeErrorMatcher {
    fn extract_from_lines(
        &self,
        lines: &[&str],
        offset: usize,
    ) -> Result<Option<(Box<dyn Match>, Option<Box<dyn Problem>>)>, Error> {
        let (_path, _start_linenos) = if let Some((_, _, path, start_lineno, _)) = lazy_regex::regex_captures!(
            r"CMake (Error|Warning) at (.+):([0-9]+) \((.*)\):",
            lines[offset].trim_end_matches('\n')
        ) {
            (path, start_lineno.parse::<usize>().unwrap())
        } else {
            return Ok(None);
        };

        let (linenos, error_string) = extract_cmake_error_lines(lines, offset);

        let mut actual_lines: Vec<_> = vec![];
        for lineno in &linenos {
            actual_lines.push(lines[*lineno].to_string());
        }

        let r#match = Box::new(MultiLineMatch::new(
            Origin("CMake".to_string()),
            linenos,
            actual_lines,
        ));

        if let Some((_match, problem)) =
            CMAKE_ERROR_MATCHERS.extract_from_lines(&[&error_string], 0)?
        {
            Ok(Some((r#match, problem)))
        } else {
            Ok(Some((r#match, None)))
        }
    }
}

pub fn match_lines(
    lines: &[&str],
    offset: usize,
) -> Result<Option<(Box<dyn Match>, Option<Box<dyn Problem>>)>, Error> {
    COMMON_MATCHERS.extract_from_lines(lines, offset)
}

macro_rules! secondary_matcher {
    ($re:expr) => {
        fancy_regex::Regex::new($re).unwrap()
    };
}

lazy_static::lazy_static! {
    /// Regexps that hint at an error of some sort, but not the error itself.
    static ref SECONDARY_MATCHERS: Vec<fancy_regex::Regex> = vec![
    secondary_matcher!(r"E: pybuild pybuild:[0-9]+: test: plugin [^ ]+ failed with:"),
    secondary_matcher!(r"[^:]+: error: (.*)"),
    secondary_matcher!(r"[^:]+:[0-9]+: error: (.*)"),
    secondary_matcher!(r"[^:]+:[0-9]+:[0-9]+: error: (.*)"),
    secondary_matcher!(r"error TS[0-9]+: (.*)"),

    secondary_matcher!(r"mount: .*: mount failed: Operation not permitted\."),

    secondary_matcher!(r"  [0-9]+:[0-9]+\s+error\s+.+"),

    secondary_matcher!(r"fontmake: Error: In '(.*)': (.*)"),

    secondary_matcher!(r"#   Failed test at t\/.*\.t line [0-9]+\."),

    secondary_matcher!(r"Gradle build daemon disappeared unexpectedly \(it may have been killed or may have crashed\)"),

    // ocaml
    secondary_matcher!(r"\*\*\* omake error:"),
    secondary_matcher!(r".*ocamlc.*: OCam has been configured with -force-safe-string: -unsafe-string is not available\."),

    // latex
    secondary_matcher!(r"\! LaTeX Error: .*"),

    secondary_matcher!(r"Killed"),

    // Java
    secondary_matcher!(r#"Exception in thread "(.*)" (.*): (.*);"#),
    secondary_matcher!(r"error: Unrecognized option: \'.*\'"),
    secondary_matcher!(r"Segmentation fault"),
    secondary_matcher!(r"\[ERROR\] (.*\.java):\[[0-9]+,[0-9]+\] (.*)"),
    secondary_matcher!(r"make: \*\*\* No targets specified and no makefile found\.  Stop\."),
    secondary_matcher!(r"make\[[0-9]+\]: \*\*\* No targets specified and no makefile found\.  Stop\."),
    secondary_matcher!(r"make\[[0-9]+\]: (.*): No such file or directory"),
    secondary_matcher!(r"make\[[0-9]+\]: \*\*\* \[.*:[0-9]+: .*\] Segmentation fault"),
    secondary_matcher!(
    r".*:[0-9]+: \*\*\* empty variable name.  Stop."),
    secondary_matcher!(
    r"error: can't copy '(.*)': doesn't exist or not a regular file"),
    secondary_matcher!(
    r"error: ([0-9]+) test executed, ([0-9]+) fatal tests failed, "),
    secondary_matcher!(
    r"([0-9]+) nonfatal test failed\."),
    secondary_matcher!(
    r".*\.rst:toctree contains ref to nonexisting file \'.*\'"),
    secondary_matcher!(
    r".*\.rst:[0-9]+:term not in glossary: .*"),
    secondary_matcher!(
    r"Try adding AC_PREREQ\(\[(.*)\]\) to your configure\.ac\."),
    // Erlang
    secondary_matcher!(
    r"  (.*_test): (.+)\.\.\.\*failed\*"),
    secondary_matcher!(
    r"(.*\.erl):[0-9]+:[0-9]+: erlang:.*"),
    // Clojure
    secondary_matcher!(
    r"Could not locate (.*) or (.*) on classpath\."),
    // QMake
    secondary_matcher!(
    r"Project ERROR: .*"),
    // pdflatex
    secondary_matcher!(
    r"\!  ==> Fatal error occurred, no output PDF file produced\!"),
    // latex
    secondary_matcher!(
    r"\! Undefined control sequence\."),
    secondary_matcher!(
    r"\! Emergency stop\."),
    secondary_matcher!(r"\!pdfTeX error: pdflatex: fwrite\(\) failed"),
    // inkscape
    secondary_matcher!(r"Unknown option (?!.*ignoring.*)"),
    // CTest
    secondary_matcher!(
    r"not ok [0-9]+ .*"),
    secondary_matcher!(
    r"Errors while running CTest"),
    secondary_matcher!(
    r"dh_auto_install: error: .*"),
    secondary_matcher!(
    r"dh_quilt_patch: error: (.*)"),
    secondary_matcher!(
    r"dh.*: Aborting due to earlier error"),
    secondary_matcher!(
    r"dh.*: unknown option or error during option parsing; aborting"),
    secondary_matcher!(
    r"Could not import extension .* \(exception: .*\)"),
    secondary_matcher!(
    r"configure.ac:[0-9]+: error: (.*)"),
    secondary_matcher!(
    r"Reconfigure the source tree (via './config' or 'perl Configure'), please."),
    secondary_matcher!(
    r"dwz: Too few files for multifile optimization"),
    secondary_matcher!(
    r"\[CJM/MatchManifest\] Aborted because of MANIFEST mismatch"),
    secondary_matcher!(
    r"dh_dwz: dwz -q -- .* returned exit code [0-9]+"),
    secondary_matcher!(
    r"help2man: can\'t get `-?-help\' info from .*"),
    secondary_matcher!(
    r"[^:]+: line [0-9]+:\s+[0-9]+ Segmentation fault.*"),
    secondary_matcher!(
    r"dpkg-gencontrol: error: (.*)"),
    secondary_matcher!(
    r".*:[0-9]+:[0-9]+: (error|ERROR): (.*)"),
    secondary_matcher!(
    r".*[.]+FAILED .*"),
    secondary_matcher!(
    r"FAIL: (.*)"),
    secondary_matcher!(
    r"FAIL\!  : (.*)"),
    secondary_matcher!(
    r"\s*FAIL (.*) \(.*\)"),
    secondary_matcher!(
    r"FAIL\s+(.*) \[.*\] ?"),
    secondary_matcher!(
    r"([0-9]+)% tests passed, ([0-9]+) tests failed out of ([0-9]+)"),
    secondary_matcher!(
    r"TEST FAILURE"),
    secondary_matcher!(
    r"make\[[0-9]+\]: \*\*\* \[.*\] Error [0-9]+"),
    secondary_matcher!(
    r"make\[[0-9]+\]: \*\*\* \[.*\] Aborted"),
    secondary_matcher!(
    r"exit code=[0-9]+: .*"),
    secondary_matcher!(
    r"chmod: cannot access \'.*\': .*"),
    secondary_matcher!(
    r"dh_autoreconf: autoreconf .* returned exit code [0-9]+"),
    secondary_matcher!(
    r"make: \*\*\* \[.*\] Error [0-9]+"),
    secondary_matcher!(
    r".*:[0-9]+: \*\*\* missing separator\.  Stop\."),
    secondary_matcher!(
    r"[0-9]+ tests: [0-9]+ ok, [0-9]+ failure\(s\), [0-9]+ test\(s\) skipped"),
    secondary_matcher!(
    r"\*\*Error:\*\* (.*)"),
    secondary_matcher!(
    r"^Error: (.*)"),
    secondary_matcher!(
    r"Failed [0-9]+ tests? out of [0-9]+, [0-9.]+% okay."),
    secondary_matcher!(
    r"Failed [0-9]+\/[0-9]+ test programs. [0-9]+/[0-9]+ subtests failed."),
    secondary_matcher!(
    r"Original error was: (.*)"),
    secondary_matcher!(
    r"-- Error \(.*\.R:[0-9]+:[0-9]+\): \(.*\) [-]*"),
    secondary_matcher!(
    r"^Error \[ERR_.*\]: .*"),
    secondary_matcher!(
    r"^FAILED \(.*\)"),
    secondary_matcher!(
    r"FAILED .*"),
    // Random Python errors
    secondary_matcher!(
    "^(E  +)?(SyntaxError|TypeError|ValueError|AttributeError|NameError|django.core.exceptions..*|RuntimeError|subprocess.CalledProcessError|testtools.matchers._impl.MismatchError|PermissionError|IndexError|TypeError|AssertionError|IOError|ImportError|SerialException|OSError|qtawesome.iconic_font.FontError|redis.exceptions.ConnectionError|builtins.OverflowError|ArgumentError|httptools.parser.errors.HttpParserInvalidURLError|HypothesisException|SSLError|KeyError|Exception|rnc2rng.parser.ParseError|pkg_resources.UnknownExtra|tarfile.ReadError|numpydoc.docscrape.ParseError|distutils.errors.DistutilsOptionError|datalad.support.exceptions.IncompleteResultsError|AssertionError|Cython.Compiler.Errors.CompileError|UnicodeDecodeError|UnicodeEncodeError): .*"),
    // Rust
    secondary_matcher!(
    r"error\[E[0-9]+\]: .*"),
    secondary_matcher!(
    "^E   DeprecationWarning: .*"),
    secondary_matcher!(
    "^E       fixture '(.*)' not found"),
    // Rake
    secondary_matcher!(
    r"[0-9]+ runs, [0-9]+ assertions, [0-9]+ failures, [0-9]+ errors, [0-9]+ skips"),
    // Node
    secondary_matcher!(
    r"# failed [0-9]+ of [0-9]+ tests"),
    // Pytest
    secondary_matcher!(
    r"(.*).py:[0-9]+: AssertionError"),
    secondary_matcher!(
    r"============================ no tests ran in ([0-9.]+)s ============================="),
    // Perl
    secondary_matcher!(
    r"  Failed tests:  [0-9-]+"),
    secondary_matcher!(
    r"Failed (.*\.t): output changed"),
    // Go
    secondary_matcher!(
    r"no packages to test"),
    secondary_matcher!(
    "FAIL\t(.*)\t[0-9.]+s"),
    secondary_matcher!(
    r".*.go:[0-9]+:[0-9]+: (?!note:).*"),
    secondary_matcher!(
    r"can\'t load package: package \.: no Go files in /<<PKGBUILDDIR>>/(.*)"),
    // Ld
    secondary_matcher!(
    r"\/usr\/bin\/ld: cannot open output file (.*): No such file or directory"),
    secondary_matcher!(
    r"configure: error: (.+)"),
    secondary_matcher!(
    r"config.status: error: (.*)"),
    secondary_matcher!(
    r"E: Build killed with signal TERM after ([0-9]+) minutes of inactivity"),
    secondary_matcher!(
    r"    \[javac\] [^: ]+:[0-9]+: error: (.*)"),
    secondary_matcher!(
    r"1\) TestChannelFeature: ([^:]+):([0-9]+): assert failed"),
    secondary_matcher!(
    r"cp: target \'(.*)\' is not a directory"),
    secondary_matcher!(
    r"cp: cannot create regular file \'(.*)\': No such file or directory"),
    secondary_matcher!(
    r"couldn\'t determine home directory at (.*)"),
    secondary_matcher!(
    r"ln: failed to create symbolic link \'(.*)\': File exists"),
    secondary_matcher!(
    r"ln: failed to create symbolic link \'(.*)\': No such file or directory"),
    secondary_matcher!(
    r"ln: failed to create symbolic link \'(.*)\': Permission denied"),
    secondary_matcher!(
    r"ln: invalid option -- .*"),
    secondary_matcher!(
    r"mkdir: cannot create directory [‘'](.*)['’]: No such file or directory"),
    secondary_matcher!(
    r"mkdir: cannot create directory [‘'](.*)['’]: File exists"),
    secondary_matcher!(
    r"mkdir: missing operand"),
    secondary_matcher!(
    r"rmdir: failed to remove '.*': No such file or directory"),
    secondary_matcher!(
    r"Fatal error: .*"),
    secondary_matcher!(
    "Fatal Error: (.*)"),
    secondary_matcher!(
    r"Alert: (.*)"),
    secondary_matcher!(
    r#"ERROR: Test "(.*)" failed. Exiting."#),
    // scons
    secondary_matcher!(
    r"ERROR: test\(s\) failed in (.*)"),
    secondary_matcher!(
    r"./configure: line [0-9]+: syntax error near unexpected token `.*\'"),
    secondary_matcher!(
    r"scons: \*\*\* \[.*\] ValueError : unsupported pickle protocol: .*"),
    // yarn
    secondary_matcher!(
    r"ERROR: There are no scenarios; must have at least one."),
    // perl
    secondary_matcher!(
    r"Execution of (.*) aborted due to compilation errors."),
    // Mocha
    secondary_matcher!(
    r"     AssertionError \[ERR_ASSERTION\]: Missing expected exception."),
    // lt (C++)
    secondary_matcher!(
    r".*: .*:[0-9]+: .*: Assertion `.*\' failed."),
    secondary_matcher!(
    r"(.*).xml: FAILED:"),
    secondary_matcher!(
    r" BROKEN .*"),
    secondary_matcher!(
    r"failed: [0-9]+-.*"),
    // ninja
    secondary_matcher!(
    r"ninja: build stopped: subcommand failed."),
    secondary_matcher!(
    r".*\.s:[0-9]+: Error: .*"),
    // rollup
    secondary_matcher!(r"\[\!\] Error: Unexpected token"),
    // glib
    secondary_matcher!(r"\(.*:[0-9]+\): [a-zA-Z0-9]+-CRITICAL \*\*: [0-9:.]+: .*"),
    secondary_matcher!(
    r"tar: option requires an argument -- \'.\'"),
    secondary_matcher!(
    r"tar: .*: Cannot stat: No such file or directory"),
    secondary_matcher!(
    r"tar: .*: Cannot open: No such file or directory"),
    // rsvg-convert
    secondary_matcher!(
    r"Could not render file (.*.svg)"),
    // pybuild tests
    secondary_matcher!(
    r"ERROR: file not found: (.*)"),
    // msgfmt
    secondary_matcher!(
    r"/usr/bin/msgfmt: found [0-9]+ fatal errors"),
    // Docker
    secondary_matcher!(
    r"Cannot connect to the Docker daemon at unix:///var/run/docker.sock. Is the docker daemon running\?"),
    secondary_matcher!(
    r"dh_makeshlibs: failing due to earlier errors"),
    // Ruby
    secondary_matcher!(
    r"([^:]+)\.rb:[0-9]+:in `([^\'])+\': (.*) \((.*)\)"),
    secondary_matcher!(
    r".*: \*\*\* ERROR: There where errors/warnings in server logs after running test cases."),
    secondary_matcher!(
    r"Errno::EEXIST: File exists @ dir_s_mkdir - .*"),
    secondary_matcher!(
    r"Test environment was found to be incomplete at configuration time,"),
    secondary_matcher!(
    r"libtool:   error: cannot find the library \'(.*)\' or unhandled argument \'(.*)\'"),
    secondary_matcher!(
    r"npm ERR\! (.*)"),
    secondary_matcher!(
    r"install: failed to access \'(.*)\': (.*)"),
    secondary_matcher!(
    r"MSBUILD: error MSBUILD[0-9]+: Project file \'(.*)\' not found."),
    secondary_matcher!(
    r"E: (.*)"),
    secondary_matcher!(
    r"(.*)\(([0-9]+),([0-9]+)\): Error: .*"),
    // C #
    secondary_matcher!(
    r"(.*)\.cs\([0-9]+,[0-9]+\): error CS[0-9]+: .*"),
    secondary_matcher!(
    r".*Segmentation fault.*"),
    secondary_matcher!(
    r"a2x: ERROR: (.*) returned non-zero exit status ([0-9]+)"),
    secondary_matcher!(
    r"-- Configuring incomplete, errors occurred\!"),
    secondary_matcher!(
    r#"Error opening link script "(.*)""#),
    secondary_matcher!(
    r"cc: error: (.*)"),
    secondary_matcher!(
    r"\[ERROR\] .*"),
    secondary_matcher!(
    r"dh_auto_(test|build): error: (.*)"),
    secondary_matcher!(
    r"tar: This does not look like a tar archive"),
    secondary_matcher!(
    r"\[DZ\] no (name|version) was ever set"),
    secondary_matcher!(
    r"\[Runtime\] No -phase or -relationship specified at .* line [0-9]+\."),
    secondary_matcher!(
    r"diff: (.*): No such file or directory"),
    secondary_matcher!(
    r"gpg: signing failed: .*"),
    // mh_install
    secondary_matcher!(
    r"Cannot find the jar to install: (.*)"),
    secondary_matcher!(
    r"ERROR: .*"),
    secondary_matcher!(
    r"> error: (.*)"),
    secondary_matcher!(
    r"error: (.*)"),
    secondary_matcher!(
    r"(.*\.hs):[0-9]+:[0-9]+: error:"),
    secondary_matcher!(
    r"go1: internal compiler error: .*"),
];
}

pub fn find_secondary_build_failure(
    lines: &[&str],
    start_offset: usize,
) -> Option<SingleLineMatch> {
    for (offset, line) in lines.enumerate_tail_forward(start_offset) {
        let match_line = line.trim_end_matches('\n');
        for regexp in SECONDARY_MATCHERS.iter() {
            if regexp.is_match(match_line).unwrap() {
                let origin = Origin(format!("secondary regex {:?}", regexp));
                log::debug!(
                    "Found match against {:?} on {:?} (line {})",
                    regexp,
                    line,
                    offset + 1
                );
                return Some(SingleLineMatch {
                    origin,
                    offset,
                    line: line.to_string(),
                });
            }
        }
    }
    None
}

/// Find the key failure line in build output.
///
/// # Returns
/// A tuple with (match object, error object)
pub fn find_build_failure_description(
    lines: Vec<&str>,
) -> (Option<Box<dyn Match>>, Option<Box<dyn Problem>>) {
    pub const OFFSET: usize = 250;
    // Is this cmake-specific, or rather just kf5 / qmake ?
    let mut cmake = false;
    // We search backwards for clear errors.
    for (lineno, line) in lines.enumerate_backward(Some(250)) {
        if line.contains("cmake") {
            cmake = true;
        }
        if let Some((mm, merr)) = match_lines(lines.as_slice(), lineno).unwrap() {
            return (Some(mm), merr);
        }
    }

    // TODO(jelmer): Remove this in favour of CMakeErrorMatcher above.
    if cmake {
        // Urgh, multi-line regexes---
        for (mut lineno, line) in lines.enumerate_forward(None) {
            let line = line.trim_end_matches('\n');
            if let Some((_, target)) =
                lazy_regex::regex_captures!(r"  Could NOT find (.*) \(missing: .*\)", line)
            {
                return (
                    Some(Box::new(SingleLineMatch::from_lines(
                        &lines,
                        lineno,
                        Some("direct regex"),
                    )) as Box<dyn Match>),
                    Some(Box::new(MissingCommand(target.to_lowercase())) as Box<dyn Problem>),
                );
            }
            if let Some((_, _target)) = lazy_regex::regex_captures!(
                r#"\s*The imported target "(.*)" references the file"#,
                line
            ) {
                lineno += 1;
                while lineno < lines.len() && !line.is_empty() {
                    lineno += 1;
                }
                if lines[lineno + 2].starts_with("  but this file does not exist.") {
                    let filename = if let Some((_, entry)) =
                        lazy_regex::regex_captures!(r#"\s*"(.*)""#, line)
                    {
                        entry
                    } else {
                        line
                    };
                    return (
                        Some(Box::new(SingleLineMatch::from_lines(
                            &lines,
                            lineno,
                            Some("direct regex"),
                        )) as Box<dyn Match>),
                        Some(Box::new(MissingFile {
                            path: filename.into(),
                        }) as Box<dyn Problem>),
                    );
                }
                continue;
            }
            if lineno + 1 < lines.len() {
                if let Some((_, _pkg)) = lazy_regex::regex_captures!("^  Could not find a package configuration file provided by \"(.*)\" with any of the following names:", &(line.to_string() + " " + lines[lineno + 1].trim_start_matches(' ').trim_end_matches('\n'))) {
                    if lines[lineno + 2] == "\n" {
                        let mut i = 3;
                        let mut filenames = vec![];
                        while !lines[lineno + i].trim().is_empty() {
                            filenames.push(lines[lineno + i].trim().to_string());
                            i += 1;
                        }
                        return (
                            Some(Box::new(SingleLineMatch::from_lines(
                                &lines, lineno, Some("direct regex (cmake)")
                            )) as Box<dyn Match>),
                            Some(Box::new(CMakeFilesMissing{filenames, version: None}) as Box<dyn Problem>),
                        )
                    }
                }
            }
        }
    }

    // And forwards for vague ("secondary") errors.
    let m = find_secondary_build_failure(lines.as_slice(), OFFSET);
    if let Some(m) = m {
        return (Some(Box::new(m)), None);
    }

    (None, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_just_match(lines: Vec<&str>, lineno: usize) {
        let (r#match, actual_err) = super::find_build_failure_description(lines.clone());
        assert!(actual_err.is_none());
        if let Some(r#match) = r#match.as_ref() {
            assert_eq!(&r#match.line(), &lines[lineno - 1]);
            assert_eq!(lineno, r#match.lineno());
        } else {
            assert!(r#match.is_none());
        }
    }

    fn assert_match(lines: Vec<&str>, lineno: usize, mut expected: Option<impl Problem + 'static>) {
        let (r#match, actual_err) = super::find_build_failure_description(lines.clone());
        if let Some(r#match) = r#match.as_ref() {
            assert_eq!(&r#match.line(), &lines[lineno - 1]);
            assert_eq!(lineno, r#match.lineno());
        } else {
            assert!(r#match.is_none());
        }
        if let Some(expected) = expected.take() {
            assert!(
                r#match.is_some(),
                "err ({:?}) provided but match missing",
                &expected
            );
            assert_eq!(
                actual_err.as_ref().map(|x| x.as_ref()),
                Some(&expected as &dyn Problem)
            );
        } else {
            assert!(actual_err.is_none());
        }
    }

    #[test]
    fn test_make_missing_rule() {
        assert_match(
            vec![
                "make[1]: *** No rule to make target 'nno.autopgen.bin', needed by 'dan-nno.autopgen.bin'.  Stop."
            ],
            1,
            Some(MissingMakeTarget::new( "nno.autopgen.bin", Some("dan-nno.autopgen.bin"))),
        );

        assert_match(vec![
                "make[1]: *** No rule to make target '/usr/share/blah/blah', needed by 'dan-nno.autopgen.bin'.  Stop."
            ],
            1,
            Some(MissingMakeTarget::new("/usr/share/blah/blah", Some("dan-nno.autopgen.bin"))),
        );
        assert_match(
            vec![
                "debian/rules:4: /usr/share/openstack-pkg-tools/pkgos.make: No such file or directory"
            ],
            1,
            Some(MissingFile::new("/usr/share/openstack-pkg-tools/pkgos.make".into())),
        );
    }

    #[test]
    fn test_git_identity() {
        assert_match(
            vec![
                "fatal: unable to auto-detect email address (got 'jenkins@osuosl167-amd64.(none)')",
            ],
            1,
            Some(MissingGitIdentity),
        );
    }

    #[test]
    fn test_ioerror() {
        assert_match(
            vec![
                "E   IOError: [Errno 2] No such file or directory: '/usr/lib/python2.7/poly1305/rfc7539.txt'"
            ],
            1,
            Some(MissingFile::new("/usr/lib/python2.7/poly1305/rfc7539.txt".into())),
        );
    }

    #[test]
    fn test_vignette() {
        assert_match(
            vec![
                "Error: processing vignette 'uroot-intro.Rnw' failed with diagnostics:",
                "pdflatex is not available",
            ],
            2,
            Some(MissingVagueDependency::simple("pdflatex")),
        );
    }

    #[test]
    fn test_upstart_file_present() {
        assert_match(
            vec![
                "dh_installinit: upstart jobs are no longer supported!  Please remove debian/sddm.upstart and check if you need to add a conffile removal"
            ],
            1,
            Some(UpstartFilePresent("debian/sddm.upstart".into())),
        );
    }

    #[test]
    fn test_missing_go_mod_file() {
        assert_match(
            vec![
                "go: go.mod file not found in current directory or any parent directory; see 'go help modules'"
            ],
            1,
            Some(MissingGoModFile),
        );
    }

    #[test]
    fn test_missing_javascript_runtime() {
        assert_match(
            vec![
                "ExecJS::RuntimeUnavailable: Could not find a JavaScript runtime. See https://github.com/rails/execjs for a list of available runtimes."],
            1,
            Some(MissingJavaScriptRuntime)
        );
    }

    #[test]
    fn test_directory_missing() {
        assert_match(
            vec!["debian/components/build: 19: cd: can't cd to rollup-plugin"],
            1,
            Some(DirectoryNonExistant("rollup-plugin".to_owned())),
        );
    }

    #[test]
    fn test_vcs_control_directory() {
        assert_match(
            vec!["   > Cannot find '.git' directory"],
            1,
            Some(VcsControlDirectoryNeeded::new(vec!["git"])),
        );
    }

    #[test]
    fn test_missing_sprockets_file() {
        assert_match(
            vec![
                "Sprockets::FileNotFound: couldn't find file 'activestorage' with type 'application/javascript'"
            ],
            1,
            Some(MissingSprocketsFile { name: "activestorage".to_owned(), content_type: "application/javascript".to_owned()}),
        );
    }

    #[test]
    fn test_gxx_missing_file() {
        assert_match(
            vec!["g++: error: /usr/lib/x86_64-linux-gnu/libGL.so: No such file or directory"],
            1,
            Some(MissingFile::new(
                "/usr/lib/x86_64-linux-gnu/libGL.so".into(),
            )),
        );
    }

    #[test]
    fn test_build_xml_missing_file() {
        assert_match(
            vec!["/<<PKGBUILDDIR>>/build.xml:59: /<<PKGBUILDDIR>>/lib does not exist."],
            1,
            Some(MissingBuildFile {
                filename: "lib".to_owned(),
            }),
        );
    }

    #[test]
    fn test_vignette_builder() {
        assert_match(
            vec!["  vignette builder 'R.rsp' not found"],
            1,
            Some(MissingRPackage::simple("R.rsp")),
        );
    }

    #[test]
    fn test_dh_missing_addon() {
        assert_match(
            vec![
                "   dh_auto_clean -O--buildsystem=pybuild",
                "E: Please add appropriate interpreter package to Build-Depends, see pybuild(1) for details.this: $VAR1 = bless( {",
                "     'py3vers' => '3.8',",
                "     'py3def' => '3.8',",
                "     'pyvers' => '',",
                "     'parallel' => '2',",
                "     'cwd' => '/<<PKGBUILDDIR>>',",
                "     'sourcedir' => '.',",
                "     'builddir' => undef,",
                "     'pypydef' => '',",
                "     'pydef' => ''",
                "   }, 'Debian::Debhelper::Buildsystem::pybuild' );",
                "deps: $VAR1 = [];",
            ],
            2,
            Some(DhAddonLoadFailure{ name: "pybuild".to_owned(), path: "Debian/Debhelper/Buildsystem/pybuild.pm".to_owned()}),
        );
    }

    #[test]
    fn test_libtoolize_missing_file() {
        assert_match(
            vec!["libtoolize:   error: '/usr/share/aclocal/ltdl.m4' does not exist."],
            1,
            Some(MissingFile::new("/usr/share/aclocal/ltdl.m4".into())),
        );
    }

    #[test]
    fn test_ruby_missing_file() {
        assert_match(
            vec![
                "Error: Error: ENOENT: no such file or directory, open '/usr/lib/nodejs/requirejs/text.js'"
            ],
            1,
            Some(MissingFile::new("/usr/lib/nodejs/requirejs/text.js".into())),
        );
    }

    #[test]
    fn test_vcversioner() {
        assert_match(
            vec![
                "vcversioner: ['git', '--git-dir', '/build/tmp0tlam4pe/pyee/.git', 'describe', '--tags', '--long'] failed and '/build/tmp0tlam4pe/pyee/version.txt' isn't present."
            ],
            1,
            Some(MissingVcVersionerVersion),
        );
    }

    #[test]
    fn test_python_missing_file() {
        assert_match(
            vec![
                "python3.7: can't open file '/usr/bin/blah.py': [Errno 2] No such file or directory"
            ],
            1,
            Some(MissingFile::new("/usr/bin/blah.py".into())),
        );
        assert_match(
            vec!["python3.7: can't open file 'setup.py': [Errno 2] No such file or directory"],
            1,
            Some(MissingBuildFile::new("setup.py".into())),
        );
        assert_match(
            vec![
                "E           FileNotFoundError: [Errno 2] No such file or directory: '/usr/share/firmware-microbit-micropython/firmware.hex'"
            ],
            1,
            Some(MissingFile::new(
                "/usr/share/firmware-microbit-micropython/firmware.hex".into()
            )),
        );
    }

    #[test]
    fn test_vague() {
        assert_match(
            vec![
                "configure: error: Please install gnu flex from http://www.gnu.org/software/flex/",
            ],
            1,
            Some(MissingVagueDependency {
                name: "gnu flex".to_string(),
                url: Some("http://www.gnu.org/software/flex/".to_owned()),
                minimum_version: None,
                current_version: None,
            }),
        );
        assert_match(
            vec!["RuntimeError: cython is missing"],
            1,
            Some(MissingVagueDependency::simple("cython")),
        );
        assert_match(
            vec![
                "configure: error:",
                "",
                "        Unable to find the Multi Emulator Super System (MESS).",
            ],
            3,
            Some(MissingVagueDependency::simple(
                "the Multi Emulator Super System (MESS)",
            )),
        );
        assert_match(
            vec![
                "configure: error: libwandio 4.0.0 or better is required to compile this version of libtrace. If you have installed libwandio in a non-standard location please use LDFLAGS to specify the location of the library. WANDIO can be obtained from http://research.wand.net.nz/software/libwandio.php"
            ],
            1,
            Some(MissingVagueDependency{
                name: "libwandio".to_owned(),
                minimum_version: Some("4.0.0".to_owned()),
                current_version: None,
                url: None,
            }),
        );
        assert_match(
            vec![
                "configure: error: libpcap0.8 or greater is required to compile libtrace. If you have installed it in a non-standard location please use LDFLAGS to specify the location of the library"
            ],
            1,
            Some(MissingVagueDependency::simple("libpcap0.8")),
        );
        assert_match(
            vec!["Error: Please install xml2 package"],
            1,
            Some(MissingVagueDependency::simple("xml2")),
        );
    }

    #[test]
    fn test_gettext_mismatch() {
        assert_match(
            vec![
                "*** error: gettext infrastructure mismatch: using a Makefile.in.in from gettext version 0.19 but the autoconf macros are from gettext version 0.20"
            ],
            1,
            Some(MismatchGettextVersions{makefile_version: "0.19".to_string(), autoconf_version: "0.20".to_string()}),
        );
    }

    #[test]
    fn test_x11_missing() {
        assert_match(
            vec![
                "configure: error: *** No X11! Install X-Windows development headers/libraries! ***"
            ],
            1,
            Some(MissingX11),
        );
    }

    #[test]
    fn test_multi_line_configure_error() {
        assert_just_match(
            vec!["configure: error:", "", "        Some other error."],
            3,
        );
        assert_match(
            vec![
                "configure: error:",
                "",
                "   Unable to find the Multi Emulator Super System (MESS).",
                "",
                "   Please install MESS, or specify the MESS command with",
                "   a MESS environment variable.",
                "",
                "e.g. MESS=/path/to/program/mess ./configure",
            ],
            3,
            Some(MissingVagueDependency::simple(
                "the Multi Emulator Super System (MESS)",
            )),
        );
    }

    #[test]
    fn test_interpreter_missing() {
        assert_match(
            vec![
                "/bin/bash: /usr/bin/rst2man: /usr/bin/python: bad interpreter: No such file or directory"
            ],
            1,
            Some(MissingFile::new("/usr/bin/python".into()))
        );
        assert_just_match(
            vec!["env: ‘/<<PKGBUILDDIR>>/socket-activate’: No such file or directory"],
            1,
        );
    }

    #[test]
    fn test_webpack_missing() {
        assert_just_match(
            vec![
                "ERROR in Entry module not found: Error: Can't resolve 'index.js' in '/<<PKGBUILDDIR>>'"
            ],
            1,
        );
    }

    #[test]
    fn test_installdocs_missing() {
        assert_match(
            vec![
                r#"dh_installdocs: Cannot find (any matches for) "README.txt" (tried in ., debian/tmp)"#,
            ],
            1,
            Some(DebhelperPatternNotFound {
                pattern: "README.txt".to_owned(),
                tool: "installdocs".to_owned(),
                directories: vec![".".to_string(), "debian/tmp".to_owned()],
            }),
        );
    }

    #[test]
    fn test_dh_compat_dupe() {
        assert_match(
            vec![
                "dh_autoreconf: debhelper compat level specified both in debian/compat and via build-dependency on debhelper-compat"
            ],
            1,
            Some(DuplicateDHCompatLevel{command: "dh_autoreconf".to_owned()}),
        );
    }

    #[test]
    fn test_dh_compat_missing() {
        assert_match(
            vec!["dh_clean: Please specify the compatibility level in debian/compat"],
            1,
            Some(MissingDHCompatLevel {
                command: "dh_clean".to_owned(),
            }),
        );
    }

    #[test]
    fn test_dh_compat_too_old() {
        assert_match(
            vec! [
                "dh_clean: error: Compatibility levels before 7 are no longer supported (level 5 requested)"
            ],
            1,
            Some(UnsupportedDebhelperCompatLevel{ oldest_supported: 7, requested: 5})
        );
    }

    #[test]
    fn test_dh_udeb_shared_library() {
        assert_just_match(vec![
                "dh_makeshlibs: The udeb libepoxy0-udeb (>= 1.3) does not contain any shared libraries but --add-udeb=libepoxy0-udeb (>= 1.3) was passed!?"
            ],
            1,
        );
    }

    #[test]
    fn test_dh_systemd() {
        assert_just_match(
            vec![
                "dh: unable to load addon systemd: dh: The systemd-sequence is no longer provided in compat >= 11, please rely on dh_installsystemd instead"
            ],
            1,
        );
    }

    #[test]
    fn test_dh_before() {
        assert_just_match(vec![
                "dh: The --before option is not supported any longer (#932537). Use override targets instead."
            ],
            1,
        );
    }

    #[test]
    fn test_meson_missing_git() {
        assert_match(
            vec!["meson.build:13:0: ERROR: Git program not found."],
            1,
            Some(MissingCommand("git".to_owned())),
        );
    }

    #[test]
    fn test_meson_missing_lib() {
        assert_match(
            vec!["meson.build:85:0: ERROR: C++ shared or static library 'vulkan-1' not found"],
            1,
            Some(MissingLibrary("vulkan-1".to_owned())),
        );
    }

    #[test]
    fn test_ocaml_library_missing() {
        assert_match(
            vec![r#"Error: Library "camlp-streams" not found."#],
            1,
            Some(MissingOCamlPackage("camlp-streams".to_owned())),
        );
    }

    #[test]
    fn test_meson_version() {
        assert_match(
            vec!["meson.build:1:0: ERROR: Meson version is 0.49.2 but project requires >=0.50"],
            1,
            Some(MissingVagueDependency {
                name: "meson".to_owned(),
                minimum_version: Some("0.50".to_owned()),
                current_version: Some("0.49.2".to_owned()),
                url: None,
            }),
        );
        assert_match(
            vec!["../meson.build:1:0: ERROR: Meson version is 0.49.2 but project requires >=0.50"],
            1,
            Some(MissingVagueDependency {
                name: "meson".to_string(),
                minimum_version: Some("0.50".to_owned()),
                current_version: Some("0.49.2".to_owned()),
                url: None,
            }),
        );
    }

    #[test]
    fn test_need_pgbuildext() {
        assert_match(
            vec![
                "Error: debian/control needs updating from debian/control.in. Run 'pg_buildext updatecontrol'."
            ],
            1,
            Some(NeedPgBuildExtUpdateControl{generated_path: "debian/control".to_owned(), template_path: "debian/control.in".to_owned()})
        );
    }

    #[test]
    fn test_cmake_missing_command() {
        assert_match(
            vec![
                "  Could NOT find Git (missing: GIT_EXECUTABLE)",
                "dh_auto_configure: cd obj-x86_64-linux-gnu && cmake with args",
            ],
            1,
            Some(MissingCommand("git".to_owned())),
        );
    }

    #[test]
    fn test_autoconf_version() {
        assert_match(
            vec!["configure.ac:13: error: Autoconf version 2.71 or higher is required"],
            1,
            Some(MissingVagueDependency {
                name: "autoconf".to_string(),
                minimum_version: Some("2.71".to_string()),
                current_version: None,
                url: None,
            }),
        );
    }

    #[test]
    fn test_claws_version() {
        assert_match(
            vec!["configure: error: libetpan 0.57 not found"],
            1,
            Some(MissingVagueDependency {
                name: "libetpan".to_string(),
                minimum_version: Some("0.57".to_string()),
                current_version: None,
                url: None,
            }),
        );
    }

    #[test]
    fn test_config_status_input() {
        assert_match(
            vec!["config.status: error: cannot find input file: `po/Makefile.in.in'"],
            1,
            Some(MissingConfigStatusInput {
                path: "po/Makefile.in.in".to_owned(),
            }),
        );
    }

    #[test]
    fn test_jvm() {
        assert_match(
            vec!["ERROR: JAVA_HOME is set to an invalid directory: /usr/lib/jvm/default-java/"],
            1,
            Some(MissingJVM),
        );
    }

    #[test]
    fn test_cp() {
        assert_match(
            vec![
                "cp: cannot stat '/<<PKGBUILDDIR>>/debian/patches/lshw-gtk.desktop': No such file or directory"
            ],
            1,
            Some(MissingBuildFile::new("debian/patches/lshw-gtk.desktop".to_owned()))
        );
    }

    #[test]
    fn test_bash_redir_missing() {
        assert_match(
            vec!["/bin/bash: idna-tables-properties.csv: No such file or directory"],
            1,
            Some(MissingBuildFile::new(
                "idna-tables-properties.csv".to_owned(),
            )),
        );
    }

    #[test]
    fn test_automake_input() {
        assert_match(
            vec!["automake: error: cannot open < gtk-doc.make: No such file or directory"],
            1,
            Some(MissingAutomakeInput {
                path: "gtk-doc.make".to_owned(),
            }),
        );
    }

    #[test]
    fn test_shellcheck() {
        assert_just_match(
            vec![
                &(" ".repeat(40)
                    + "^----^ SC2086: Double quote to prevent globbing and word splitting."),
            ],
            1,
        );
    }

    #[test]
    fn test_autoconf_macro() {
        assert_match(
            vec!["configure.in:1802: error: possibly undefined macro: AC_CHECK_CCA"],
            1,
            Some(MissingAutoconfMacro {
                r#macro: "AC_CHECK_CCA".to_owned(),
                need_rebuild: false,
            }),
        );
        assert_match(
            vec!["./configure: line 12569: PKG_PROG_PKG_CONFIG: command not found"],
            1,
            Some(MissingAutoconfMacro {
                r#macro: "PKG_PROG_PKG_CONFIG".to_owned(),
                need_rebuild: false,
            }),
        );
        assert_match(
            vec![
                "checking for gawk... (cached) mawk",
                "./configure: line 2368: syntax error near unexpected token `APERTIUM,'",
                "./configure: line 2368: `PKG_CHECK_MODULES(APERTIUM, apertium >= 3.7.1)'",
            ],
            3,
            Some(MissingAutoconfMacro {
                r#macro: "PKG_CHECK_MODULES".to_owned(),
                need_rebuild: true,
            }),
        );
        assert_match(
            vec![
                "checking for libexif to use... ./configure: line 15968: syntax error near unexpected token `LIBEXIF,libexif'",
                "./configure: line 15968: `\t\t\t\t\t\tPKG_CHECK_MODULES(LIBEXIF,libexif >= 0.6.18,have_LIBEXIF=yes,:)'",
            ],
            2,
            Some(MissingAutoconfMacro{
                r#macro: "PKG_CHECK_MODULES".to_owned(), need_rebuild:true})
        );
    }

    #[test]
    fn test_r_missing() {
        assert_match(
            vec![
                "ERROR: dependencies ‘ellipsis’, ‘pkgload’ are not available for package ‘testthat’"
            ],
            1,
            Some(MissingRPackage::simple("ellipsis")),
        );
        assert_match(
            vec!["  namespace ‘DBI’ 1.0.0 is being loaded, but >= 1.0.0.9003 is required"],
            1,
            Some(MissingRPackage {
                package: "DBI".to_owned(),
                minimum_version: Some("1.0.0.9003".to_owned()),
            }),
        );
        assert_match(
            vec![
                "  namespace ‘spatstat.utils’ 1.13-0 is already loaded, but >= 1.15.0 is required",
            ],
            1,
            Some(MissingRPackage {
                package: "spatstat.utils".to_owned(),
                minimum_version: Some("1.15.0".to_owned()),
            }),
        );
        assert_match(
            vec!["Error in library(zeligverse) : there is no package called 'zeligverse'"],
            1,
            Some(MissingRPackage::simple("zeligverse")),
        );
        assert_match(
            vec!["there is no package called 'mockr'"],
            1,
            Some(MissingRPackage::simple("mockr")),
        );
        assert_match(
            vec![
                "ERROR: dependencies 'igraph', 'matlab', 'expm', 'RcppParallel' are not available for package 'markovchain'"
            ],
            1,
            Some(MissingRPackage::simple("igraph"))
        );
        assert_match(
            vec![
                "Error: package 'BH' 1.66.0-1 was found, but >= 1.75.0.0 is required by 'RSQLite'",
            ],
            1,
            Some(MissingRPackage {
                package: "BH".to_owned(),
                minimum_version: Some("1.75.0.0".to_owned()),
            }),
        );
        assert_match(
         vec![
                "Error: package ‘AnnotationDbi’ 1.52.0 was found, but >= 1.53.1 is required by ‘GO.db’"
            ],
            1,
            Some(MissingRPackage{ package: "AnnotationDbi".to_owned(), minimum_version: Some("1.53.1".to_owned())})
        );
        assert_match(
            vec!["  namespace 'alakazam' 1.1.0 is being loaded, but >= 1.1.0.999 is required"],
            1,
            Some(MissingRPackage {
                package: "alakazam".to_string(),
                minimum_version: Some("1.1.0.999".to_string()),
            }),
        );
    }

    #[test]
    fn test_mv_stat() {
        assert_match(
            vec!["mv: cannot stat '/usr/res/boss.png': No such file or directory"],
            1,
            Some(MissingFile::new("/usr/res/boss.png".into())),
        );
        assert_just_match(
            vec!["mv: cannot stat 'res/boss.png': No such file or directory"],
            1,
        );
    }

    #[test]
    fn test_dh_link_error() {
        assert_match(
            vec![
                "dh_link: link destination debian/r-cran-crosstalk/usr/lib/R/site-library/crosstalk/lib/ionrangeslider is a directory"
            ],
            1,
            Some(DhLinkDestinationIsDirectory(
                "debian/r-cran-crosstalk/usr/lib/R/site-library/crosstalk/lib/ionrangeslider".to_owned()
            )),
        );
    }

    #[test]
    fn test_go_test() {
        assert_just_match(vec!["FAIL\tgithub.com/edsrzf/mmap-go\t0.083s"], 1);
    }

    #[test]
    fn test_debhelper_pattern() {
        assert_match(
            vec![
                r#"dh_install: Cannot find (any matches for) "server/etc/gnumed/gnumed-restore.conf" (tried in ., debian/tmp)"#,
            ],
            1,
            Some(DebhelperPatternNotFound {
                pattern: "server/etc/gnumed/gnumed-restore.conf".to_owned(),
                tool: "install".to_owned(),
                directories: vec![".".to_string(), "debian/tmp".to_string()],
            }),
        );
    }

    #[test]
    fn test_symbols() {
        assert_match(
            vec![
                "dpkg-gensymbols: error: some symbols or patterns disappeared in the symbols file: see diff output below"
            ],
            1,
            Some(DisappearedSymbols)
        );
    }

    #[test]
    fn test_missing_php_class() {
        assert_match(
            vec![
                "PHP Fatal error:  Uncaught Error: Class 'PHPUnit_Framework_TestCase' not found in /tmp/autopkgtest.gO7h1t/build.b1p/src/Horde_Text_Diff-2.2.0/test/Horde/Text/Diff/EngineTest.php:9"
            ],
            1,
            Some(MissingPhpClass{php_class: "PHPUnit_Framework_TestCase".to_owned()})
        );
    }

    #[test]
    fn test_missing_java_class() {
        assert_match(
            r#"Caused by: java.lang.ClassNotFoundException: org.codehaus.Xpp3r$Builder
\tat org.codehaus.strategy.SelfFirstStrategy.loadClass(lfFirstStrategy.java:50)
\tat org.codehaus.realm.ClassRealm.unsynchronizedLoadClass(ClassRealm.java:271)
\tat org.codehaus.realm.ClassRealm.loadClass(ClassRealm.java:247)
\tat org.codehaus.realm.ClassRealm.loadClass(ClassRealm.java:239)
\t... 46 more
"#
            .split("\n")
            .collect::<Vec<&str>>(),
            1,
            Some(MissingJavaClass {
                classname: "org.codehaus.Xpp3r$Builder".to_owned(),
            }),
        );
    }

    #[test]
    fn test_install_docs_link() {
        assert_just_match(
            r#"dh_installdocs: --link-doc not allowed between sympow and sympow-data (one is \
arch:all and the other not)"#
                .split("\n")
                .collect::<Vec<&str>>(),
            1,
        );
    }

    #[test]
    fn test_dh_until_unsupported() {
        assert_match(
            vec![
                "dh: The --until option is not supported any longer (#932537). Use override targets instead."
            ],
            1,
            Some(DhUntilUnsupported)
        );
    }

    #[test]
    fn test_missing_xml_entity() {
        assert_match(
            vec![
                "I/O error : Attempt to load network entity http://www.oasis-open.org/docbook/xml/4.5/docbookx.dtd"
            ],
            1,
            Some(MissingXmlEntity{url: "http://www.oasis-open.org/docbook/xml/4.5/docbookx.dtd".to_owned()})
        );
    }

    #[test]
    fn test_ccache_error() {
        assert_match(
            vec![
                "ccache: error: Failed to create directory /sbuild-nonexistent/.ccache/tmp: Permission denied"
            ],
            1,
            Some(CcacheError(
                "Failed to create directory /sbuild-nonexistent/.ccache/tmp: Permission denied".to_owned()
            ))
        );
    }

    #[test]
    fn test_dh_addon_load_failure() {
        assert_match(
            vec![
                "dh: unable to load addon nodejs: Debian/Debhelper/Sequence/nodejs.pm did not return a true value at (eval 11) line 1."
            ],
            1,
            Some(DhAddonLoadFailure{name: "nodejs".to_owned(), path: "Debian/Debhelper/Sequence/nodejs.pm".to_owned()})
        );
    }

    #[test]
    fn test_missing_library() {
        assert_match(
            vec!["/usr/bin/ld: cannot find -lpthreads"],
            1,
            Some(MissingLibrary("pthreads".to_owned())),
        );
        assert_just_match(
            vec!["./testFortranCompiler.f:4: undefined reference to `sgemm_'"],
            1,
        );
        assert_just_match(
            vec!["writer.d:59: error: undefined reference to 'sam_hdr_parse_'"],
            1,
        );
    }

    #[test]
    fn test_assembler() {
        assert_match(vec!["Found no assembler"], 1, Some(MissingAssembler))
    }

    #[test]
    fn test_command_missing() {
        assert_match(
            vec!["./ylwrap: line 176: yacc: command not found"],
            1,
            Some(MissingCommand("yacc".to_owned())),
        );
        assert_match(
            vec!["/bin/sh: 1: cmake: not found"],
            1,
            Some(MissingCommand("cmake".to_owned())),
        );
        assert_match(
            vec!["sh: 1: git: not found"],
            1,
            Some(MissingCommand("git".to_owned())),
        );
        assert_match(
            vec!["/usr/bin/env: ‘python3’: No such file or directory"],
            1,
            Some(MissingCommand("python3".to_owned())),
        );
        assert_match(
            vec!["%Error: 'flex' must be installed to build"],
            1,
            Some(MissingCommand("flex".to_owned())),
        );
        assert_match(
            vec![r#"pkg-config: exec: "pkg-config": executable file not found in $PATH"#],
            1,
            Some(MissingCommand("pkg-config".to_owned())),
        );
        assert_match(
            vec![r#"Can't exec "git": No such file or directory at Makefile.PL line 25."#],
            1,
            Some(MissingCommand("git".to_owned())),
        );
        assert_match(
            vec![
                "vcver.scm.git.GitCommandError: 'git describe --tags --match 'v*' --abbrev=0' returned an error code 127"
            ],
            1,
            Some(MissingCommand("git".to_owned())),
        );
        assert_match(
            vec!["make[1]: docker: Command not found"],
            1,
            Some(MissingCommand("docker".to_owned())),
        );
        assert_match(
            vec!["make[1]: git: Command not found"],
            1,
            Some(MissingCommand("git".to_owned())),
        );
        assert_just_match(vec!["make[1]: ./docker: Command not found"], 1);
        assert_match(
            vec!["make: dh_elpa: Command not found"],
            1,
            Some(MissingCommand("dh_elpa".to_owned())),
        );
        assert_match(
            vec!["/bin/bash: valac: command not found"],
            1,
            Some(MissingCommand("valac".to_owned())),
        );
        assert_match(
            vec!["E: Failed to execute “python3”: No such file or directory"],
            1,
            Some(MissingCommand("python3".to_owned())),
        );
        assert_match(
            vec![
                r#"Can't exec "cmake": No such file or directory at /usr/share/perl5/Debian/Debhelper/Dh_Lib.pm line 484."#,
            ],
            1,
            Some(MissingCommand("cmake".to_owned())),
        );
        assert_match(
            vec!["Invalid gemspec in [unicorn.gemspec]: No such file or directory - git"],
            1,
            Some(MissingCommand("git".to_owned())),
        );
        assert_match(
            vec!["dbus-run-session: failed to exec 'xvfb-run': No such file or directory"],
            1,
            Some(MissingCommand("xvfb-run".to_owned())),
        );
        assert_match(
            vec!["/bin/sh: 1: ./configure: not found"],
            1,
            Some(MissingConfigure),
        );
        assert_match(
            vec!["xvfb-run: error: xauth command not found"],
            1,
            Some(MissingCommand("xauth".to_owned())),
        );
        assert_match(
            vec!["meson.build:39:2: ERROR: Program(s) ['wrc'] not found or not executable"],
            1,
            Some(MissingCommand("wrc".to_owned())),
        );
        assert_match(
            vec![
                "/tmp/autopkgtest.FnbV06/build.18W/src/debian/tests/blas-testsuite: 7: dpkg-architecture: not found"
            ],
            1,
            Some(MissingCommand("dpkg-architecture".to_owned())),
        );
        assert_match(
            vec![
                "Traceback (most recent call last):",
                r#"  File "/usr/lib/python3/dist-packages/mesonbuild/mesonmain.py", line 140, in run"#,
                "    return options.run_func(options)",
                r#"  File "/usr/lib/python3/dist-packages/mesonbuild/mdist.py", line 267, in run"#,
                "    names = create_dist_git(dist_name, archives, src_root, bld_root, dist_sub, b.dist_scripts, subprojects)",
                r#"  File "/usr/lib/python3/dist-packages/mesonbuild/mdist.py", line 119, in create_dist_git"#,
                "    git_clone(src_root, distdir)",
                r#"  File "/usr/lib/python3/dist-packages/mesonbuild/mdist.py", line 108, in git_clone"#,
                "    if git_have_dirty_index(src_root):",
                r#"  File "/usr/lib/python3/dist-packages/mesonbuild/mdist.py", line 104, in git_have_dirty_index"#,
                "    ret = subprocess.call(['git', '-C', src_root, 'diff-index', '--quiet', 'HEAD'])",
                r#"  File "/usr/lib/python3.9/subprocess.py", line 349, in call"#,
                "    with Popen(*popenargs, **kwargs) as p:",
                r#"  File "/usr/lib/python3.9/subprocess.py", line 951, in __init__"#,
                "    self._execute_child(args, executable, preexec_fn, close_fds,",
                r#"  File "/usr/lib/python3.9/subprocess.py", line 1823, in _execute_child"#,
                "    raise child_exception_type(errno_num, err_msg, err_filename)",
                "FileNotFoundError: [Errno 2] No such file or directory: 'git'",
            ],
            18,
            Some(MissingCommand("git".to_owned())),
        );
        assert_match(
            vec![r#"> Cannot run program "git": error=2, No such file or directory"#],
            1,
            Some(MissingCommand("git".to_owned())),
        );
        assert_match(
            vec!["E ImportError: Bad git executable"],
            1,
            Some(MissingCommand("git".to_owned())),
        );
        assert_match(
            vec!["E ImportError: Bad git executable."],
            1,
            Some(MissingCommand("git".to_owned())),
        );
        assert_match(
            vec![r#"Could not find external command "java""#],
            1,
            Some(MissingCommand("java".to_owned())),
        );
    }

    #[test]
    fn test_ts_error() {
        assert_just_match(
            vec!["blah/tokenizer.ts(175,21): error TS2532: Object is possibly 'undefined'."],
            1,
        );
    }

    #[test]
    fn test_pkg_config_missing() {
        assert_match(
            vec!["configure: error: Package requirements (apertium-3.2 >= 3.2.0) were not met:"],
            1,
            Some(MissingPkgConfig::new(
                "apertium-3.2".to_owned(),
                Some("3.2.0".to_owned()),
            )),
        );
        assert_match(
            vec![
                "checking for GLEW... configure: error: Package requirements (glew) were not met:",
            ],
            1,
            Some(MissingPkgConfig::simple("glew".to_owned())),
        );
        assert_match(
            vec!["meson.build:10:0: ERROR: Dependency \"gssdp-1.2\" not found, tried pkgconfig"],
            1,
            Some(MissingPkgConfig::simple("gssdp-1.2".to_owned())),
        );
        assert_match(
            vec![
                "src/plugins/sysprof/meson.build:3:0: ERROR: Dependency \"sysprof-3\" not found, tried pkgconfig"
            ],
            1,
            Some(MissingPkgConfig::simple("sysprof-3".to_owned())),
        );
        assert_match(
            vec![
                "meson.build:84:0: ERROR: Invalid version of dependency, need 'libpeas-1.0' ['>= 1.24.0'] found '1.22.0'."
            ],
            1,
            Some(MissingPkgConfig::new("libpeas-1.0".to_owned(), Some("1.24.0".to_owned()))),
        );
        assert_match(
            vec![
                "meson.build:233:0: ERROR: Invalid version of dependency, need 'vte-2.91' ['>=0.63.0'] found '0.62.3'."
            ],
            1,
            Some(MissingPkgConfig::new("vte-2.91".to_owned(), Some("0.63.0".to_owned()))),
        );

        assert_match(
            vec!["No package 'tepl-3' found"],
            1,
            Some(MissingPkgConfig::simple("tepl-3".to_owned())),
        );
        assert_match(
            vec!["Requested 'vte-2.91 >= 0.59.0' but version of vte is 0.58.2"],
            1,
            Some(MissingPkgConfig::new(
                "vte-2.91".to_owned(),
                Some("0.59.0".to_owned()),
            )),
        );
        assert_match(
            vec!["configure: error: x86_64-linux-gnu-pkg-config sdl2 couldn't be found"],
            1,
            Some(MissingPkgConfig::simple("sdl2".to_owned())),
        );
        assert_match(
            vec!["configure: error: No package 'libcrypto' found"],
            1,
            Some(MissingPkgConfig::simple("libcrypto".to_owned())),
        );
        assert_match(
            vec![
                "-- Checking for module 'gtk+-3.0'",
                "--   Package 'gtk+-3.0', required by 'virtual:world', not found",
            ],
            2,
            Some(MissingPkgConfig::simple("gtk+-3.0".to_owned())),
        );
        assert_match(
            vec![
                "configure: error: libfilezilla not found: Package dependency requirement 'libfilezilla >= 0.17.1' could not be satisfied."
            ],
            1,
            Some(MissingPkgConfig::new("libfilezilla".to_owned(), Some("0.17.1".to_owned()))),
        );
    }

    #[test]
    fn test_pkgconf() {
        assert_match(
            vec!["checking for LAPACK... configure: error: \"Cannot check for existence of module lapack without pkgconf\""],
            1,
            Some(MissingCommand("pkgconf".to_owned())),
        );
    }

    #[test]
    fn test_dh_with_order() {
        assert_match(
            vec!["dh: Unknown sequence --with (options should not come before the sequence)"],
            1,
            Some(DhWithOrderIncorrect),
        );
    }

    #[test]
    fn test_fpic() {
        assert_just_match(
            vec![
                "/usr/bin/ld: pcap-linux.o: relocation R_X86_64_PC32 against symbol `stderr@@GLIBC_2.2.5' can not be used when making a shared object; recompile with -fPIC"
            ],
            1,
        );
    }

    #[test]
    fn test_rspec() {
        assert_just_match(
            vec![
                "rspec ./spec/acceptance/cookbook_resource_spec.rb:20 # Client API operations downloading a cookbook when the cookbook of the name/version is found downloads the cookbook to the destination"
            ],
            1,
        );
    }

    #[test]
    fn test_multiple_definition() {
        assert_just_match(
            vec![
                "./dconf-paths.c:249: multiple definition of `dconf_is_rel_dir'; client/libdconf-client.a(dconf-paths.c.o):./obj-x86_64-linux-gnu/../common/dconf-paths.c:249: first defined here"
            ],
            1,
        );
        assert_just_match(
            vec![
                "/usr/bin/ld: ../lib/libaxe.a(stream.c.o):(.bss+0x10): multiple definition of `gsl_message_mask'; ../lib/libaxe.a(error.c.o):(.bss+0x8): first defined here"
            ],
            1,
        );
    }

    #[test]
    fn test_missing_ruby_gem() {
        assert_match(
            vec![
                "Could not find gem 'childprocess (~> 0.5)', which is required by gem 'selenium-webdriver', in any of the sources."
            ],
            1,
            Some(MissingRubyGem::new("childprocess".to_owned(), Some("0.5".to_owned()))),
        );
        assert_match(
            vec![
                "Could not find gem 'rexml', which is required by gem 'rubocop', in any of the sources."
            ],
            1,
            Some(MissingRubyGem::simple("rexml".to_owned())),
        );
        assert_match(
            vec![
                "/usr/lib/ruby/2.5.0/rubygems/dependency.rb:310:in `to_specs': Could not find 'http-parser' (~> 1.2.0) among 59 total gem(s) (Gem::MissingSpecError)"
            ],
            1,
            Some(MissingRubyGem::new("http-parser".to_owned(), Some("1.2.0".to_string()))),
        );
        assert_match(
            vec![
                "/usr/lib/ruby/2.5.0/rubygems/dependency.rb:312:in `to_specs': Could not find 'celluloid' (~> 0.17.3) - did find: [celluloid-0.16.0] (Gem::MissingSpecVersionError)"
            ],
            1,
            Some(MissingRubyGem{gem:"celluloid".to_owned(), version:Some("0.17.3".to_owned())}),
        );
        assert_match(
            vec![
                "/usr/lib/ruby/2.5.0/rubygems/dependency.rb:312:in `to_specs': Could not find 'i18n' (~> 0.7) - did find: [i18n-1.5.3] (Gem::MissingSpecVersionError)"
            ],
            1,
            Some(MissingRubyGem{gem:"i18n".to_owned(), version: Some("0.7".to_owned())}),
        );
        assert_match(
            vec![
                "/usr/lib/ruby/2.5.0/rubygems/dependency.rb:310:in `to_specs': Could not find 'sassc' (>= 2.0.0) among 34 total gem(s) (Gem::MissingSpecError)"
            ],
            1,
            Some(MissingRubyGem{gem:"sassc".to_string(), version: Some("2.0.0".to_string())}),
        );
        assert_match(
            vec![
                "/usr/lib/ruby/2.7.0/bundler/resolver.rb:290:in `block in verify_gemfile_dependencies_are_found!': Could not find gem 'rake-compiler' in any of the gem sources listed in your Gemfile. (Bundler::GemNotFound)"
            ],
            1,
            Some(MissingRubyGem::simple("rake-compiler".to_owned())),
        );
        assert_match(
            vec![
                "/usr/lib/ruby/2.7.0/rubygems.rb:275:in `find_spec_for_exe': can't find gem rdoc (>= 0.a) with executable rdoc (Gem::GemNotFoundException)"
            ],
            1,
            Some(MissingRubyGem::new("rdoc".to_owned(), Some("0.a".to_owned()))),
        );
    }

    #[test]
    fn test_missing_maven_artifacts() {
        assert_match(
            vec![
                "[ERROR] Failed to execute goal on project byteman-bmunit5: Could not resolve dependencies for project org.jboss.byteman:byteman-bmunit5:jar:4.0.7: The following artifacts could not be resolved: org.junit.jupiter:junit-jupiter-api:jar:5.4.0, org.junit.jupiter:junit-jupiter-params:jar:5.4.0, org.junit.jupiter:junit-jupiter-engine:jar:5.4.0: Cannot access central (https://repo.maven.apache.org/maven2) in offline mode and the artifact org.junit.jupiter:junit-jupiter-api:jar:5.4.0 has not been downloaded from it before. -> [Help 1]"
            ],
            1,
            Some(MissingMavenArtifacts(
                vec![
                    "org.junit.jupiter:junit-jupiter-api:jar:5.4.0".to_string(),
                    "org.junit.jupiter:junit-jupiter-params:jar:5.4.0".to_string(),
                    "org.junit.jupiter:junit-jupiter-engine:jar:5.4.0".to_string(),
                ]
            )),
        );
        assert_match(
            vec![
                "[ERROR] Failed to execute goal on project opennlp-uima: Could not resolve dependencies for project org.apache.opennlp:opennlp-uima:jar:1.9.2-SNAPSHOT: Cannot access ApacheIncubatorRepository (http://people.apache.org/repo/m2-incubating-repository/) in offline mode and the artifact org.apache.opennlp:opennlp-tools:jar:debian has not been downloaded from it before. -> [Help 1]"
            ],
            1,
            Some(MissingMavenArtifacts(vec!["org.apache.opennlp:opennlp-tools:jar:debian".to_string()])),
        );
        assert_match(
            vec![
                "[ERROR] Failed to execute goal on project bookkeeper-server: Could not resolve dependencies for project org.apache.bookkeeper:bookkeeper-server:jar:4.4.0: Cannot access central (https://repo.maven.apache.org/maven2) in offline mode and the artifact io.netty:netty:jar:debian has not been downloaded from it before. -> [Help 1]"
            ],
            1,
            Some(MissingMavenArtifacts(vec!["io.netty:netty:jar:debian".to_string()])),
        );
        assert_match(
            vec![
                "[ERROR] Unresolveable build extension: Plugin org.apache.felix:maven-bundle-plugin:2.3.7 or one of its dependencies could not be resolved: Cannot access central (https://repo.maven.apache.org/maven2) in offline mode and the artifact org.apache.felix:maven-bundle-plugin:jar:2.3.7 has not been downloaded from it before. @"
            ],
            1,
            Some(MissingMavenArtifacts(vec!["org.apache.felix:maven-bundle-plugin:2.3.7".to_string()])),
        );
        assert_match(
            vec![
                "[ERROR] Plugin org.apache.maven.plugins:maven-jar-plugin:2.6 or one of its dependencies could not be resolved: Cannot access central (https://repo.maven.apache.org/maven2) in offline mode and the artifact org.apache.maven.plugins:maven-jar-plugin:jar:2.6 has not been downloaded from it before. -> [Help 1]"
            ],
            1,
            Some(MissingMavenArtifacts(vec!["org.apache.maven.plugins:maven-jar-plugin:2.6".to_string()])),
        );

        assert_match(
            vec![
                "[FATAL] Non-resolvable parent POM for org.joda:joda-convert:2.2.1: Cannot access central (https://repo.maven.apache.org/maven2) in offline mode and the artifact org.joda:joda-parent:pom:1.4.0 has not been downloaded from it before. and 'parent.relativePath' points at wrong local POM @ line 8, column 10"],
            1,
            Some(MissingMavenArtifacts(vec!["org.joda:joda-parent:pom:1.4.0".to_string()])),
        );

        assert_match(
            vec![
                "[ivy:retrieve] \t\t:: com.carrotsearch.randomizedtesting#junit4-ant;${/com.carrotsearch.randomizedtesting/junit4-ant}: not found"
            ],
            1,
            Some(MissingMavenArtifacts(
                vec!["com.carrotsearch.randomizedtesting:junit4-ant:jar:debian".to_string()]
            )),
        );
        assert_match(
            vec![
                "[ERROR] Plugin org.apache.maven.plugins:maven-compiler-plugin:3.10.1 or one of its dependencies could not be resolved: Failed to read artifact descriptor for org.apache.maven.plugins:maven-compiler-plugin:jar:3.10.1: 1 problem was encountered while building the effective model for org.apache.maven.plugins:maven-compiler-plugin:3.10.1"
            ],
            1,
            Some(MissingMavenArtifacts(
                vec!["org.apache.maven.plugins:maven-compiler-plugin:3.10.1".to_string()]
            )),
        );
    }

    #[test]
    fn test_maven_errors() {
        assert_just_match(
            vec![
                "[ERROR] Failed to execute goal org.apache.maven.plugins:maven-jar-plugin:3.1.2:jar (default-jar) on project xslthl: Execution default-jar of goal org.apache.maven.plugins:maven-jar-plugin:3.1.2:jar failed: An API incompatibility was encountered while executing org.apache.maven.plugins:maven-jar-plugin:3.1.2:jar: java.lang.NoSuchMethodError: 'void org.codehaus.plexus.util.DirectoryScanner.setFilenameComparator(java.util.Comparator)'"],
            1,
        );
    }

    #[test]
    fn test_dh_missing_uninstalled() {
        assert_match(
            vec![
                "dh_missing --fail-missing", "dh_missing: usr/share/man/man1/florence_applet.1 exists in debian/tmp but is not installed to anywhere", "dh_missing: usr/lib/x86_64-linux-gnu/libflorence-1.0.la exists in debian/tmp but is not installed to anywhere", "dh_missing: missing files, aborting",
            ],
            3,
            Some(DhMissingUninstalled("usr/lib/x86_64-linux-gnu/libflorence-1.0.la".to_owned())),
        );
    }

    #[test]
    fn test_missing_perl_module() {
        assert_match(
            vec![
                "Converting tags.ledger... Can't locate String/Interpolate.pm in @INC (you may need to install the String::Interpolate module) (@INC contains: /etc/perl /usr/local/lib/x86_64-linux-gnu/perl/5.28.1 /usr/local/share/perl/5.28.1 /usr/lib/x86_64-linux-gnu/perl5/5.28 /usr/share/perl5 /usr/lib/x86_64-linux-gnu/perl/5.28 /usr/share/perl/5.28 /usr/local/lib/site_perl /usr/lib/x86_64-linux-gnu/perl-base) at ../bin/ledger2beancount line 23."
            ],
            1,
            Some(MissingPerlModule {
                filename: Some("String/Interpolate.pm".to_owned()),
                module: "String::Interpolate".to_owned(),
                inc: Some(vec![
                    "/etc/perl".to_owned(),
                    "/usr/local/lib/x86_64-linux-gnu/perl/5.28.1".to_owned(),
                    "/usr/local/share/perl/5.28.1".to_owned(),
                    "/usr/lib/x86_64-linux-gnu/perl5/5.28".to_owned(),
                    "/usr/share/perl5".to_owned(),
                    "/usr/lib/x86_64-linux-gnu/perl/5.28".to_owned(),
                    "/usr/share/perl/5.28".to_owned(),
                    "/usr/local/lib/site_perl".to_owned(),
                    "/usr/lib/x86_64-linux-gnu/perl-base".to_owned(),
                ]),
                minimum_version: None
            }),
        );
        assert_match(
            vec![
                "Can't locate Test/Needs.pm in @INC (you may need to install the Test::Needs module) (@INC contains: t/lib /<<PKGBUILDDIR>>/blib/lib /<<PKGBUILDDIR>>/blib/arch /etc/perl /usr/local/lib/x86_64-linux-gnu/perl/5.30.0 /usr/local/share/perl/5.30.0 /usr/lib/x86_64-linux-gnu/perl5/5.30 /usr/share/perl5 /usr/lib/x86_64-linux-gnu/perl/5.30 /usr/share/perl/5.30 /usr/local/lib/site_perl /usr/lib/x86_64-linux-gnu/perl-base .) at t/anon-basic.t line 7."
            ],
            1,
            Some(MissingPerlModule{
                filename: Some("Test/Needs.pm".to_owned()),
                module: "Test::Needs".to_owned(),
                inc: Some(vec![
                    "t/lib".to_owned(),
                    "/<<PKGBUILDDIR>>/blib/lib".to_owned(),
                    "/<<PKGBUILDDIR>>/blib/arch".to_owned(),
                    "/etc/perl".to_owned(),
                    "/usr/local/lib/x86_64-linux-gnu/perl/5.30.0".to_owned(),
                    "/usr/local/share/perl/5.30.0".to_owned(),
                    "/usr/lib/x86_64-linux-gnu/perl5/5.30".to_owned(),
                    "/usr/share/perl5".to_owned(),
                    "/usr/lib/x86_64-linux-gnu/perl/5.30".to_owned(),
                    "/usr/share/perl/5.30".to_owned(),
                    "/usr/local/lib/site_perl".to_owned(),
                    "/usr/lib/x86_64-linux-gnu/perl-base".to_owned(),
                    ".".to_owned(),
                ]),
                minimum_version: None
            }),
        );
        assert_match(
            vec!["- ExtUtils::Depends         ...missing. (would need 0.302)"],
            1,
            Some(MissingPerlModule {
                filename: None,
                module: "ExtUtils::Depends".to_owned(),
                inc: None,
                minimum_version: Some("0.302".to_owned()),
            }),
        );
        assert_match(
            vec![
                r#"Can't locate object method "new" via package "Dist::Inkt::Profile::TOBYINK" (perhaps you forgot to load "Dist::Inkt::Profile::TOBYINK"?) at /usr/share/perl5/Dist/Inkt.pm line 208."#,
            ],
            1,
            Some(MissingPerlModule::simple("Dist::Inkt::Profile::TOBYINK")),
        );
        assert_match(
            vec![
                "Can't locate ExtUtils/Depends.pm in @INC (you may need to install the ExtUtils::Depends module) (@INC contains: /etc/perl /usr/local/lib/x86_64-linux-gnu/perl/5.32.1 /usr/local/share/perl/5.32.1 /usr/lib/x86_64-linux-gnu/perl5/5.32 /usr/share/perl5 /usr/lib/x86_64-linux-gnu/perl-base /usr/lib/x86_64-linux-gnu/perl/5.32 /usr/share/perl/5.32 /usr/local/lib/site_perl) at (eval 11) line 1."
            ],
            1,
            Some(MissingPerlModule{
                filename: Some("ExtUtils/Depends.pm".to_owned()),
                module: "ExtUtils::Depends".to_owned(),
                inc: Some(vec![
                    "/etc/perl".to_owned(),
                    "/usr/local/lib/x86_64-linux-gnu/perl/5.32.1".to_owned(),
                    "/usr/local/share/perl/5.32.1".to_owned(),
                    "/usr/lib/x86_64-linux-gnu/perl5/5.32".to_owned(),
                    "/usr/share/perl5".to_owned(),
                    "/usr/lib/x86_64-linux-gnu/perl-base".to_owned(),
                    "/usr/lib/x86_64-linux-gnu/perl/5.32".to_owned(),
                    "/usr/share/perl/5.32".to_owned(),
                    "/usr/local/lib/site_perl".to_owned(),
                ]),
                minimum_version: None
            }),
        );
        assert_match(
            vec![
                "Pod::Weaver::Plugin::WikiDoc (for section -WikiDoc) does not appear to be installed"
            ],
            1,
            Some(MissingPerlModule::simple("Pod::Weaver::Plugin::WikiDoc")),
        );
        assert_match(
            vec![
                "List::Util version 1.56 required--this is only version 1.55 at /build/tmpttq5hhpt/package/blib/lib/List/AllUtils.pm line 8."
            ],
            1,
            Some(MissingPerlModule {
                filename: None,
                inc: None,
                module: "List::Util".to_owned(), minimum_version: Some("1.56".to_owned())}),
        );
    }

    #[test]
    fn test_missing_perl_file() {
        assert_match(
            vec![
                "Can't locate debian/perldl.conf in @INC (@INC contains: /<<PKGBUILDDIR>>/inc /etc/perl /usr/local/lib/x86_64-linux-gnu/perl/5.28.1 /usr/local/share/perl/5.28.1 /usr/lib/x86_64-linux-gnu/perl5/5.28 /usr/share/perl5 /usr/lib/x86_64-linux-gnu/perl/5.28 /usr/share/perl/5.28 /usr/local/lib/site_perl /usr/lib/x86_64-linux-gnu/perl-base) at Makefile.PL line 131."
            ],
            1,
            Some(MissingPerlFile {
                filename: "debian/perldl.conf".to_owned(),
                inc: Some(vec![
                    "/<<PKGBUILDDIR>>/inc".to_owned(),
                    "/etc/perl".to_owned(),
                    "/usr/local/lib/x86_64-linux-gnu/perl/5.28.1".to_owned(),
                    "/usr/local/share/perl/5.28.1".to_owned(),
                    "/usr/lib/x86_64-linux-gnu/perl5/5.28".to_owned(),
                    "/usr/share/perl5".to_owned(),
                    "/usr/lib/x86_64-linux-gnu/perl/5.28".to_owned(),
                    "/usr/share/perl/5.28".to_owned(),
                    "/usr/local/lib/site_perl".to_owned(),
                    "/usr/lib/x86_64-linux-gnu/perl-base".to_owned(),
                ]),
            }),
        );
        assert_match(
            vec![r#"Can't open perl script "Makefile.PL": No such file or directory"#],
            1,
            Some(MissingPerlFile {
                filename: "Makefile.PL".to_owned(),
                inc: None,
            }),
        );
    }

    #[test]
    fn test_perl_expand() {
        assert_match(
            vec![">(error): Could not expand [ 'Dist::Inkt::Profile::TOBYINK'"],
            1,
            Some(MissingPerlModule::simple("Dist::Inkt::Profile::TOBYINK")),
        );
    }

    #[test]
    fn test_perl_missing_predeclared() {
        assert_match(
            vec![
                "String found where operator expected at Makefile.PL line 13, near \"author_tests 'xt'\"", "\t(Do you need to predeclare author_tests?)",
                "syntax error at Makefile.PL line 13, near \"author_tests 'xt'\"", r#""strict subs" in use at Makefile.PL line 13."#,
            ],
            2,
            Some(MissingPerlPredeclared("author_tests".to_owned())),
        );
        assert_match(
            vec![
                "String found where operator expected at Makefile.PL line 8, near \"readme_from    'lib/URL/Encode.pod'\""
            ],
            1,
            Some(MissingPerlPredeclared("readme_from".to_owned())),
        );

        assert_match(
            vec![
                r#"Bareword "use_test_base" not allowed while "strict subs" in use at Makefile.PL line 12."#,
            ],
            1,
            Some(MissingPerlPredeclared("use_test_base".to_owned())),
        );
    }

    #[test]
    fn test_unknown_cert_authority() {
        assert_match(
            vec![
                r#"go: github.com/golangci/golangci-lint@v1.24.0: Get "https://proxy.golang.org/github.com/golangci/golangci-lint/@v/v1.24.0.mod": x509: certificate signed by unknown authority"#,
            ],
            1,
            Some(UnknownCertificateAuthority(
                "https://proxy.golang.org/github.com/golangci/golangci-lint/@v/v1.24.0.mod"
                    .to_owned(),
            )),
        );
    }

    #[test]
    fn test_no_disk_space() {
        assert_match(
            vec![
                "/usr/bin/install: error writing '/<<PKGBUILDDIR>>/debian/tmp/usr/lib/gcc/x86_64-linux-gnu/8/cc1objplus': No space left on device"
            ],
            1,
            Some(NoSpaceOnDevice)
        );

        assert_match(
            ["OSError: [Errno 28] No space left on device"].to_vec(),
            1,
            Some(NoSpaceOnDevice),
        );
    }

    #[test]
    fn test_segmentation_fault() {
        assert_just_match(
            vec![
                r#"/bin/bash: line 3:  7392 Segmentation fault      itstool -m "${mo}" ${d}/C/index.docbook ${d}/C/legal.xml"#,
            ],
            1,
        );
    }

    #[test]
    fn test_missing_perl_plugin() {
        assert_match(
            vec!["Required plugin bundle Dist::Zilla::PluginBundle::Git isn't installed."],
            1,
            Some(MissingPerlModule::simple("Dist::Zilla::PluginBundle::Git")),
        );
        assert_match(
            vec!["Required plugin Dist::Zilla::Plugin::PPPort isn't installed."],
            1,
            Some(MissingPerlModule::simple("Dist::Zilla::Plugin::PPPort")),
        );
    }

    #[test]
    fn test_nim_error() {
        assert_just_match(
            vec![
                "/<<PKGBUILDDIR>>/msgpack4nim.nim(470, 6) Error: usage of 'isNil' is a user-defined error",
            ],
            1,
        );
    }

    #[test]
    fn test_scala_error() {
        assert_just_match(
            vec![
                "core/src/main/scala/org/json4s/JsonFormat.scala:131: error: No JSON deserializer found for type List[T]. Try to implement an implicit Reader or JsonFormat for this type."
            ],
            1,
        );
    }

    #[test]
    fn test_vala_error() {
        assert_just_match(
vec![
                "../src/Backend/FeedServer.vala:60.98-60.148: error: The name `COLLECTION_CREATE_NONE' does not exist in the context of `Secret.CollectionCreateFlags'"
            ],
            1,
        );
        assert_match(
            vec![
                "error: Package `glib-2.0' not found in specified Vala API directories or GObject-Introspection GIR directories"
            ],
            1,
            Some(MissingValaPackage("glib-2.0".to_owned())),
        );
    }

    #[test]
    fn test_gir() {
        assert_match(
            vec!["ValueError: Namespace GnomeDesktop not available"],
            1,
            Some(MissingIntrospectionTypelib("GnomeDesktop".to_owned())),
        );
    }

    #[test]
    fn test_missing_boost_components() {
        assert_match(
            r#"""CMake Error at /usr/share/cmake-3.18/Modules/FindPackageHandleStandardArgs.cmake:165 (message):
  Could NOT find Boost (missing: program_options filesystem system graph
  serialization iostreams) (found suitable version "1.74.0", minimum required
  is "1.55.0")
Call Stack (most recent call first):
  /usr/share/cmake-3.18/Modules/FindPackageHandleStandardArgs.cmake:458 (_FPHSA_FAILURE_MESSAGE)
  /usr/share/cmake-3.18/Modules/FindBoost.cmake:2177 (find_package_handle_standard_args)
  src/CMakeLists.txt:4 (find_package)
"""#.split_inclusive('\n').collect::<Vec<&str>>(),
            4,
            Some(MissingCMakeComponents{
                name: "Boost".to_owned(),
                components: vec![
                    "program_options".to_owned(),
                    "filesystem".to_owned(),
                    "system".to_owned(),
                    "graph".to_owned(),
                    "serialization".to_owned(),
                    "iostreams".to_owned(),
                ],
            }),
        );
    }

    #[test]
    fn test_pkg_config_too_old() {
        assert_match(
            vec![
                "checking for pkg-config... no",
                "",
                "*** Your version of pkg-config is too old. You need atleast",
                "*** pkg-config 0.9.0 or newer. You can download pkg-config",
                "*** from the freedesktop.org software repository at",
                "***",
                "***    https://www.freedesktop.org/wiki/Software/pkg-config/",
                "***",
            ],
            4,
            Some(MissingVagueDependency {
                name: "pkg-config".to_owned(),
                minimum_version: Some("0.9.0".to_owned()),
                url: None,
                current_version: None,
            }),
        );
    }

    #[test]
    fn test_missing_jdk() {
        assert_match(
            vec![
                "> Kotlin could not find the required JDK tools in the Java installation '/usr/lib/jvm/java-8-openjdk-amd64/jre' used by Gradle. Make sure Gradle is running on a JDK, not JRE.",
            ],
            1,
            Some(MissingJDK::new("/usr/lib/jvm/java-8-openjdk-amd64/jre".to_owned())),
        );
    }

    #[test]
    fn test_missing_jre() {
        assert_match(
            vec!["ERROR: JAVA_HOME is not set and no 'java' command could be found in your PATH."],
            1,
            Some(MissingJRE),
        );
    }

    #[test]
    fn test_node_module_missing() {
        assert_match(
            vec!["Error: Cannot find module 'tape'"],
            1,
            Some(MissingNodeModule("tape".to_owned())),
        );
        assert_just_match(
            vec!["✖ [31mERROR:[39m Cannot find module '/<<PKGBUILDDIR>>/test'"],
            1,
        );
        assert_match(
            vec!["npm ERR! [!] Error: Cannot find module '@rollup/plugin-buble'"],
            1,
            Some(MissingNodeModule("@rollup/plugin-buble".to_owned())),
        );
        assert_match(
            vec!["npm ERR! Error: Cannot find module 'fs-extra'"],
            1,
            Some(MissingNodeModule("fs-extra".to_owned())),
        );
        assert_match(
            vec!["\x1b[1m\x1b[31m[!] \x1b[1mError: Cannot find module '@rollup/plugin-buble'"],
            1,
            Some(MissingNodeModule("@rollup/plugin-buble".to_owned())),
        );
    }

    #[test]
    fn test_setup_py_command() {
        assert_match(
            r#"""/usr/lib/python3.9/distutils/dist.py:274: UserWarning: Unknown distribution option: 'long_description_content_type'
  warnings.warn(msg)
/usr/lib/python3.9/distutils/dist.py:274: UserWarning: Unknown distribution option: 'test_suite'
  warnings.warn(msg)
/usr/lib/python3.9/distutils/dist.py:274: UserWarning: Unknown distribution option: 'python_requires'
  warnings.warn(msg)
usage: setup.py [global_opts] cmd1 [cmd1_opts] [cmd2 [cmd2_opts] ...]
   or: setup.py --help [cmd1 cmd2 ...]
   or: setup.py --help-commands
   or: setup.py cmd --help

error: invalid command 'test'
"""#.split_inclusive('\n').collect::<Vec<&str>>(),
            12,
            Some(MissingSetupPyCommand("test".to_owned())),
        );
    }

    #[test]
    fn test_c_header_missing() {
        assert_match(
            vec!["cdhit-common.h:39:9: fatal error: zlib.h: No such file or directory"],
            1,
            Some(MissingCHeader {
                header: "zlib.h".to_owned(),
            }),
        );
        assert_match(
            vec![
                "/<<PKGBUILDDIR>>/Kernel/Operation_Vector.cpp:15:10: fatal error: petscvec.h: No such file or directory"
            ],
            1,
            Some(MissingCHeader{header: "petscvec.h".to_owned()}),
        );
        assert_match(
            vec!["src/bubble.h:27:10: fatal error: DBlurEffectWidget: No such file or directory"],
            1,
            Some(MissingCHeader {
                header: "DBlurEffectWidget".to_owned(),
            }),
        );
    }

    #[test]
    fn test_missing_jdk_file() {
        assert_match(
            vec![
                "> Could not find tools.jar. Please check that /usr/lib/jvm/java-8-openjdk-amd64 contains a valid JDK installation.",
            ],
            1,
            Some(MissingJDKFile{jdk_path: "/usr/lib/jvm/java-8-openjdk-amd64".to_owned(), filename: "tools.jar".to_owned()}),
        );
    }

    #[test]
    fn test_python2_import() {
        assert_match(
            vec!["ImportError: No module named pytz"],
            1,
            Some(MissingPythonModule::simple("pytz".to_owned())),
        );
        assert_just_match(vec!["ImportError: cannot import name SubfieldBase"], 1);
    }

    #[test]
    fn test_python3_import() {
        assert_match(
            ["ModuleNotFoundError: No module named 'django_crispy_forms'"].to_vec(),
            1,
            Some(MissingPythonModule {
                module: "django_crispy_forms".to_owned(),
                python_version: Some(3),
                minimum_version: None,
            }),
        );
        assert_match(
            [" ModuleNotFoundError: No module named 'Cython'"].to_vec(),
            1,
            Some(MissingPythonModule {
                module: "Cython".to_owned(),
                python_version: Some(3),
                minimum_version: None,
            }),
        );
        assert_match(
            ["ModuleNotFoundError: No module named 'distro'"].to_vec(),
            1,
            Some(MissingPythonModule {
                module: "distro".to_owned(),
                python_version: Some(3),
                minimum_version: None,
            }),
        );
        assert_match(
            ["E   ModuleNotFoundError: No module named 'twisted'"].to_vec(),
            1,
            Some(MissingPythonModule {
                module: "twisted".to_owned(),
                python_version: Some(3),
                minimum_version: None,
            }),
        );
        assert_match(
            vec![
                "E   ImportError: cannot import name 'async_poller' from 'msrest.polling' (/usr/lib/python3/dist-packages/msrest/polling/__init__.py)"
            ],
            1,
            Some(MissingPythonModule::simple("msrest.polling.async_poller".to_owned())),
        );
        assert_match(
            vec!["/usr/bin/python3: No module named sphinx"],
            1,
            Some(MissingPythonModule {
                module: "sphinx".to_owned(),
                python_version: Some(3),
                minimum_version: None,
            }),
        );
        assert_match(
            vec![
                "Could not import extension sphinx.ext.pngmath (exception: No module named pngmath)"
            ],
            1,
            Some(MissingPythonModule::simple("pngmath".to_owned())),
        );
        assert_match(
            vec![
                "/usr/bin/python3: Error while finding module specification for 'pep517.build' (ModuleNotFoundError: No module named 'pep517')"
            ],
            1,
            Some(MissingPythonModule{module: "pep517".to_owned(), python_version:Some(3), minimum_version: None}),
        );
    }

    #[test]
    fn test_sphinx() {
        assert_just_match(
            vec!["There is a syntax error in your configuration file: Unknown syntax: Constant"],
            1,
        );
    }

    #[test]
    fn test_go_missing() {
        assert_match(
            vec![
                r#"src/github.com/vuls/config/config.go:30:2: cannot find package "golang.org/x/xerrors" in any of:"#,
            ],
            1,
            Some(MissingGoPackage {
                package: "golang.org/x/xerrors".to_owned(),
            }),
        );
    }

    #[test]
    fn test_lazy_font() {
        assert_match(
            vec![
                "[ERROR] LazyFont - Failed to read font file /usr/share/texlive/texmf-dist/fonts/opentype/public/stix2-otf/STIX2Math.otf <java.io.FileNotFoundException: /usr/share/texlive/texmf-dist/fonts/opentype/public/stix2-otf/STIX2Math.otf (No such file or directory)>java.io.FileNotFoundException: /usr/share/texlive/texmf-dist/fonts/opentype/public/stix2-otf/STIX2Math.otf (No such file or directory)"],
            1,
            Some(MissingFile::new(
                "/usr/share/texlive/texmf-dist/fonts/opentype/public/stix2-otf/STIX2Math.otf".into()
            )),
        );
    }

    #[test]
    fn test_missing_latex_files() {
        assert_match(
            vec!["! LaTeX Error: File `fancyvrb.sty' not found."],
            1,
            Some(MissingLatexFile("fancyvrb.sty".to_owned())),
        );
    }

    #[test]
    fn test_pytest_import() {
        assert_match(
            vec!["E   ImportError: cannot import name cmod"],
            1,
            Some(MissingPythonModule::simple("cmod".to_owned())),
        );
        assert_match(
            vec!["E   ImportError: No module named mock"],
            1,
            Some(MissingPythonModule::simple("mock".to_owned())),
        );
        assert_match(
            vec![
                "pluggy.manager.PluginValidationError: Plugin 'xdist.looponfail' could not be loaded: (pytest 3.10.1 (/usr/lib/python2.7/dist-packages), Requirement.parse('pytest>=4.4.0'))!"
            ],
            1,
            Some(MissingPythonModule{
                module: "pytest".to_owned(), python_version: Some(2), minimum_version: Some("4.4.0".to_owned())
            }),
        );
        assert_match(
            vec![
                r#"ImportError: Error importing plugin "tests.plugins.mock_libudev": No module named mock"#,
            ],
            1,
            Some(MissingPythonModule::simple("mock".to_owned())),
        );
    }

    #[test]
    fn test_sed() {
        assert_match(
            vec!["sed: can't read /etc/locale.gen: No such file or directory"],
            1,
            Some(MissingFile::new("/etc/locale.gen".into())),
        );
    }

    #[test]
    fn test_pytest_args() {
        assert_match(
            vec![
                "pytest: error: unrecognized arguments: --cov=janitor --cov-report=html --cov-report=term-missing:skip-covered"
            ],
            1,
            Some(UnsupportedPytestArguments(
                vec![
                    "--cov=janitor".to_owned(),
                    "--cov-report=html".to_owned(),
                    "--cov-report=term-missing:skip-covered".to_owned(),
                ]
            )),
        );
    }

    #[test]
    fn test_pytest_config() {
        assert_match(
            vec!["INTERNALERROR> pytest.PytestConfigWarning: Unknown config option: asyncio_mode"],
            1,
            Some(UnsupportedPytestConfigOption("asyncio_mode".to_owned())),
        );
    }

    #[test]
    fn test_distutils_missing() {
        assert_match(
            vec![
                "distutils.errors.DistutilsError: Could not find suitable distribution for Requirement.parse('pytest-runner')"
            ],
            1,
            Some(MissingPythonDistribution::simple("pytest-runner")),
        );
        assert_match(
            vec![
                "distutils.errors.DistutilsError: Could not find suitable distribution for Requirement.parse('certifi>=2019.3.9')"
            ],
            1,
            Some(MissingPythonDistribution{distribution: "certifi".to_owned(), minimum_version: Some("2019.3.9".to_owned()), python_version: None }),
        );
        assert_match(
            vec![
                r#"distutils.errors.DistutilsError: Could not find suitable distribution for Requirement.parse('cffi; platform_python_implementation == "CPython"\')"#,
            ],
            1,
            Some(MissingPythonDistribution::simple("cffi")),
        );
        assert_match(
            vec!["error: Could not find suitable distribution for Requirement.parse('gitlab')"],
            1,
            Some(MissingPythonDistribution::simple("gitlab")),
        );
        assert_match(
            vec![
                "pkg_resources.DistributionNotFound: The 'configparser>=3.5' distribution was not found and is required by importlib-metadata"
            ],
            1,
            Some(MissingPythonDistribution{distribution:"configparser".to_owned(), minimum_version: Some("3.5".to_owned()), python_version: None}),
        );
        assert_match(
            vec![
                "error: Command '['/usr/bin/python3.9', '-m', 'pip', '--disable-pip-version-check', 'wheel', '--no-deps', '-w', '/tmp/tmp973_8lhm', '--quiet', 'asynctest']' returned non-zero exit status 1."
            ],
            1,
            Some(MissingPythonDistribution{distribution: "asynctest".to_owned(), python_version:Some(3), minimum_version: None}),
        );
        assert_match(
            vec![
                "subprocess.CalledProcessError: Command '['/usr/bin/python', '-m', 'pip', '--disable-pip-version-check', 'wheel', '--no-deps', '-w', '/tmp/tmpm2l3kcgv', '--quiet', 'setuptools_scm']' returned non-zero exit status 1."
            ],
            1,
            Some(MissingPythonDistribution::simple("setuptools-scm")),
        );
    }

    #[test]
    fn test_cmake_missing_file() {
        assert_match(
            r#"""CMake Error at /usr/lib/x86_64-/cmake/Qt5Gui/Qt5GuiConfig.cmake:27 (message):
  The imported target "Qt5::Gui" references the file

     "/usr/lib/x86_64-linux-gnu/libEGL.so"

  but this file does not exist.  Possible reasons include:

  * The file was deleted, renamed, or moved to another location.

  * An install or uninstall procedure did not complete successfully.

  * The installation package was faulty and contained

     "/usr/lib/x86_64-linux-gnu/cmake/Qt5Gui/Qt5GuiConfigExtras.cmake"

  but not all the files it references.

Call Stack (most recent call first):
  /usr/lib/x86_64-linux-gnu/QtGui/Qt5Gui.cmake:63 (_qt5_Gui_check_file_exists)
  /usr/lib/x86_64-linux-gnu/QtGui/Qt5Gui.cmake:85 (_qt5gui_find_extra_libs)
  /usr/lib/x86_64-linux-gnu/QtGui/Qt5Gui.cmake:186 (include)
  /usr/lib/x86_64-linux-gnu/QtWidgets/Qt5Widgets.cmake:101 (find_package)
  /usr/lib/x86_64-linux-gnu/Qt/Qt5Config.cmake:28 (find_package)
  CMakeLists.txt:34 (find_package)
dh_auto_configure: cd obj-x86_64-linux-gnu && cmake with args
"""#
            .split_inclusive('\n')
            .collect::<Vec<&str>>(),
            16,
            Some(MissingFile::new(
                "/usr/lib/x86_64-linux-gnu/libEGL.so".into(),
            )),
        );
    }

    #[test]
    fn test_cmake_missing_include() {
        assert_match(
            r#"""-- Performing Test _OFFT_IS_64BIT
-- Performing Test _OFFT_IS_64BIT - Success
-- Performing Test HAVE_DATE_TIME
-- Performing Test HAVE_DATE_TIME - Success
CMake Error at CMakeLists.txt:43 (include):
  include could not find load file:

    KDEGitCommitHooks


-- Found KF5Activities: /usr/lib/x86_64-linux-gnu/cmake/KF5Activities/KF5ActivitiesConfig.cmake (found version "5.78.0") 
-- Found KF5Config: /usr/lib/x86_64-linux-gnu/cmake/KF5Config/KF5ConfigConfig.cmake (found version "5.78.0") 
"""#.split_inclusive('\n').collect::<Vec<&str>>(),
            8,
            Some(CMakeFilesMissing{filenames:vec!["KDEGitCommitHooks.cmake".to_string()], version :None}),
        );
    }

    #[test]
    fn test_cmake_missing_cmake_files() {
        assert_match(
            r#"""CMake Error at /usr/share/cmake-3.22/Modules/FindPackageHandleStandardArgs.cmake:230 (message):
  Could not find a package configuration file provided by "sensor_msgs" with
  any of the following names:

    sensor_msgsConfig.cmake
    sensor_msgs-config.cmake

  Add the installation prefix of "sensor_msgs" to CMAKE_PREFIX_PATH or set
  "sensor_msgs_DIR" to a directory containing one of the above files.  If
  "sensor_msgs" provides a separate development package or SDK, be sure it
  has been installed.
dh_auto_configure: cd obj-x86_64-linux-gnu && cmake with args
"""#
            .split_inclusive('\n')
            .collect::<Vec<&str>>(),
            11,
            Some(CMakeFilesMissing {
                filenames: vec![
                    "sensor_msgsConfig.cmake".to_string(),
                    "sensor_msgs-config.cmake".to_string(),
                ],
                version: None,
            }),
        );
        assert_match(
            r#"""CMake Error at /usr/share/cmake-3.22/Modules/FindPackageHandleStandardArgs.cmake:230 (message):
  Could NOT find KF5 (missing: Plasma PlasmaQuick Wayland ModemManagerQt
  NetworkManagerQt) (found suitable version "5.92.0", minimum required is
  "5.86")
"""#.split_inclusive('\n').collect::<Vec<&str>>(),
            4,
            Some(MissingCMakeComponents{
                name: "KF5".into(),
                components: vec![
                    "Plasma".into(),
                    "PlasmaQuick".into(),
                    "Wayland".into(),
                    "ModemManagerQt".into(),
                    "NetworkManagerQt".into(),
                ],
            }),
        );
    }

    #[test]
    fn test_cmake_missing_exact_version() {
        assert_match(
            r#"""CMake Error at /usr/share/cmake-3.18/Modules/FindPackageHandleStandardArgs.cmake:165 (message):
  Could NOT find SignalProtocol: Found unsuitable version "2.3.3", but
  required is exact version "2.3.2" (found
  /usr/lib/x86_64-linux-gnu/libsignal-protocol-c.so)
"""#.split_inclusive('\n').collect::<Vec<&str>>(),
            4,
            Some(CMakeNeedExactVersion{
                package: "SignalProtocol".to_owned(),
                version_found: "2.3.3".to_owned(),
                exact_version_needed: "2.3.2".to_owned(),
                path: "/usr/lib/x86_64-linux-gnu/libsignal-protocol-c.so".into(),
            }),
        );
    }

    #[test]
    fn test_cmake_missing_vague() {
        assert_match(
            vec![
                "CMake Error at CMakeLists.txt:84 (MESSAGE):",
                "  alut not found",
            ],
            2,
            Some(MissingVagueDependency::simple("alut")),
        );
        assert_match(
            vec![
                "CMake Error at CMakeLists.txt:213 (message):",
                "  could not find zlib",
            ],
            2,
            Some(MissingVagueDependency::simple("zlib")),
        );
        assert_match(
            r#"""-- Found LibSolv_ext: /usr/lib/x86_64-linux-gnu/libsolvext.so  
-- Found LibSolv: /usr/include /usr/lib/x86_64-linux-gnu/libsolv.so;/usr/lib/x86_64-linux-gnu/libsolvext.so
-- No usable gpgme flavours found.
CMake Error at cmake/modules/FindGpgme.cmake:398 (message):
  Did not find GPGME
Call Stack (most recent call first):
  CMakeLists.txt:223 (FIND_PACKAGE)
  """#.split_inclusive('\n').collect::<Vec<&str>>(),
            5,
            Some(MissingVagueDependency::simple("GPGME")),
        );
    }

    #[test]
    fn test_secondary() {
        assert!(super::find_secondary_build_failure(&["Unknown option --foo"], 10).is_some());
        assert!(
            super::find_secondary_build_failure(&["Unknown option --foo, ignoring."], 10).is_none()
        );
    }
}
