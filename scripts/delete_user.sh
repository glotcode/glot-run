#!/bin/bash

userId="$1"

curl --request DELETE \
     --header 'Authorization: Token tamed-busman-want-vendetta' \
     --header 'Content-type: application/json' \
     --url "http://localhost:8089/admin/users/${userId}"
