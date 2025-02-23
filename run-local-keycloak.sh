#!/usr/bin/env bash
cd "$(dirname "$0")"
(sleep 30s; ./add-test-user.sh)&
podman run -p 8082:8080 --rm -e KEYCLOAK_ADMIN=admin -e KEYCLOAK_ADMIN_PASSWORD=admin -v $(pwd)/keycloak-realm.json:/opt/keycloak/data/import/example-realm.json quay.io/keycloak/keycloak:20.0.1 start-dev --import-realm
