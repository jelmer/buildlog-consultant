all: check

build-inplace:
	python3 setup.py build_ext --inplace

check:: testsuite

testsuite: build-inplace
	python3 -m unittest tests.test_suite

check:: style

style:
	flake8

check:: typing

typing:
	python3 -m mypy buildlog_consultant
