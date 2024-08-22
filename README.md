The build log consultant can parse and analyse build log files.

Currently supported container formats:

 * sbuild
 * plain

For a longer introduction, see the
[blog post](https://www.jelmer.uk/buildlog-consultant.html).

## Example usage

```console

$ analyze-sbuild-log < build.log
Error: unsatisfied apt dependencies: librust-breezyshim+dirty-tracker-dev:amd64 (>= 0.1.138-\~\~)
Issue found at lines 105-120:
    (I)Dose_deb: Parsing Packages file -...
    (I)Dose_common: total packages 71128
    (I)Dose_applications: Cudf Universe: 71128 packages
    (I)Dose_applications: --checkonly specified, consider all packages as background packages
    (I)Dose_applications: Solving...
 >  output-version: 1.2
 >  native-architecture: amd64
 >  report:
 >   -
 >    package: sbuild-build-depends-main-dummy
 >    version: 0.invalid.0
 >    architecture: amd64
 >    status: broken
 >    reasons:
 >     -
 >      missing:
 >       pkg:
 >        package: sbuild-build-depends-main-dummy
 >        version: 0.invalid.0
 >        architecture: amd64
 >        unsat-dependency: librust-breezyshim+dirty-tracker-dev:amd64 (>= 0.1.138-~~)

    background-packages: 71127
    foreground-packages: 1
    total-packages: 71128
    broken-packages: 1
Identified issue: unsatisfied apt dependencies: librust-breezyshim+dirty-tracker-dev:amd64 (>= 0.1.138-\~\~)
```

Or using the JSON output:

```console
$ analyze-sbuild-log --json < build.log
{
  "details": {
    "relations": "librust-breezyshim+dirty-tracker-dev:amd64 (>= 0.1.138-\~\~)"
  },
  "line": "      unsat-dependency: librust-breezyshim+dirty-tracker-dev:amd64 (>= 0.1.138-\~\~)\n",
  "lineno": 120,
  "problem": "unsatisfied-apt-dependencies"
}
```
