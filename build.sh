#!/bin/sh

cd yew-app &&\
trunk clean &&\
trunk build --release &&\
cd .. &&\
cargo build -p backend-artifact --release &&
docker build .