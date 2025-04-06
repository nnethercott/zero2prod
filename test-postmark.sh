curl "https://api.postmarkapp.com/email" \
  -X POST \
  -v \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
  -H "X-Postmark-Server-Token: secret-token" \
  -d '{
  "From": "nathaniel.nethercott@deepomatic.com",
  "To": "nathaniel.nethercott@deepomatic.com",
  "Subject": "Postmark test",
  "TextBody": "Hello dear Postmark user.",
  "HtmlBody": "<html><body><strong>Hello</strong> dear Postmark user.</body></html>",
  "MessageStream": "outbound"
}'
