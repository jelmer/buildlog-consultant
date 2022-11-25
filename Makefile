all: check

check:: testsuite

testsuite:
	python3 -m unittest tests.test_suite

check:: style

style:
	flake8

check:: typing

typing:
	python3 -m mypy buildlog_consultant
