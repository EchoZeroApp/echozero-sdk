package echozero

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"net/url"
	"strings"
	"sync"
	"time"

	"github.com/gorilla/websocket"
)

// SignalNamespace is the Socket.IO namespace of the developer signal gateway.
const SignalNamespace = "/ws/signals"

// SignalEvent is a server event from the signal gateway, e.g.
// "signal:received" or "signal:error". Data is the raw JSON payload.
// A final event named "disconnect" is delivered when the connection ends.
type SignalEvent struct {
	Name string
	Data json.RawMessage
}

// SignalClient speaks the Socket.IO v4 protocol (WebSocket transport) to the
// EchoZero signal gateway.
//
// The WebSocket gateway accepts structured envelopes (eventType) and the
// legacy {action, tokenAddress, amount} shape. Natural-language "text"
// signals are only accepted by the HTTP inbound webhook.
type SignalClient struct {
	BaseURL string
	APIKey  string
	Dialer  *websocket.Dialer

	conn    *websocket.Conn
	writeMu sync.Mutex
	events  chan SignalEvent
}

func NewSignalClient(apiKey string) *SignalClient {
	return &SignalClient{
		BaseURL: "https://mcp.echozero.app",
		APIKey:  apiKey,
		Dialer:  websocket.DefaultDialer,
	}
}

func (client *SignalClient) WithBaseURL(baseURL string) *SignalClient {
	client.BaseURL = baseURL
	return client
}

// Connect dials the gateway and blocks until it emits "authenticated".
func (client *SignalClient) Connect(ctx context.Context) error {
	endpoint, err := socketIOURL(client.BaseURL)
	if err != nil {
		return err
	}
	dialer := client.Dialer
	if dialer == nil {
		dialer = websocket.DefaultDialer
	}
	headers := http.Header{}
	headers.Set("x-api-key", client.APIKey)
	conn, _, err := dialer.DialContext(ctx, endpoint, headers)
	if err != nil {
		return err
	}

	fail := func(err error) error {
		conn.Close()
		return err
	}
	if deadline, ok := ctx.Deadline(); ok {
		conn.SetReadDeadline(deadline)
	}

	// Engine.IO open packet: 0{"sid":...}
	if _, message, err := conn.ReadMessage(); err != nil {
		return fail(err)
	} else if !strings.HasPrefix(string(message), "0") {
		return fail(fmt.Errorf("echozero signal gateway: unexpected open packet %q", message))
	}

	auth, err := json.Marshal(map[string]string{"apiKey": client.APIKey})
	if err != nil {
		return fail(err)
	}
	client.conn = conn
	if err := client.write("40" + SignalNamespace + "," + string(auth)); err != nil {
		client.conn = nil
		return fail(err)
	}

	for {
		_, message, err := conn.ReadMessage()
		if err != nil {
			client.conn = nil
			return fail(err)
		}
		packet := string(message)
		switch {
		case packet == "2":
			client.write("3")
		case strings.HasPrefix(packet, "44"+SignalNamespace):
			client.conn = nil
			return fail(fmt.Errorf("echozero signal gateway: %s", gatewayMessage(payloadOf(packet, "44"))))
		case strings.HasPrefix(packet, "42"+SignalNamespace):
			event, err := parseEvent(packet)
			if err != nil {
				continue
			}
			switch event.Name {
			case "authenticated":
				conn.SetReadDeadline(time.Time{})
				client.events = make(chan SignalEvent, 64)
				go client.readLoop(conn, client.events)
				return nil
			case "error":
				client.conn = nil
				return fail(fmt.Errorf("echozero signal gateway: %s", gatewayMessage(event.Data)))
			}
		}
	}
}

// Events returns server events received after Connect. The channel is
// closed after the "disconnect" event.
func (client *SignalClient) Events() <-chan SignalEvent {
	return client.events
}

// SendSignal emits a "signal" event for one of your developer agents.
func (client *SignalClient) SendSignal(developerAgentID string, signal map[string]any) error {
	payload := make(map[string]any, len(signal)+1)
	for key, value := range signal {
		payload[key] = value
	}
	payload["developerAgentId"] = developerAgentID
	frame, err := json.Marshal([]any{"signal", payload})
	if err != nil {
		return err
	}
	return client.write("42" + SignalNamespace + "," + string(frame))
}

func (client *SignalClient) Close() error {
	client.writeMu.Lock()
	conn := client.conn
	client.conn = nil
	client.writeMu.Unlock()
	if conn == nil {
		return nil
	}
	_ = conn.WriteMessage(websocket.TextMessage, []byte("41"+SignalNamespace+","))
	return conn.Close()
}

func (client *SignalClient) write(packet string) error {
	client.writeMu.Lock()
	defer client.writeMu.Unlock()
	if client.conn == nil {
		return errors.New("signal websocket is not connected")
	}
	return client.conn.WriteMessage(websocket.TextMessage, []byte(packet))
}

func (client *SignalClient) readLoop(conn *websocket.Conn, events chan SignalEvent) {
	defer close(events)
	for {
		_, message, err := conn.ReadMessage()
		if err != nil {
			events <- SignalEvent{Name: "disconnect"}
			return
		}
		packet := string(message)
		switch {
		case packet == "2":
			client.write("3")
		case strings.HasPrefix(packet, "41"+SignalNamespace):
			events <- SignalEvent{Name: "disconnect"}
			conn.Close()
			return
		case strings.HasPrefix(packet, "42"+SignalNamespace):
			if event, err := parseEvent(packet); err == nil {
				events <- event
			}
		}
	}
}

func socketIOURL(baseURL string) (string, error) {
	parsed, err := url.Parse(strings.TrimRight(baseURL, "/"))
	if err != nil {
		return "", err
	}
	switch parsed.Scheme {
	case "https", "wss":
		parsed.Scheme = "wss"
	case "http", "ws":
		parsed.Scheme = "ws"
	default:
		return "", fmt.Errorf("unsupported base URL scheme %q", parsed.Scheme)
	}
	parsed.Path = "/socket.io/"
	parsed.RawQuery = "EIO=4&transport=websocket"
	return parsed.String(), nil
}

// payloadOf strips "<type>/ws/signals," from a namespaced packet.
func payloadOf(packet, packetType string) string {
	return strings.TrimPrefix(packet, packetType+SignalNamespace+",")
}

func parseEvent(packet string) (SignalEvent, error) {
	var frame []json.RawMessage
	if err := json.Unmarshal([]byte(payloadOf(packet, "42")), &frame); err != nil || len(frame) == 0 {
		return SignalEvent{}, fmt.Errorf("malformed event packet %q", packet)
	}
	var name string
	if err := json.Unmarshal(frame[0], &name); err != nil {
		return SignalEvent{}, err
	}
	event := SignalEvent{Name: name}
	if len(frame) > 1 {
		event.Data = frame[1]
	}
	return event, nil
}

func gatewayMessage[T string | json.RawMessage](data T) string {
	var body struct {
		Message string `json:"message"`
	}
	if json.Unmarshal([]byte(data), &body) == nil && body.Message != "" {
		return body.Message
	}
	return string(data)
}
