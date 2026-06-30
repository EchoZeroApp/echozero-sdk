package echozero

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strings"
)

type Client struct {
	BaseURL       string
	APIKey        string
	BearerToken   string
	HMACSecretKey string
	HTTPClient    *http.Client
}

type RequestOptions struct {
	Query map[string]string
	Body  any
	HMAC  bool
}

type APIError struct {
	Status  int
	Code    string
	Message string
	Payload any
}

func (err *APIError) Error() string {
	if err.Code != "" {
		return fmt.Sprintf("echozero api error %d %s: %s", err.Status, err.Code, err.Message)
	}
	return fmt.Sprintf("echozero api error %d: %s", err.Status, err.Message)
}

func NewClient(baseURL string) *Client {
	if baseURL == "" {
		baseURL = "https://mcp.echozero.app"
	}
	return &Client{
		BaseURL:    strings.TrimRight(baseURL, "/"),
		HTTPClient: http.DefaultClient,
	}
}

func (client *Client) WithAPIKey(apiKey string) *Client {
	client.APIKey = apiKey
	return client
}

func (client *Client) WithBearerToken(token string) *Client {
	client.BearerToken = token
	return client
}

func (client *Client) WithHMACSecretKey(secretKey string) *Client {
	client.HMACSecretKey = secretKey
	return client
}

func (client *Client) Get(ctx context.Context, path string, out any) error {
	return client.Request(ctx, http.MethodGet, path, RequestOptions{}, out)
}

func (client *Client) Post(ctx context.Context, path string, body any, out any) error {
	return client.Request(ctx, http.MethodPost, path, RequestOptions{Body: body}, out)
}

func (client *Client) Request(ctx context.Context, method, path string, options RequestOptions, out any) error {
	requestURL, pathWithQuery, err := client.buildURL(path, options.Query)
	if err != nil {
		return err
	}

	var requestBody io.Reader
	var bodyBytes []byte
	if options.Body != nil {
		bodyBytes, err = json.Marshal(options.Body)
		if err != nil {
			return err
		}
		requestBody = bytes.NewReader(bodyBytes)
	}

	request, err := http.NewRequestWithContext(ctx, method, requestURL, requestBody)
	if err != nil {
		return err
	}
	request.Header.Set("Accept", "application/json")
	if options.Body != nil {
		request.Header.Set("Content-Type", "application/json")
	}
	if client.BearerToken != "" {
		request.Header.Set("Authorization", "Bearer "+client.BearerToken)
	} else if client.APIKey != "" {
		request.Header.Set("x-api-key", client.APIKey)
	}
	if options.HMAC {
		if client.HMACSecretKey == "" {
			return fmt.Errorf("HMACSecretKey is required when HMAC=true")
		}
		headers, err := SignRestRequest(client.HMACSecretKey, method, pathWithQuery, string(bodyBytes), 0)
		if err != nil {
			return err
		}
		request.Header.Set("x-timestamp", headers.Timestamp)
		request.Header.Set("x-signature", headers.Signature)
	}

	httpClient := client.HTTPClient
	if httpClient == nil {
		httpClient = http.DefaultClient
	}
	response, err := httpClient.Do(request)
	if err != nil {
		return err
	}
	defer response.Body.Close()

	responseBytes, err := io.ReadAll(response.Body)
	if err != nil {
		return err
	}

	var payload any
	if len(responseBytes) > 0 {
		if err := json.Unmarshal(responseBytes, &payload); err != nil {
			payload = string(responseBytes)
		}
	}

	if response.StatusCode < 200 || response.StatusCode >= 300 {
		return apiErrorFromPayload(response.StatusCode, response.Status, payload)
	}

	unwrapped := unwrapEnvelope(payload)
	if out == nil {
		return nil
	}
	outputBytes, err := json.Marshal(unwrapped)
	if err != nil {
		return err
	}
	return json.Unmarshal(outputBytes, out)
}

func (client *Client) buildURL(path string, query map[string]string) (string, string, error) {
	var parsed *url.URL
	var err error
	if strings.HasPrefix(path, "http://") || strings.HasPrefix(path, "https://") {
		parsed, err = url.Parse(path)
	} else {
		parsed, err = url.Parse(client.BaseURL + "/" + strings.TrimLeft(path, "/"))
	}
	if err != nil {
		return "", "", err
	}
	params := parsed.Query()
	for key, value := range query {
		if value != "" {
			params.Set(key, value)
		}
	}
	parsed.RawQuery = params.Encode()
	pathWithQuery := parsed.EscapedPath()
	if parsed.RawQuery != "" {
		pathWithQuery += "?" + parsed.RawQuery
	}
	return parsed.String(), pathWithQuery, nil
}

func unwrapEnvelope(payload any) any {
	record, ok := payload.(map[string]any)
	if !ok {
		return payload
	}
	if success, ok := record["success"].(bool); ok && success {
		if data, ok := record["data"]; ok {
			return data
		}
	}
	return payload
}

func apiErrorFromPayload(status int, statusText string, payload any) error {
	apiError := &APIError{Status: status, Message: statusText, Payload: payload}
	record, ok := payload.(map[string]any)
	if !ok {
		return apiError
	}
	errorRecord, ok := record["error"].(map[string]any)
	if !ok {
		return apiError
	}
	if code, ok := errorRecord["code"].(string); ok {
		apiError.Code = code
	}
	if message, ok := errorRecord["message"].(string); ok {
		apiError.Message = message
	}
	return apiError
}
