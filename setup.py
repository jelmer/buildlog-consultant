#!/usr/bin/python3

from setuptools import setup

setup(
    name="buildlog-consultant",
    packages=[
        "buildlog_consultant",
    ],
    version="0.0.3",
    author="Jelmer Vernooij",
    author_email="jelmer@jelmer.uk",
    url="https://github.com/jelmer/buildlog-consultant",
    description="buildlog parser and analyser",
    project_urls={
        "Repository": "https://github.com/jelmer/buildlog-consultant.git",
    },
    test_suite="buildlog_consultant.tests.test_suite",
    install_requires=['python_debian', 'PyYAML'],
)
