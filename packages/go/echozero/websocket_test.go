package echozero

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/gorilla/websocket"
)

// fakeGateway is a minimal Socket.IO v4 stand-in for echozero-be TradeSignalGateway.
func fakeGateway(t *testing.T) *httptest.Server {
	upgrader := websocket.Upgrader{}
	return httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/socket.io/" || r.URL.Query().Get("EIO") != "4" {
			http.NotFound(w, r)
			return
		}
		conn, err := upgrader.Upgrade(w, r, nil)
		if err != nil {
			return
		}
		defer conn.Close()
		send := func(packet string) { conn.WriteMessage(websocket.TextMessage, []byte(packet)) }
		send(`0{"sid":"engine","upgrades":[],"pingInterval":25000,"pingTimeout":20000}`)
		for {
			_, message, err := conn.ReadMessage()
			if err != nil {
				return
			}
			packet := string(message)
			switch {
			case strings.HasPrefix(packet, "40/ws/signals,"):
				var auth map[string]string
				json.Unmarshal([]byte(strings.TrimPrefix(packet, "40/ws/signals,")), &auth)
				send(`40/ws/signals,{"sid":"ns"}`)
				send("2")
				if auth["apiKey"] != "ez_live_good" {
					send(`42/ws/signals,["error",{"message":"Invalid or expired API key"}]`)
					send("41/ws/signals,")
					return
				}
				send(`42/ws/signals,["authenticated",{"ok":true}]`)
			case strings.HasPrefix(packet, "42/ws/signals,"):
				var raw []json.RawMessage
				json.Unmarshal([]byte(strings.TrimPrefix(packet, "42/ws/signals,")), &raw)
				var data map[string]any
				json.Unmarshal(raw[1], &data)
				if data["developerAgentId"] != "agent-1" {
					send(`42/ws/signals,["signal:error",{"message":"Missing developerAgentId"}]`)
					continue
				}
				send(`42/ws/signals,["signal:received",{"signalId":"sig-` + data["idempotencyKey"].(string) + `","status":"accepted"}]`)
			}
		}
	}))
}

func TestSignalClientConnectSendReceive(t *testing.T) {
	server := fakeGateway(t)
	defer server.Close()

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	client := NewSignalClient("ez_live_good").WithBaseURL(server.URL)
	if err := client.Connect(ctx); err != nil {
		t.Fatalf("connect: %v", err)
	}
	defer client.Close()

	if err := client.SendSignal("agent-1", map[string]any{"eventType": "buy", "idempotencyKey": "k1"}); err != nil {
		t.Fatalf("send: %v", err)
	}
	select {
	case event := <-client.Events():
		if event.Name != "signal:received" || !strings.Contains(string(event.Data), `"signalId":"sig-k1"`) {
			t.Fatalf("unexpected event %s %s", event.Name, event.Data)
		}
	case <-ctx.Done():
		t.Fatal("timed out waiting for signal:received")
	}
}

func TestSignalClientInvalidKey(t *testing.T) {
	server := fakeGateway(t)
	defer server.Close()

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	err := NewSignalClient("ez_live_bad").WithBaseURL(server.URL).Connect(ctx)
	if err == nil || !strings.Contains(err.Error(), "Invalid or expired API key") {
		t.Fatalf("expected invalid key error, got %v", err)
	}
}

func TestSignalClientSendBeforeConnect(t *testing.T) {
	if err := NewSignalClient("ez_live_good").SendSignal("agent-1", nil); err == nil {
		t.Fatal("expected not connected error")
	}
}
