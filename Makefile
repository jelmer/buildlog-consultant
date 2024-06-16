export PYTHON=python3

all: check

build-inplace:
	$(PYTHON) setup.py build_ext --inplace

check:: testsuite

testsuite: build-inplace
	PYTHONPATH=$(shell pwd)/py $(PYTHON) -m unittest tests.test_suite

check:: cargo-test

cargo-test:
	cargo test

check:: style

style:
	ruff check py

check:: typing

typing: build-inplace
	$(PYTHON) -m mypy py/buildlog_consultant
