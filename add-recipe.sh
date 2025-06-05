
#!/bin/sh
PW=`cat secrets/password.txt`
CREDS="{
  \"email\": \"your@email.com\",
  \"full_name\": \"Some name\",
  \"password\": \"$PW\"
}"

ACCESS_TOKEN=`curl -s -X POST -H "Content-type: application/json" \
     -d "$CREDS" \
     http://localhost:3000/api/v1/register | jq .access_token | sed 's/"//g'`

JOKE='{
  "id": 40,
  "title": "Sugar milk",
  "category": "drink",
  "ingredient_amount": [
    "100ml milk", "20g sugar"
  ],
  "preparation": "put the sugar in the milk and stir"
}'

curl -X POST -H "Content-type: application/json"  \
     -H "Authorization: Bearer $ACCESS_TOKEN" \
     -d "$JOKE" http://localhost:3000/api/v1/add-recipe
