package engine

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"
)

type EmbeddingService struct {
	baseURL    string
	model      string
	dimension  int
	httpClient *http.Client
}

func NewEmbeddingService(baseURL, model string, dimension int) *EmbeddingService {
	return &EmbeddingService{
		baseURL: baseURL,
		model:   model,
		dimension: dimension,
		httpClient: &http.Client{
			Timeout: 30 * time.Second,
		},
	}
}

type EmbeddingRequest struct {
	Texts []string `json:"texts"`
	Model string   `json:"model,omitempty"`
}

type EmbeddingResponse struct {
	Embeddings [][]float64 `json:"embeddings"`
	Model     string      `json:"model"`
}

func (es *EmbeddingService) GenerateEmbedding(ctx context.Context, text string) ([]float64, error) {
	req := EmbeddingRequest{
		Texts: []string{text},
		Model: es.model,
	}

	reqBody, err := json.Marshal(req)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal request: %w", err)
	}

	httpReq, err := http.NewRequestWithContext(ctx, "POST", es.baseURL+"/embeddings", io.NopCloser(&byteReader{data: reqBody}))
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}
	httpReq.Header.Set("Content-Type", "application/json")

	resp, err := es.httpClient.Do(httpReq)
	if err != nil {
		return nil, fmt.Errorf("failed to send request: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("embedding service returned status %d: %s", resp.StatusCode, string(body))
	}

	var embeddingResp EmbeddingResponse
	if err := json.NewDecoder(resp.Body).Decode(&embeddingResp); err != nil {
		return nil, fmt.Errorf("failed to decode response: %w", err)
	}

	if len(embeddingResp.Embeddings) == 0 {
		return nil, fmt.Errorf("no embeddings returned")
	}

	return embeddingResp.Embeddings[0], nil
}

func (es *EmbeddingService) GenerateBatchEmbeddings(ctx context.Context, texts []string) ([][]float64, error) {
	req := EmbeddingRequest{
		Texts: texts,
		Model: es.model,
	}

	reqBody, err := json.Marshal(req)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal request: %w", err)
	}

	httpReq, err := http.NewRequestWithContext(ctx, "POST", es.baseURL+"/embeddings", io.NopCloser(&byteReader{data: reqBody}))
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}
	httpReq.Header.Set("Content-Type", "application/json")

	resp, err := es.httpClient.Do(httpReq)
	if err != nil {
		return nil, fmt.Errorf("failed to send request: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("embedding service returned status %d: %s", resp.StatusCode, string(body))
	}

	var embeddingResp EmbeddingResponse
	if err := json.NewDecoder(resp.Body).Decode(&embeddingResp); err != nil {
		return nil, fmt.Errorf("failed to decode response: %w", err)
	}

	return embeddingResp.Embeddings, nil
}

func (es *EmbeddingService) GetDimension() int {
	return es.dimension
}

type byteReader struct {
	data []byte
	pos  int
}

func (r *byteReader) Read(p []byte) (n int, err error) {
	if r.pos >= len(r.data) {
		return 0, io.EOF
	}
	n = copy(p, r.data[r.pos:])
	r.pos += n
	return n, nil
}
