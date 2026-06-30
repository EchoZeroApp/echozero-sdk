package echozero

import (
	"context"
	"fmt"
	"net/http"
	"net/url"

	"github.com/gorilla/websocket"
)

type SignalClient struct {
	URL         string
	APIKey      string
	BearerToken string
	Dialer      *websocket.Dialer
	Conn        *websocket.Conn
}

func NewSignalClient(signalURL string) *SignalClient {
	return &SignalClient{
		URL:    signalURL,
		Dialer: websocket.DefaultDialer,
	}
}

func (client *SignalClient) WithAPIKey(apiKey string) *SignalClient {
	client.APIKey = apiKey
	return client
}

func (client *SignalClient) WithBearerToken(token string) *SignalClient {
	client.BearerToken = token
	return client
}

func (client *SignalClient) Connect(ctx context.Context) (*websocket.Conn, *http.Response, error) {
	parsed, err := url.Parse(client.URL)
	if err != nil {
		return nil, nil, err
	}
	query := parsed.Query()
	if client.APIKey != "" {
		query.Set("api_key", client.APIKey)
	}
	if client.BearerToken != "" {
		query.Set("access_token", client.BearerToken)
	}
	parsed.RawQuery = query.Encode()

	dialer := client.Dialer
	if dialer == nil {
		dialer = websocket.DefaultDialer
	}
	conn, response, err := dialer.DialContext(ctx, parsed.String(), nil)
	if err != nil {
		return nil, response, err
	}
	client.Conn = conn
	return conn, response, nil
}

func (client *SignalClient) SendSignal(signal any) error {
	return client.SendJSON(map[string]any{
		"event": "signal",
		"data":  signal,
	})
}

func (client *SignalClient) SendJSON(value any) error {
	if client.Conn == nil {
		return fmt.Errorf("signal websocket is not connected")
	}
	return client.Conn.WriteJSON(value)
}

func (client *SignalClient) Close() error {
	if client.Conn == nil {
		return nil
	}
	err := client.Conn.Close()
	client.Conn = nil
	return err
}
