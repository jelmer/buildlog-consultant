export PYTHON=python3

all: check

build-inplace:
	$(PYTHON) setup.py build_ext --inplace

check:: testsuite

testsuite: build-inplace
	$(PYTHON) -m unittest tests.test_suite

check:: cargo-test

cargo-test:
	cargo test

check:: style

style:
	ruff check .

check:: typing

typing: build-inplace
	$(PYTHON) -m mypy buildlog_consultant
