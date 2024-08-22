#!/usr/bin/python3
from setuptools import setup
from setuptools_rust import Binding, RustBin, RustExtension

setup(
    rust_extensions=[
        RustExtension(
            "buildlog_consultant._buildlog_consultant_rs",
            "buildlog-consultant-py/Cargo.toml",
            binding=Binding.PyO3,
            features = ["extension-module"],
        ),
        RustBin("analyze-build-log", "Cargo.toml", features=["cli"]),
        RustBin("analyze-apt-log", "Cargo.toml", features=["cli"]),
        RustBin("analyze-autopkgtest-log", "Cargo.toml", features=["cli"]),
        RustBin("chatgpt-analyze-log", "Cargo.toml", features=["cli", "chatgpt", "tokio"]),
        RustBin("analyze-sbuild-log", "Cargo.toml", features=["cli"]),
    ],
)
