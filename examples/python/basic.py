import os

from echozero import EchoZeroClient, sign_inbound_webhook


client = EchoZeroClient(
    base_url=os.getenv("ECHOZERO_BASE_URL", "https://mcp.echozero.app"),
    api_key=os.getenv("ECHOZERO_API_KEY"),
    bearer_token=os.getenv("ECHOZERO_OAUTH_TOKEN"),
)

metadata = client.get("/public/index.json")
print("Connected to:", metadata.get("name", "EchoZero"))

if os.getenv("ECHOZERO_API_KEY") or os.getenv("ECHOZERO_OAUTH_TOKEN"):
    api_keys = client.get("/api/api-keys")
    print("Authenticated request OK:", "list" if isinstance(api_keys, list) else type(api_keys).__name__)

webhook_secret = os.getenv("ECHOZERO_WEBHOOK_SECRET")
if webhook_secret:
    body = {"text": "SDK health check message with no market instruction"}
    headers = sign_inbound_webhook(signing_secret=webhook_secret, body=body)
    print("Webhook signature generated:", bool(headers["X-EZ-Signature"]))
