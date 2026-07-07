package echozero

import "testing"

func TestSignRestRequestMatchesBackendPayloadFormat(t *testing.T) {
	headers, err := SignRestRequest(
		"test_secret",
		"POST",
		"/api/api-keys",
		map[string]any{"name": "SDK HMAC Test"},
		1_710_000_000_000,
	)
	if err != nil {
		t.Fatal(err)
	}

	const expected = "274c9ff280eadf751530e9e7fce2c2a573d8676b13b7108fe353407df7cc9e00"
	if headers.Signature != expected {
		t.Fatalf("signature mismatch: got %s want %s", headers.Signature, expected)
	}
}

func TestSignInboundWebhookUsesCanonicalJSON(t *testing.T) {
	headers, err := SignInboundWebhook(
		"test_secret",
		map[string]any{
			"text":           "BUY SOL 500 USDC",
			"idempotencyKey": "sdk-test-1",
		},
		1_710_000_000,
	)
	if err != nil {
		t.Fatal(err)
	}

	const expected = "1d30d896fc609e62bcf9be991c1dc1177a9c63219909869ef2846bad4685de3b"
	if headers.Signature != expected {
		t.Fatalf("signature mismatch: got %s want %s", headers.Signature, expected)
	}
}

func TestInboundCanonicalStripsUnknownFields(t *testing.T) {
	canonical, err := InboundWebhookCanonicalJSON(map[string]any{
		"text":       "BUY SOL",
		"extraField": "ignored",
	})
	if err != nil {
		t.Fatal(err)
	}
	if canonical != `{"text":"BUY SOL"}` {
		t.Fatalf("unexpected canonical: %s", canonical)
	}
}

func TestSignInboundWebhookIgnoresUnknownFields(t *testing.T) {
	body := map[string]any{
		"eventType":      "buy",
		"idempotencyKey": "test-1",
		"reasoning":      "test",
		"tokenAddress":   "So11111111111111111111111111111111111111112",
		"amount":         500,
	}
	withExtra := map[string]any{
		"eventType":      "buy",
		"idempotencyKey": "test-1",
		"reasoning":      "test",
		"tokenAddress":   "So11111111111111111111111111111111111111112",
		"amount":         500,
		"unknownField":   "strip me",
	}
	a, err := SignInboundWebhook("secret", body, 1_710_000_000)
	if err != nil {
		t.Fatal(err)
	}
	b, err := SignInboundWebhook("secret", withExtra, 1_710_000_000)
	if err != nil {
		t.Fatal(err)
	}
	if a.Signature != b.Signature {
		t.Fatalf("expected equal signatures, got %s vs %s", a.Signature, b.Signature)
	}
	const expected = "be420f61d91e6b871481774c62f972f0aafa6f5f1a727ba1e4a32558784f77c3"
	if a.Signature != expected {
		t.Fatalf("signature mismatch: got %s want %s", a.Signature, expected)
	}
}

func TestVerifyInboundWebhook(t *testing.T) {
	body := map[string]any{
		"text":           "BUY SOL 500 USDC",
		"idempotencyKey": "sdk-test-1",
	}
	headers, err := SignInboundWebhook("test_secret", body, 1_710_000_000)
	if err != nil {
		t.Fatal(err)
	}

	ok, err := VerifyInboundWebhook("test_secret", body, 1_710_000_000, headers.Signature, 999_999_999)
	if err != nil {
		t.Fatal(err)
	}
	if !ok {
		t.Fatal("expected webhook verification to pass")
	}
}

func TestVerifyOutboundWebhook(t *testing.T) {
	rawBody := `{"event":"signal.execution","signalId":"sig_1","developerAgentId":"agent_1","status":"executed","timestamp":"2026-07-06T12:00:00.000Z"}`
	timestamp := "2026-07-06T12:00:00.000Z"
	signature := hmacSHA256Hex("webhook_secret", timestamp+"."+rawBody)
	if !VerifyOutboundWebhook("webhook_secret", rawBody, timestamp, signature) {
		t.Fatal("expected outbound webhook verification to pass")
	}
}
