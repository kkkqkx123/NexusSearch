package vector

import (
	"context"
	"fmt"

	"github.com/qdrant/go-client/qdrant"
)

func (c *VectorClient) VectorSearch(ctx context.Context, collection string, query *SearchQuery) ([]*SearchResult, error) {
	if c.qdrantClient == nil {
		return nil, fmt.Errorf("Qdrant client not initialized")
	}

	queryPoints := &qdrant.QueryPoints{
		CollectionName: collection,
		Query:          qdrant.NewQuery(query.Vector...),
		Limit:          qdrant.PtrOf(uint64(query.Limit)),
		WithPayload:    qdrant.NewWithPayload(query.WithPayload),
		WithVectors:    qdrant.NewWithVectors(query.WithVector),
	}

	if query.Offset > 0 {
		queryPoints.Offset = qdrant.PtrOf(uint64(query.Offset))
	}

	if query.ScoreThreshold > 0 {
		queryPoints.ScoreThreshold = qdrant.PtrOf(query.ScoreThreshold)
	}

	if query.Filter != nil {
		queryPoints.Filter = ConvertFilter(query.Filter)
	}

	points, err := c.qdrantClient.Query(ctx, queryPoints)
	if err != nil {
		return nil, err
	}

	results := make([]*SearchResult, len(points))
	for i, p := range points {
		results[i] = convertScoredPoint(p)
	}

	return results, nil
}

func (c *VectorClient) SearchBatch(ctx context.Context, collection string, queries []*SearchQuery) ([][]*SearchResult, error) {
	if c.qdrantClient == nil {
		return nil, fmt.Errorf("Qdrant client not initialized")
	}

	allResults := make([][]*SearchResult, len(queries))
	for i, query := range queries {
		results, err := c.VectorSearch(ctx, collection, query)
		if err != nil {
			return nil, err
		}
		allResults[i] = results
	}

	return allResults, nil
}

func (c *VectorClient) Scroll(ctx context.Context, collection string, opts *ScrollOptions) (*ScrollResult, error) {
	if c.qdrantClient == nil {
		return nil, fmt.Errorf("Qdrant client not initialized")
	}

	if opts == nil {
		opts = &ScrollOptions{
			Limit:       10,
			WithPayload: true,
			WithVector:  false,
		}
	}

	scrollReq := &qdrant.ScrollPoints{
		CollectionName: collection,
		Limit:          qdrant.PtrOf(uint32(opts.Limit)),
		WithPayload:    qdrant.NewWithPayload(opts.WithPayload),
		WithVectors:    qdrant.NewWithVectors(opts.WithVector),
	}

	if opts.Offset != "" {
		scrollReq.Offset = qdrant.NewID(opts.Offset)
	}

	if opts.Filter != nil {
		scrollReq.Filter = ConvertFilter(opts.Filter)
	}

	result, err := c.qdrantClient.Scroll(ctx, scrollReq)
	if err != nil {
		return nil, err
	}

	points := make([]*VectorPoint, len(result))
	for i, p := range result {
		points[i] = convertRetrievedPoint(p)
	}

	var nextOffset string
	if len(result) > 0 && result[len(result)-1].Id != nil {
		nextOffset = fmt.Sprintf("%v", result[len(result)-1].Id)
	}

	return &ScrollResult{
		Points:     points,
		NextOffset: nextOffset,
	}, nil
}

func (c *VectorClient) Count(ctx context.Context, collection string) (uint64, error) {
	if c.qdrantClient == nil {
		return 0, fmt.Errorf("Qdrant client not initialized")
	}

	count, err := c.qdrantClient.Count(ctx, &qdrant.CountPoints{
		CollectionName: collection,
		Exact:          qdrant.PtrOf(true),
	})
	if err != nil {
		return 0, err
	}

	return count, nil
}

func (c *VectorClient) CountWithFilter(ctx context.Context, collection string, filter *VectorFilter) (uint64, error) {
	if c.qdrantClient == nil {
		return 0, fmt.Errorf("Qdrant client not initialized")
	}

	count, err := c.qdrantClient.Count(ctx, &qdrant.CountPoints{
		CollectionName: collection,
		Filter:         ConvertFilter(filter),
		Exact:          qdrant.PtrOf(true),
	})
	if err != nil {
		return 0, err
	}

	return count, nil
}

func convertScoredPoint(point *qdrant.ScoredPoint) *SearchResult {
	if point == nil {
		return nil
	}

	id := ""
	if point.Id != nil {
		id = fmt.Sprintf("%v", point.Id)
	}

	payload := make(map[string]interface{})
	for k, v := range point.Payload {
		payload[k] = extractValue(v)
	}

	var vector []float32
	if point.Vectors != nil {
		switch v := point.Vectors.VectorsOptions.(type) {
		case *qdrant.VectorsOutput_Vector:
			vector = v.Vector.Data
		}
	}

	return &SearchResult{
		ID:      id,
		Score:   point.Score,
		Payload: payload,
		Vector:  vector,
	}
}
