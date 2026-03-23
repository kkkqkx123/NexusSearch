package client

import (
	"context"
	"time"

	"github.com/flexsearch/api-gateway/internal/config"
	pb "github.com/flexsearch/api-gateway/proto"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/trace"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

type CoordinatorClient struct {
	conn     *grpc.ClientConn
	search   pb.SearchServiceClient
	document pb.DocumentServiceClient
	index    pb.IndexServiceClient
	health   pb.HealthClient
	tracer   trace.Tracer
}

func NewCoordinatorClient(cfg *config.CoordinatorConfig) (*CoordinatorClient, error) {
	conn, err := grpc.Dial(cfg.Address,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithBlock(),
		grpc.WithTimeout(time.Duration(cfg.Timeout)*time.Second),
	)
	if err != nil {
		return nil, err
	}

	return &CoordinatorClient{
		conn:     conn,
		search:   pb.NewSearchServiceClient(conn),
		document: pb.NewDocumentServiceClient(conn),
		index:    pb.NewIndexServiceClient(conn),
		health:   pb.NewHealthClient(conn),
		tracer:   otel.Tracer("coordinator-client"),
	}, nil
}

func (c *CoordinatorClient) Close() error {
	return c.conn.Close()
}

func (c *CoordinatorClient) Search(ctx context.Context, req *pb.SearchRequest, opts ...grpc.CallOption) (*pb.SearchResponse, error) {
	ctx, span := c.tracer.Start(ctx, "CoordinatorClient.Search",
		trace.WithAttributes(
			attribute.String("query", req.Query),
			attribute.Int("page", int(req.Page)),
			attribute.Int("page_size", int(req.PageSize)),
		))
	defer span.End()

	resp, err := c.search.Search(ctx, req, opts...)
	if err != nil {
		span.RecordError(err)
		return nil, err
	}

	span.SetAttributes(attribute.Int("result_count", len(resp.Results)))
	return resp, nil
}

func (c *CoordinatorClient) GetDocument(ctx context.Context, req *pb.GetDocumentRequest, opts ...grpc.CallOption) (*pb.DocumentResponse, error) {
	ctx, span := c.tracer.Start(ctx, "CoordinatorClient.GetDocument",
		trace.WithAttributes(
			attribute.String("index_id", req.IndexId),
			attribute.String("document_id", req.DocumentId),
		))
	defer span.End()

	return c.document.GetDocument(ctx, req, opts...)
}

func (c *CoordinatorClient) AddDocument(ctx context.Context, req *pb.AddDocumentRequest, opts ...grpc.CallOption) (*pb.AddDocumentResponse, error) {
	ctx, span := c.tracer.Start(ctx, "CoordinatorClient.AddDocument",
		trace.WithAttributes(
			attribute.String("index_id", req.IndexId),
		))
	defer span.End()

	return c.document.AddDocument(ctx, req, opts...)
}

func (c *CoordinatorClient) UpdateDocument(ctx context.Context, req *pb.UpdateDocumentRequest, opts ...grpc.CallOption) (*pb.UpdateDocumentResponse, error) {
	ctx, span := c.tracer.Start(ctx, "CoordinatorClient.UpdateDocument",
		trace.WithAttributes(
			attribute.String("index_id", req.IndexId),
			attribute.String("document_id", req.DocumentId),
		))
	defer span.End()

	return c.document.UpdateDocument(ctx, req, opts...)
}

func (c *CoordinatorClient) DeleteDocument(ctx context.Context, req *pb.DeleteDocumentRequest, opts ...grpc.CallOption) (*pb.DeleteDocumentResponse, error) {
	ctx, span := c.tracer.Start(ctx, "CoordinatorClient.DeleteDocument",
		trace.WithAttributes(
			attribute.String("index_id", req.IndexId),
			attribute.String("document_id", req.DocumentId),
		))
	defer span.End()

	return c.document.DeleteDocument(ctx, req, opts...)
}

func (c *CoordinatorClient) BatchDocuments(ctx context.Context, req *pb.BatchDocumentsRequest, opts ...grpc.CallOption) (*pb.BatchDocumentsResponse, error) {
	ctx, span := c.tracer.Start(ctx, "CoordinatorClient.BatchDocuments",
		trace.WithAttributes(
			attribute.String("index_id", req.IndexId),
			attribute.Int("batch_size", len(req.Documents)),
		))
	defer span.End()

	resp, err := c.document.BatchDocuments(ctx, req, opts...)
	if err != nil {
		span.RecordError(err)
		return nil, err
	}

	span.SetAttributes(
		attribute.Int("success_count", int(resp.SuccessCount)),
		attribute.Int("failure_count", int(resp.FailureCount)),
	)
	return resp, nil
}

func (c *CoordinatorClient) CreateIndex(ctx context.Context, req *pb.CreateIndexRequest, opts ...grpc.CallOption) (*pb.CreateIndexResponse, error) {
	ctx, span := c.tracer.Start(ctx, "CoordinatorClient.CreateIndex",
		trace.WithAttributes(
			attribute.String("name", req.Name),
			attribute.String("index_type", req.IndexType),
		))
	defer span.End()

	return c.index.CreateIndex(ctx, req, opts...)
}

func (c *CoordinatorClient) ListIndexes(ctx context.Context, req *pb.ListIndexesRequest, opts ...grpc.CallOption) (*pb.ListIndexesResponse, error) {
	ctx, span := c.tracer.Start(ctx, "CoordinatorClient.ListIndexes",
		trace.WithAttributes(
			attribute.Int("page", int(req.Page)),
			attribute.Int("page_size", int(req.PageSize)),
		))
	defer span.End()

	resp, err := c.index.ListIndexes(ctx, req, opts...)
	if err != nil {
		span.RecordError(err)
		return nil, err
	}

	span.SetAttributes(attribute.Int("index_count", len(resp.Indexes)))
	return resp, nil
}

func (c *CoordinatorClient) GetIndex(ctx context.Context, req *pb.GetIndexRequest, opts ...grpc.CallOption) (*pb.GetIndexResponse, error) {
	ctx, span := c.tracer.Start(ctx, "CoordinatorClient.GetIndex",
		trace.WithAttributes(
			attribute.String("index_id", req.IndexId),
		))
	defer span.End()

	return c.index.GetIndex(ctx, req, opts...)
}

func (c *CoordinatorClient) DeleteIndex(ctx context.Context, req *pb.DeleteIndexRequest, opts ...grpc.CallOption) (*pb.DeleteIndexResponse, error) {
	ctx, span := c.tracer.Start(ctx, "CoordinatorClient.DeleteIndex",
		trace.WithAttributes(
			attribute.String("index_id", req.IndexId),
		))
	defer span.End()

	return c.index.DeleteIndex(ctx, req, opts...)
}

func (c *CoordinatorClient) RebuildIndex(ctx context.Context, req *pb.RebuildIndexRequest, opts ...grpc.CallOption) (*pb.RebuildIndexResponse, error) {
	ctx, span := c.tracer.Start(ctx, "CoordinatorClient.RebuildIndex",
		trace.WithAttributes(
			attribute.String("index_id", req.IndexId),
			attribute.Bool("async", req.Async),
		))
	defer span.End()

	return c.index.RebuildIndex(ctx, req, opts...)
}

func (c *CoordinatorClient) HealthCheck(ctx context.Context, req *pb.HealthCheckRequest, opts ...grpc.CallOption) (*pb.HealthCheckResponse, error) {
	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	return c.health.Check(ctx, req, opts...)
}
