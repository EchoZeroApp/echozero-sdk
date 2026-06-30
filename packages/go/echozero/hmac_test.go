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
