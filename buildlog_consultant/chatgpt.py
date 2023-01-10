#!/usr/bin/python
# Copyright (C) 2019-2021 Jelmer Vernooij <jelmer@jelmer.uk>
# encoding: utf-8
#
# This program is free software; you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation; either version 2 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program; if not, write to the Free Software
# Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA

import logging

import openai


from . import SingleLineMatch


def chatgpt_analyze(lines):
    truncated = ''.join(lines)[-4000:]

    openai_logger = logging.getLogger("openai")
    openai_logger.setLevel(logging.WARNING)

    prompt = (
        "Which line in the log file below is the clearest explanation of a problem:\n\n"
        + truncated)

    response = openai.Completion.create(
        model="text-davinci-003",
        prompt=prompt)

    text = response["choices"][0]["text"].lstrip('\n')
    for i, line in enumerate(lines):
        if line.startswith(text):
            return SingleLineMatch.from_lines(lines, i)
    logging.debug('Unable to find chatgpt answer in lines: %r', text)
    return None


if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument('path', type=str)
    args = parser.parse_args()

    logging.basicConfig(format='%(message)s', level=logging.INFO)

    with open(args.path, 'r', encoding='utf-8') as f:
        match = chatgpt_analyze(f.readlines())
        if match:
            logging.info('match: %s', match)
