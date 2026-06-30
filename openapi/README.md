# OpenAPI Generation

Task 7.3 calls for SDKs to be generated from OpenAPI where possible.

The source of truth for MCP REST endpoints is the deployed Swagger/OpenAPI document:

```bash
curl -o echozero-mcp.openapi.json https://mcp.echozero.app/api/docs-json
```

Recommended publishing workflow:

1. Fetch the latest OpenAPI document from the target environment.
2. Generate endpoint-specific clients into language-specific `generated/` folders.
3. Keep shared hand-written helpers for:
   - Authentication headers.
   - HMAC request signing.
   - Inbound webhook signing/verification.
   - WebSocket signal clients.
4. Review generated diffs before publishing a new SDK version.

The placeholder Dev Portal `openapi.yaml` is not used for package generation because it is explicitly a preview stub.
