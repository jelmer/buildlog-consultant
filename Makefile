export PYTHON=python3

all: check

build-inplace:
	$(PYTHON) setup.py build_ext --inplace

check:: testsuite

testsuite: build-inplace
	$(PYTHON) -m unittest tests.test_suite

check:: style

style:
	flake8

check:: typing

typing:
	$(PYTHON) -m mypy buildlog_consultant
