#!/bin/bash
#
# This bash script is meant to be used with perftree for debugging the move
# generator functionality (https://github.com/agausmann/perftree)

if [ ! -z $3 ]
then
	cargo run -q -p chrs-perft -- "$1" "$2" "$3"
else
	cargo run -q -p chrs-perft -- "$1" "$2"
fi
