all: check flake8 mypy

check:
	python3 setup.py test

flake8:
	flake8

mypy:
	python3 -m mypy buildlog_consultant
