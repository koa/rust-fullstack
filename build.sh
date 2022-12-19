#!/bin/sh

cd backend-impl && cargo build --release &&\
cd ../yew-app &&\
trunk clean &&\
trunk build --release &&\
cd ../backend-artifact &&\
cargo build --release &&
cd .. &&
docker build .