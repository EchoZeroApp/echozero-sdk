import os

from echozero import EchoZeroClient, sign_inbound_webhook


client = EchoZeroClient(
    base_url=os.getenv("ECHOZERO_BASE_URL", "https://mcp.echozero.app"),
    api_key=os.getenv("ECHOZERO_API_KEY"),
    bearer_token=os.getenv("ECHOZERO_OAUTH_TOKEN"),
)

print(client.get("/api/v1/users/me"))

webhook_secret = os.getenv("ECHOZERO_WEBHOOK_SECRET")
if webhook_secret:
    body = {"text": "BUY SOL 500 USDC"}
    print(sign_inbound_webhook(signing_secret=webhook_secret, body=body))
