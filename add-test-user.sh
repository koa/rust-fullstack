#!/usr/bin/bash

echo "* Request for authorization"
RESULT=`curl --data "username=admin&password=admin&grant_type=password&client_id=admin-cli" http://localhost:8082/realms/master/protocol/openid-connect/token`

echo "\n"
echo "* Recovery of the token"
TOKEN=`echo $RESULT | sed 's/.*access_token":"//g' | sed 's/".*//g'`

echo "\n"
echo " * user creation\n"
curl http://localhost:8082/admin/realms/rust-test/users -H "Content-Type: application/json" -H "Authorization: bearer $TOKEN"   --data '{
  "username": "test",
  "firstName": "Tester",
  "lastName": "User",
  "email": "local@berg-turbenthal.ch",
  "enabled": "true",
  "emailVerified":true,
  "requiredActions":[],
  "groups":[],
  "credentials": [
    {
      "type": "password",
      "value": "123456"
    }
  ]
}'