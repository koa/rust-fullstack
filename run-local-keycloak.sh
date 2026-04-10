#!/usr/bin/env bash
cd "$(dirname "$0")" || exit 1
(sleep 30s; ./add-test-user.sh)&
podman run -p 8082:8080 --rm -e KEYCLOAK_ADMIN=admin -e KEYCLOAK_ADMIN_PASSWORD=admin -e KC_HTTP_HOST=:: -e JAVA_OPTS_APPEND="-Djava.net.preferIPv4Stack=false -Djava.net.preferIPv6Addresses=true" -e KC_HOSTNAME_STRICT_HTTPS=false -v "$(pwd)/keycloak-realm.json:/opt/keycloak/data/import/example-realm.json" quay.io/keycloak/keycloak:20.0.1 start-dev --import-realm
