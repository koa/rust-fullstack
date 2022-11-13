#!/bin/sh

cargo build -p backend-impl $* &&\
cd yew-app &&\
trunk clean &&\
trunk build $* &&\
cd .. &&\
cargo build -p backend-artifact $*