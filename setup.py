#!/usr/bin/python3

from setuptools import setup

setup(
    name="buildlog-consultant",
    packages=[
        "buildlog_consultant",
        "buildlog_consultant.tests",
    ],
    version="0.0.6",
    author="Jelmer Vernooij",
    author_email="jelmer@jelmer.uk",
    url="https://github.com/jelmer/buildlog-consultant",
    description="buildlog parser and analyser",
    project_urls={
        "Repository": "https://github.com/jelmer/buildlog-consultant.git",
    },
    test_suite="buildlog_consultant.tests.test_suite",
    install_requires=['python_debian', 'PyYAML'],
    entry_points={
        'console_scripts': [
            ('analyse-sbuild-log='
             'buildlog_consultant.sbuild:main'),
            ('analyse-build-log='
             'buildlog_consultant.common:main'),
        ],
    },

)
