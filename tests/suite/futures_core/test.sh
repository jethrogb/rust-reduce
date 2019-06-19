#!/bin/bash
rustc --crate-type lib -o /dev/null "$1" |& exec grep -q 'trait objects without an explicit `dyn` are deprecated'
