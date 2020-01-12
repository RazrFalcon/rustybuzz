#!/usr/bin/env bash

# ignore hb-ucd-table.hh because it's too verbose

# $1 is path to harfbuzz sources
diff --exclude .directory --exclude hb-ucd-table.hh --unified --recursive "$1" harfbuzz/src
