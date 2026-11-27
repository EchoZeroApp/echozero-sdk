package echozero

import (
	"context"
	"encoding/json"
	"errors"
	"io"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestSetExecutionCallbackPutsURL(t *testing.T) {
	var gotMethod, gotPath, gotKey, gotSig string
	var gotBody map[string]string
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		gotMethod, gotPath, gotKey, gotSig = r.Method, r.URL.EscapedPath(), r.Header.Get("x-api-key"), r.Header.Get("x-signature")
		raw, _ := io.ReadAll(r.Body)
		_ = json.Unmarshal(raw, &gotBody)
		w.Write([]byte(`{"success":true,"data":{"id":"a1"}}`))
	}))
	defer server.Close()

	var agent map[string]any
	client := NewClient(server.URL).WithAPIKey("k")
	if err := client.SetExecutionCallback(context.Background(), "a1", "https://hooks.example.com/ez", &agent); err != nil {
		t.Fatal(err)
	}
	if gotMethod != http.MethodPut || gotPath != "/api/v1/agents/a1/execution-callback" {
		t.Fatalf("unexpected request %s %s", gotMethod, gotPath)
	}
	if gotBody["url"] != "https://hooks.example.com/ez" || gotKey != "k" || gotSig != "" {
		t.Fatalf("unexpected body/headers %v %q %q", gotBody, gotKey, gotSig)
	}
	if agent["id"] != "a1" {
		t.Fatalf("unexpected agent %v", agent)
	}
}

func TestExecutionCallbackSignsWhenSecretSet(t *testing.T) {
	var sig, ts string
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		sig, ts = r.Header.Get("x-signature"), r.Header.Get("x-timestamp")
		w.Write([]byte(`{"success":true,"data":{}}`))
	}))
	defer server.Close()
	client := NewClient(server.URL).WithAPIKey("k").WithHMACSecretKey("s")
	if err := client.SetExecutionCallback(context.Background(), "a1", "https://hooks.example.com/ez", nil); err != nil {
		t.Fatal(err)
	}
	if len(sig) != 64 || ts == "" {
		t.Fatalf("expected signed request, got sig=%q ts=%q", sig, ts)
	}
}

func TestClearExecutionCallbackSendsDelete(t *testing.T) {
	var gotMethod, gotPath string
	var gotLen int64
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		gotMethod, gotPath, gotLen = r.Method, r.URL.EscapedPath(), r.ContentLength
		w.Write([]byte(`{"success":true,"data":{}}`))
	}))
	defer server.Close()
	client := NewClient(server.URL).WithAPIKey("k")
	if err := client.ClearExecutionCallback(context.Background(), "a/1", nil); err != nil {
		t.Fatal(err)
	}
	if gotMethod != http.MethodDelete || gotPath != "/api/v1/agents/a%2F1/execution-callback" || gotLen > 0 {
		t.Fatalf("unexpected request %s %s len=%d", gotMethod, gotPath, gotLen)
	}
}

func TestSetExecutionCallbackRejectedURL(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(400)
		w.Write([]byte(`{"success":false,"error":{"code":"BAD_REQUEST","message":"Callback URL must use https"}}`))
	}))
	defer server.Close()
	client := NewClient(server.URL).WithAPIKey("k")
	err := client.SetExecutionCallback(context.Background(), "a1", "http://hooks.example.com/ez", nil)
	var apiErr *APIError
	if !errors.As(err, &apiErr) || apiErr.Status != 400 || apiErr.Message != "Callback URL must use https" {
		t.Fatalf("unexpected error %v", err)
	}
}
