#!/bin/sh

cargo build -p backend-impl --release &&\
cd yew-app &&\
trunk clean &&\
trunk build --release &&\
cd .. &&\
cargo build -p backend-artifact --release &&
docker build .