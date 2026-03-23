package model

import (
	"fmt"
	"strings"
)

// Validate implements ValidatableResponse for SearchResponse
func (r *SearchResponse) Validate() error {
	if r.Total < 0 {
		return fmt.Errorf("total cannot be negative: %d", r.Total)
	}

	if r.Page < 1 {
		return fmt.Errorf("page must be positive: %d", r.Page)
	}

	if r.PageSize < 1 || r.PageSize > 1000 {
		return fmt.Errorf("page_size must be between 1 and 1000: %d", r.PageSize)
	}

	if r.TotalPages < 0 {
		return fmt.Errorf("total_pages cannot be negative: %d", r.TotalPages)
	}

	if r.TookMs < 0 {
		return fmt.Errorf("took_ms cannot be negative: %f", r.TookMs)
	}

	// Validate results
	for i, result := range r.Results {
		if err := result.Validate(); err != nil {
			return fmt.Errorf("result[%d]: %w", i, err)
		}
	}

	return nil
}

// Validate implements ValidatableResponse for SearchResult
func (r *SearchResult) Validate() error {
	if r.ID == "" {
		return fmt.Errorf("id cannot be empty")
	}

	if r.Score < 0 {
		return fmt.Errorf("score cannot be negative: %f", r.Score)
	}

	// Fields can be empty but should not be nil
	if r.Fields == nil {
		r.Fields = make(map[string]string)
	}

	// Highlights can be nil (omitempty)
	if r.Highlights != nil {
		for key, value := range r.Highlights {
			if strings.TrimSpace(key) == "" {
				return fmt.Errorf("highlight key cannot be empty")
			}
			if strings.TrimSpace(value) == "" {
				return fmt.Errorf("highlight value cannot be empty for key: %s", key)
			}
		}
	}

	return nil
}

// Validate implements ValidatableResponse for AddDocumentResponse
func (r *AddDocumentResponse) Validate() error {
	if r.ID == "" {
		return fmt.Errorf("id cannot be empty")
	}

	// Message can be empty but should be consistent with Success
	if r.Success && r.Message != "" && !strings.Contains(strings.ToLower(r.Message), "success") {
		return fmt.Errorf("success response should not contain error message: %s", r.Message)
	}

	if !r.Success && r.Message == "" {
		return fmt.Errorf("error response should contain message")
	}

	return nil
}

// Validate implements ValidatableResponse for DocumentResponse
func (r *DocumentResponse) Validate() error {
	if r.ID == "" {
		return fmt.Errorf("id cannot be empty")
	}

	// Fields can be empty but should not be nil
	if r.Fields == nil {
		r.Fields = make(map[string]string)
	}

	// Score is optional but should not be negative if provided
	if r.Score < 0 {
		return fmt.Errorf("score cannot be negative: %f", r.Score)
	}

	return nil
}

// Validate implements ValidatableResponse for UpdateDocumentResponse
func (r *UpdateDocumentResponse) Validate() error {
	// Message can be empty but should be consistent with Success
	if r.Success && r.Message != "" && !strings.Contains(strings.ToLower(r.Message), "success") {
		return fmt.Errorf("success response should not contain error message: %s", r.Message)
	}

	if !r.Success && r.Message == "" {
		return fmt.Errorf("error response should contain message")
	}

	return nil
}

// Validate implements ValidatableResponse for DeleteDocumentResponse
func (r *DeleteDocumentResponse) Validate() error {
	// Message can be empty but should be consistent with Success
	if r.Success && r.Message != "" && !strings.Contains(strings.ToLower(r.Message), "success") {
		return fmt.Errorf("success response should not contain error message: %s", r.Message)
	}

	if !r.Success && r.Message == "" {
		return fmt.Errorf("error response should contain message")
	}

	return nil
}

// Validate implements ValidatableResponse for BatchDocumentsResponse
func (r *BatchDocumentsResponse) Validate() error {
	if r.SuccessCount < 0 {
		return fmt.Errorf("success_count cannot be negative: %d", r.SuccessCount)
	}

	if r.FailureCount < 0 {
		return fmt.Errorf("failure_count cannot be negative: %d", r.FailureCount)
	}

	// Errors array should have length equal to failure_count if failures exist
	if r.FailureCount > 0 && len(r.Errors) == 0 {
		return fmt.Errorf("errors should not be empty when failure_count > 0")
	}

	if r.FailureCount == 0 && len(r.Errors) > 0 {
		return fmt.Errorf("errors should be empty when failure_count == 0")
	}

	return nil
}

// Validate implements ValidatableResponse for CreateIndexResponse
func (r *CreateIndexResponse) Validate() error {
	if r.ID == "" {
		return fmt.Errorf("id cannot be empty")
	}

	// Message can be empty but should be consistent with Success
	if r.Success && r.Message != "" && !strings.Contains(strings.ToLower(r.Message), "success") {
		return fmt.Errorf("success response should not contain error message: %s", r.Message)
	}

	if !r.Success && r.Message == "" {
		return fmt.Errorf("error response should contain message")
	}

	return nil
}

// Validate implements ValidatableResponse for ListIndexesResponse
func (r *ListIndexesResponse) Validate() error {
	if r.Total < 0 {
		return fmt.Errorf("total cannot be negative: %d", r.Total)
	}

	// Validate each index info
	for i, index := range r.Indexes {
		if err := index.Validate(); err != nil {
			return fmt.Errorf("indexes[%d]: %w", i, err)
		}
	}

	return nil
}

// Validate implements ValidatableResponse for IndexInfo
func (r *IndexInfo) Validate() error {
	if r.ID == "" {
		return fmt.Errorf("id cannot be empty")
	}

	if r.Name == "" {
		return fmt.Errorf("name cannot be empty")
	}

	if r.IndexType == "" {
		return fmt.Errorf("index_type cannot be empty")
	}

	if r.DocumentCount < 0 {
		return fmt.Errorf("document_count cannot be negative: %d", r.DocumentCount)
	}

	if r.SizeBytes < 0 {
		return fmt.Errorf("size_bytes cannot be negative: %d", r.SizeBytes)
	}

	return nil
}

// Validate implements ValidatableResponse for DeleteIndexResponse
func (r *DeleteIndexResponse) Validate() error {
	// Message can be empty but should be consistent with Success
	if r.Success && r.Message != "" && !strings.Contains(strings.ToLower(r.Message), "success") {
		return fmt.Errorf("success response should not contain error message: %s", r.Message)
	}

	if !r.Success && r.Message == "" {
		return fmt.Errorf("error response should contain message")
	}

	return nil
}

// Validate implements ValidatableResponse for RebuildIndexResponse
func (r *RebuildIndexResponse) Validate() error {
	// Message can be empty but should be consistent with Success
	if r.Success && r.Message != "" && !strings.Contains(strings.ToLower(r.Message), "success") {
		return fmt.Errorf("success response should not contain error message: %s", r.Message)
	}

	if !r.Success && r.Message == "" {
		return fmt.Errorf("error response should contain message")
	}

	// TaskID can be empty for synchronous operations
	if r.TaskID != "" && len(r.TaskID) < 8 {
		return fmt.Errorf("task_id should be at least 8 characters if provided: %s", r.TaskID)
	}

	return nil
}
