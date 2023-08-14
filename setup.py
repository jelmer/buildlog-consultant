#!/usr/bin/python3
from setuptools import setup
from setuptools_rust import Binding, RustExtension

setup(
    rust_extensions=[RustExtension("buildlog_consultant._buildlog_consultant_rs", "buildlog-consultant-py/Cargo.toml", binding=Binding.PyO3)],
)
