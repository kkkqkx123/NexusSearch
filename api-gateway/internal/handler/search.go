package handler

import (
	"net/http"
	"strconv"

	"github.com/flexsearch/api-gateway/internal/client"
	"github.com/flexsearch/api-gateway/internal/model"
	"github.com/flexsearch/api-gateway/internal/util"
	pb "github.com/flexsearch/api-gateway/proto"
	"github.com/gin-gonic/gin"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/trace"
	"go.uber.org/zap"
)

type SearchHandler struct {
	client  *client.CoordinatorClient
	metrics *util.Metrics
	logger  *zap.Logger
	tracer  trace.Tracer
}

func NewSearchHandler(client *client.CoordinatorClient, metrics *util.Metrics, logger *zap.Logger) *SearchHandler {
	return &SearchHandler{
		client:  client,
		metrics: metrics,
		logger:  logger,
		tracer:  otel.Tracer("search-handler"),
	}
}

func (h *SearchHandler) Search(c *gin.Context) {
	ctx := c.Request.Context()
	ctx, span := h.tracer.Start(ctx, "SearchHandler.Search")
	defer span.End()

	var req model.SearchRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		h.logger.Error("Failed to parse search request",
			zap.Error(err))
		c.JSON(http.StatusBadRequest, model.ErrorResponse{
			Code:    "INVALID_REQUEST",
			Message: err.Error(),
		})
		return
	}

	span.SetAttributes(
		attribute.String("query", req.Query),
		attribute.Int("page", req.Page),
		attribute.Int("page_size", req.PageSize),
	)

	grpcReq := &pb.SearchRequest{
		Query:     req.Query,
		Indexes:   req.Indexes,
		Page:      int32(req.Page),
		PageSize:  int32(req.PageSize),
		Filters:   req.Filters,
		Fields:    req.Fields,
		Highlight: req.Highlight,
		SortBy:    req.SortBy,
		SortOrder: req.SortOrder,
		Explain:   req.Explain,
	}

	h.metrics.IncrementCounter("search_requests_total", []string{"endpoint:search"})

	resp, err := h.client.Search(ctx, grpcReq)
	if err != nil {
		h.logger.Error("Search failed",
			zap.Error(err),
			zap.String("query", req.Query))
		h.metrics.IncrementCounter("search_errors_total", []string{"error_type:grpc"})
		grpcErr := util.ConvertGRPCError(err)
		c.JSON(grpcErr.HTTPStatus, model.ErrorResponse{
			Code:    "SEARCH_FAILED",
			Message: grpcErr.Message,
			Details: grpcErr.Details,
		})
		return
	}

	results := make([]model.SearchResult, len(resp.Results))
	for i, r := range resp.Results {
		results[i] = model.SearchResult{
			ID:         r.Id,
			Score:      r.Score,
			Fields:     r.Fields,
			Highlights: r.Highlights,
		}
	}

	h.metrics.IncrementCounter("search_success_total", []string{})
	h.metrics.RecordHistogram("search_latency_seconds", float64(resp.TookMs)/1000, []string{})

	searchResponse := model.SearchResponse{
		Results:    results,
		Total:      int(resp.Total),
		Page:       int(resp.Page),
		PageSize:   int(resp.PageSize),
		TotalPages: int(resp.TotalPages),
		TookMs:     resp.TookMs,
	}

	// Validate response before sending
	if err := searchResponse.Validate(); err != nil {
		h.logger.Error("Search response validation failed",
			zap.Error(err),
			zap.String("query", req.Query))
		c.JSON(http.StatusInternalServerError, model.ErrorResponse{
			Code:    "RESPONSE_VALIDATION_FAILED",
			Message: "Internal server error",
			Details: err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, searchResponse)
}

func (h *SearchHandler) SearchGet(c *gin.Context) {
	ctx := c.Request.Context()
	ctx, span := h.tracer.Start(ctx, "SearchHandler.SearchGet")
	defer span.End()

	query := c.Query("query")
	indexes := c.QueryArray("index")
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "10"))

	span.SetAttributes(
		attribute.String("query", query),
		attribute.Int("page", page),
		attribute.Int("page_size", pageSize),
	)

	grpcReq := &pb.SearchRequest{
		Query:     query,
		Indexes:   indexes,
		Page:      int32(page),
		PageSize:  int32(pageSize),
		Highlight: c.Query("highlight") == "true",
	}

	resp, err := h.client.Search(ctx, grpcReq)
	if err != nil {
		h.logger.Error("Search failed",
			zap.Error(err),
			zap.String("query", query))
		grpcErr := util.ConvertGRPCError(err)
		c.JSON(grpcErr.HTTPStatus, model.ErrorResponse{
			Code:    "SEARCH_FAILED",
			Message: grpcErr.Message,
			Details: grpcErr.Details,
		})
		return
	}

	results := make([]model.SearchResult, len(resp.Results))
	for i, r := range resp.Results {
		results[i] = model.SearchResult{
			ID:         r.Id,
			Score:      r.Score,
			Fields:     r.Fields,
			Highlights: r.Highlights,
		}
	}

	searchResponse := model.SearchResponse{
		Results:    results,
		Total:      int(resp.Total),
		Page:       int(resp.Page),
		PageSize:   int(resp.PageSize),
		TotalPages: int(resp.TotalPages),
		TookMs:     resp.TookMs,
	}

	// Validate response before sending
	if err := searchResponse.Validate(); err != nil {
		h.logger.Error("Search response validation failed",
			zap.Error(err),
			zap.String("query", query))
		c.JSON(http.StatusInternalServerError, model.ErrorResponse{
			Code:    "RESPONSE_VALIDATION_FAILED",
			Message: "Internal server error",
			Details: err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, searchResponse)
}

type DocumentHandler struct {
	client  *client.CoordinatorClient
	metrics *util.Metrics
	logger  *zap.Logger
	tracer  trace.Tracer
}

func NewDocumentHandler(client *client.CoordinatorClient, metrics *util.Metrics, logger *zap.Logger) *DocumentHandler {
	return &DocumentHandler{
		client:  client,
		metrics: metrics,
		logger:  logger,
		tracer:  otel.Tracer("document-handler"),
	}
}

func (h *DocumentHandler) Create(c *gin.Context) {
	ctx := c.Request.Context()
	ctx, span := h.tracer.Start(ctx, "DocumentHandler.Create")
	defer span.End()

	var req model.AddDocumentRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		h.logger.Error("Failed to parse document request",
			zap.Error(err))
		c.JSON(http.StatusBadRequest, model.ErrorResponse{
			Code:    "INVALID_REQUEST",
			Message: err.Error(),
		})
		return
	}

	span.SetAttributes(attribute.String("index_id", req.IndexID))

	grpcReq := &pb.AddDocumentRequest{
		IndexId: req.IndexID,
		Fields:  req.Fields,
	}

	h.metrics.IncrementCounter("document_requests_total", []string{"operation:create"})

	resp, err := h.client.AddDocument(ctx, grpcReq)
	if err != nil {
		h.logger.Error("Add document failed",
			zap.Error(err),
			zap.String("index_id", req.IndexID))
		h.metrics.IncrementCounter("document_errors_total", []string{"operation:create"})
		c.JSON(http.StatusInternalServerError, model.ErrorResponse{
			Code:    "ADD_DOCUMENT_FAILED",
			Message: err.Error(),
		})
		return
	}

	h.metrics.IncrementCounter("document_success_total", []string{"operation:create"})

	c.JSON(http.StatusCreated, model.AddDocumentResponse{
		ID:      resp.Id,
		Success: resp.Success,
		Message: resp.Message,
	})
}

func (h *DocumentHandler) Get(c *gin.Context) {
	ctx := c.Request.Context()
	ctx, span := h.tracer.Start(ctx, "DocumentHandler.Get")
	defer span.End()

	indexID := c.Param("index_id")
	documentID := c.Param("id")

	span.SetAttributes(
		attribute.String("index_id", indexID),
		attribute.String("document_id", documentID),
	)

	grpcReq := &pb.GetDocumentRequest{
		IndexId:    indexID,
		DocumentId: documentID,
	}

	h.metrics.IncrementCounter("document_requests_total", []string{"operation:get"})

	resp, err := h.client.GetDocument(ctx, grpcReq)
	if err != nil {
		h.logger.Error("Get document failed",
			zap.Error(err),
			zap.String("index_id", indexID),
			zap.String("document_id", documentID))
		h.metrics.IncrementCounter("document_errors_total", []string{"operation:get"})
		c.JSON(http.StatusInternalServerError, model.ErrorResponse{
			Code:    "GET_DOCUMENT_FAILED",
			Message: err.Error(),
		})
		return
	}

	h.metrics.IncrementCounter("document_success_total", []string{"operation:get"})

	c.JSON(http.StatusOK, model.DocumentResponse{
		ID:     resp.Id,
		Fields: resp.Fields,
		Score:  resp.Score,
	})
}

func (h *DocumentHandler) Update(c *gin.Context) {
	ctx := c.Request.Context()
	ctx, span := h.tracer.Start(ctx, "DocumentHandler.Update")
	defer span.End()

	indexID := c.Param("index_id")
	documentID := c.Param("id")

	var req model.UpdateDocumentRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		h.logger.Error("Failed to parse update request",
			zap.Error(err))
		c.JSON(http.StatusBadRequest, model.ErrorResponse{
			Code:    "INVALID_REQUEST",
			Message: err.Error(),
		})
		return
	}

	span.SetAttributes(
		attribute.String("index_id", indexID),
		attribute.String("document_id", documentID),
	)

	grpcReq := &pb.UpdateDocumentRequest{
		IndexId:    indexID,
		DocumentId: documentID,
		Fields:     req.Fields,
	}

	h.metrics.IncrementCounter("document_requests_total", []string{"operation:update"})

	resp, err := h.client.UpdateDocument(ctx, grpcReq)
	if err != nil {
		h.logger.Error("Update document failed",
			zap.Error(err),
			zap.String("index_id", indexID),
			zap.String("document_id", documentID))
		h.metrics.IncrementCounter("document_errors_total", []string{"operation:update"})
		c.JSON(http.StatusInternalServerError, model.ErrorResponse{
			Code:    "UPDATE_DOCUMENT_FAILED",
			Message: err.Error(),
		})
		return
	}

	h.metrics.IncrementCounter("document_success_total", []string{"operation:update"})

	c.JSON(http.StatusOK, model.UpdateDocumentResponse{
		Success: resp.Success,
		Message: resp.Message,
	})
}

func (h *DocumentHandler) Delete(c *gin.Context) {
	ctx := c.Request.Context()
	ctx, span := h.tracer.Start(ctx, "DocumentHandler.Delete")
	defer span.End()

	indexID := c.Param("index_id")
	documentID := c.Param("id")

	span.SetAttributes(
		attribute.String("index_id", indexID),
		attribute.String("document_id", documentID),
	)

	grpcReq := &pb.DeleteDocumentRequest{
		IndexId:    indexID,
		DocumentId: documentID,
	}

	h.metrics.IncrementCounter("document_requests_total", []string{"operation:delete"})

	resp, err := h.client.DeleteDocument(ctx, grpcReq)
	if err != nil {
		h.logger.Error("Delete document failed",
			zap.Error(err),
			zap.String("index_id", indexID),
			zap.String("document_id", documentID))
		h.metrics.IncrementCounter("document_errors_total", []string{"operation:delete"})
		c.JSON(http.StatusInternalServerError, model.ErrorResponse{
			Code:    "DELETE_DOCUMENT_FAILED",
			Message: err.Error(),
		})
		return
	}

	h.metrics.IncrementCounter("document_success_total", []string{"operation:delete"})

	c.JSON(http.StatusOK, model.DeleteDocumentResponse{
		Success: resp.Success,
		Message: resp.Message,
	})
}

func (h *DocumentHandler) Batch(c *gin.Context) {
	ctx := c.Request.Context()
	ctx, span := h.tracer.Start(ctx, "DocumentHandler.Batch")
	defer span.End()

	var req model.BatchDocumentsRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		h.logger.Error("Failed to parse batch request",
			zap.Error(err))
		c.JSON(http.StatusBadRequest, model.ErrorResponse{
			Code:    "INVALID_REQUEST",
			Message: err.Error(),
		})
		return
	}

	span.SetAttributes(
		attribute.String("index_id", req.IndexID),
		attribute.Int("batch_size", len(req.Documents)),
	)

	docs := make([]map[string]string, len(req.Documents))
	copy(docs, req.Documents)

	grpcReq := &pb.BatchDocumentsRequest{
		IndexId:   req.IndexID,
		Documents: docs,
		Refresh:   req.Refresh,
	}

	h.metrics.IncrementCounter("document_requests_total", []string{"operation:batch"})

	resp, err := h.client.BatchDocuments(ctx, grpcReq)
	if err != nil {
		h.logger.Error("Batch documents failed",
			zap.Error(err),
			zap.String("index_id", req.IndexID))
		h.metrics.IncrementCounter("document_errors_total", []string{"operation:batch"})
		c.JSON(http.StatusInternalServerError, model.ErrorResponse{
			Code:    "BATCH_DOCUMENTS_FAILED",
			Message: err.Error(),
		})
		return
	}

	h.metrics.IncrementCounter("document_success_total", []string{"operation:batch"})

	c.JSON(http.StatusOK, model.BatchDocumentsResponse{
		SuccessCount: int(resp.SuccessCount),
		FailureCount: int(resp.FailureCount),
		Errors:       resp.Errors,
	})
}

type IndexHandler struct {
	client  *client.CoordinatorClient
	metrics *util.Metrics
	logger  *zap.Logger
	tracer  trace.Tracer
}

func NewIndexHandler(client *client.CoordinatorClient, metrics *util.Metrics, logger *zap.Logger) *IndexHandler {
	return &IndexHandler{
		client:  client,
		metrics: metrics,
		logger:  logger,
		tracer:  otel.Tracer("index-handler"),
	}
}

func (h *IndexHandler) Create(c *gin.Context) {
	ctx := c.Request.Context()
	ctx, span := h.tracer.Start(ctx, "IndexHandler.Create")
	defer span.End()

	var req model.CreateIndexRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		h.logger.Error("Failed to parse create index request",
			zap.Error(err))
		c.JSON(http.StatusBadRequest, model.ErrorResponse{
			Code:    "INVALID_REQUEST",
			Message: err.Error(),
		})
		return
	}

	span.SetAttributes(
		attribute.String("name", req.Name),
		attribute.String("index_type", req.IndexType),
	)

	grpcReq := &pb.CreateIndexRequest{
		Name:      req.Name,
		IndexType: req.IndexType,
		Fields:    req.Fields,
		Options:   req.Options,
	}

	h.metrics.IncrementCounter("index_requests_total", []string{"operation:create"})

	resp, err := h.client.CreateIndex(ctx, grpcReq)
	if err != nil {
		h.logger.Error("Create index failed",
			zap.Error(err),
			zap.String("name", req.Name))
		h.metrics.IncrementCounter("index_errors_total", []string{"operation:create"})
		c.JSON(http.StatusInternalServerError, model.ErrorResponse{
			Code:    "CREATE_INDEX_FAILED",
			Message: err.Error(),
		})
		return
	}

	h.metrics.IncrementCounter("index_success_total", []string{"operation:create"})

	c.JSON(http.StatusCreated, model.CreateIndexResponse{
		ID:      resp.Id,
		Success: resp.Success,
		Message: resp.Message,
	})
}

func (h *IndexHandler) List(c *gin.Context) {
	ctx := c.Request.Context()
	ctx, span := h.tracer.Start(ctx, "IndexHandler.List")
	defer span.End()

	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "10"))

	span.SetAttributes(
		attribute.Int("page", page),
		attribute.Int("page_size", pageSize),
	)

	grpcReq := &pb.ListIndexesRequest{
		Page:     int32(page),
		PageSize: int32(pageSize),
	}

	h.metrics.IncrementCounter("index_requests_total", []string{"operation:list"})

	resp, err := h.client.ListIndexes(ctx, grpcReq)
	if err != nil {
		h.logger.Error("List indexes failed", zap.Error(err))
		h.metrics.IncrementCounter("index_errors_total", []string{"operation:list"})
		c.JSON(http.StatusInternalServerError, model.ErrorResponse{
			Code:    "LIST_INDEXES_FAILED",
			Message: err.Error(),
		})
		return
	}

	h.metrics.IncrementCounter("index_success_total", []string{"operation:list"})

	indexes := make([]model.IndexInfo, len(resp.Indexes))
	for i, idx := range resp.Indexes {
		indexes[i] = model.IndexInfo{
			ID:            idx.Id,
			Name:          idx.Name,
			IndexType:     idx.IndexType,
			DocumentCount: int(idx.DocumentCount),
			SizeBytes:     int(idx.SizeBytes),
			CreatedAt:     idx.CreatedAt,
			UpdatedAt:     idx.UpdatedAt,
		}
	}

	c.JSON(http.StatusOK, model.ListIndexesResponse{
		Indexes: indexes,
		Total:   int(resp.Total),
	})
}

func (h *IndexHandler) Get(c *gin.Context) {
	ctx := c.Request.Context()
	ctx, span := h.tracer.Start(ctx, "IndexHandler.Get")
	defer span.End()

	indexID := c.Param("id")

	span.SetAttributes(attribute.String("index_id", indexID))

	grpcReq := &pb.GetIndexRequest{
		IndexId: indexID,
	}

	h.metrics.IncrementCounter("index_requests_total", []string{"operation:get"})

	resp, err := h.client.GetIndex(ctx, grpcReq)
	if err != nil {
		h.logger.Error("Get index failed",
			zap.Error(err),
			zap.String("index_id", indexID))
		h.metrics.IncrementCounter("index_errors_total", []string{"operation:get"})
		c.JSON(http.StatusInternalServerError, model.ErrorResponse{
			Code:    "GET_INDEX_FAILED",
			Message: err.Error(),
		})
		return
	}

	h.metrics.IncrementCounter("index_success_total", []string{"operation:get"})

	idx := resp.Index
	c.JSON(http.StatusOK, model.IndexInfo{
		ID:            idx.Id,
		Name:          idx.Name,
		IndexType:     idx.IndexType,
		DocumentCount: int(idx.DocumentCount),
		SizeBytes:     int(idx.SizeBytes),
		CreatedAt:     idx.CreatedAt,
		UpdatedAt:     idx.UpdatedAt,
	})
}

func (h *IndexHandler) Delete(c *gin.Context) {
	ctx := c.Request.Context()
	ctx, span := h.tracer.Start(ctx, "IndexHandler.Delete")
	defer span.End()

	indexID := c.Param("id")

	span.SetAttributes(attribute.String("index_id", indexID))

	grpcReq := &pb.DeleteIndexRequest{
		IndexId: indexID,
	}

	h.metrics.IncrementCounter("index_requests_total", []string{"operation:delete"})

	resp, err := h.client.DeleteIndex(ctx, grpcReq)
	if err != nil {
		h.logger.Error("Delete index failed",
			zap.Error(err),
			zap.String("index_id", indexID))
		h.metrics.IncrementCounter("index_errors_total", []string{"operation:delete"})
		c.JSON(http.StatusInternalServerError, model.ErrorResponse{
			Code:    "DELETE_INDEX_FAILED",
			Message: err.Error(),
		})
		return
	}

	h.metrics.IncrementCounter("index_success_total", []string{"operation:delete"})

	c.JSON(http.StatusOK, model.DeleteIndexResponse{
		Success: resp.Success,
		Message: resp.Message,
	})
}

func (h *IndexHandler) Rebuild(c *gin.Context) {
	ctx := c.Request.Context()
	ctx, span := h.tracer.Start(ctx, "IndexHandler.Rebuild")
	defer span.End()

	indexID := c.Param("id")
	async := c.Query("async") == "true"

	span.SetAttributes(
		attribute.String("index_id", indexID),
		attribute.Bool("async", async),
	)

	grpcReq := &pb.RebuildIndexRequest{
		IndexId: indexID,
		Async:   async,
	}

	h.metrics.IncrementCounter("index_requests_total", []string{"operation:rebuild"})

	resp, err := h.client.RebuildIndex(ctx, grpcReq)
	if err != nil {
		h.logger.Error("Rebuild index failed",
			zap.Error(err),
			zap.String("index_id", indexID))
		h.metrics.IncrementCounter("index_errors_total", []string{"operation:rebuild"})
		c.JSON(http.StatusInternalServerError, model.ErrorResponse{
			Code:    "REBUILD_INDEX_FAILED",
			Message: err.Error(),
		})
		return
	}

	h.metrics.IncrementCounter("index_success_total", []string{"operation:rebuild"})

	c.JSON(http.StatusOK, model.RebuildIndexResponse{
		Success: resp.Success,
		Message: resp.Message,
		TaskID:  resp.TaskId,
	})
}
