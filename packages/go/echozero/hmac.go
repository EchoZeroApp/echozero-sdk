package echozero

import (
	"bytes"
	"crypto/hmac"
	"crypto/sha256"
	"crypto/subtle"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"sort"
	"strconv"
	"time"
)

type HMACHeaders struct {
	Timestamp string
	Signature string
}

type InboundWebhookHeaders struct {
	Timestamp string
	Signature string
}

func StableJSON(value any) (string, error) {
	var normalized any
	raw, err := json.Marshal(value)
	if err != nil {
		return "", err
	}
	if err := json.Unmarshal(raw, &normalized); err != nil {
		return "", err
	}

	var buf bytes.Buffer
	if err := writeStableJSON(&buf, normalized); err != nil {
		return "", err
	}
	return buf.String(), nil
}

func SignRestRequest(secretKey, method, path string, body any, timestampMs int64) (HMACHeaders, error) {
	if timestampMs == 0 {
		timestampMs = time.Now().UnixMilli()
	}
	timestamp := strconv.FormatInt(timestampMs, 10)

	bodyText := ""
	if body != nil {
		if bodyString, ok := body.(string); ok {
			bodyText = bodyString
		} else {
			stable, err := StableJSON(body)
			if err != nil {
				return HMACHeaders{}, err
			}
			bodyText = stable
		}
	}

	signature := hmacSHA256Hex(secretKey, timestamp+methodUpper(method)+path+bodyText)
	return HMACHeaders{Timestamp: timestamp, Signature: signature}, nil
}

func CanonicalWebhookBody(body map[string]any) (string, error) {
	return StableJSON(body)
}

func SignInboundWebhook(signingSecret string, body map[string]any, timestampSeconds int64) (InboundWebhookHeaders, error) {
	if timestampSeconds == 0 {
		timestampSeconds = time.Now().Unix()
	}
	timestamp := strconv.FormatInt(timestampSeconds, 10)
	canonical, err := CanonicalWebhookBody(body)
	if err != nil {
		return InboundWebhookHeaders{}, err
	}

	signature := hmacSHA256Hex(signingSecret, timestamp+"."+canonical)
	return InboundWebhookHeaders{Timestamp: timestamp, Signature: signature}, nil
}

func VerifyInboundWebhook(signingSecret string, body map[string]any, timestampSeconds int64, signature string, maxSkewSeconds int64) (bool, error) {
	if maxSkewSeconds == 0 {
		maxSkewSeconds = 300
	}
	if absInt64(time.Now().Unix()-timestampSeconds) > maxSkewSeconds {
		return false, nil
	}

	expected, err := SignInboundWebhook(signingSecret, body, timestampSeconds)
	if err != nil {
		return false, err
	}
	return subtle.ConstantTimeCompare([]byte(expected.Signature), []byte(signature)) == 1, nil
}

func hmacSHA256Hex(secret, payload string) string {
	mac := hmac.New(sha256.New, []byte(secret))
	mac.Write([]byte(payload))
	return hex.EncodeToString(mac.Sum(nil))
}

func methodUpper(method string) string {
	bytes := []byte(method)
	for i, b := range bytes {
		if b >= 'a' && b <= 'z' {
			bytes[i] = b - ('a' - 'A')
		}
	}
	return string(bytes)
}

func writeStableJSON(buf *bytes.Buffer, value any) error {
	switch typed := value.(type) {
	case nil:
		buf.WriteString("null")
	case bool:
		if typed {
			buf.WriteString("true")
		} else {
			buf.WriteString("false")
		}
	case string:
		encoded, _ := json.Marshal(typed)
		buf.Write(encoded)
	case float64:
		encoded, _ := json.Marshal(typed)
		buf.Write(encoded)
	case []any:
		buf.WriteByte('[')
		for i, item := range typed {
			if i > 0 {
				buf.WriteByte(',')
			}
			if err := writeStableJSON(buf, item); err != nil {
				return err
			}
		}
		buf.WriteByte(']')
	case map[string]any:
		keys := make([]string, 0, len(typed))
		for key := range typed {
			keys = append(keys, key)
		}
		sort.Strings(keys)

		buf.WriteByte('{')
		for i, key := range keys {
			if i > 0 {
				buf.WriteByte(',')
			}
			encodedKey, _ := json.Marshal(key)
			buf.Write(encodedKey)
			buf.WriteByte(':')
			if err := writeStableJSON(buf, typed[key]); err != nil {
				return err
			}
		}
		buf.WriteByte('}')
	default:
		return fmt.Errorf("unsupported stable json type %T", value)
	}
	return nil
}

func absInt64(value int64) int64 {
	if value < 0 {
		return -value
	}
	return value
}
