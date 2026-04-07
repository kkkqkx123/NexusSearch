package vector

import (
	"context"
	"fmt"

	"github.com/qdrant/go-client/qdrant"
)

func (c *VectorClient) Upsert(ctx context.Context, collection string, point *VectorPoint) (*UpsertResult, error) {
	if c.qdrantClient == nil {
		return nil, fmt.Errorf("Qdrant client not initialized")
	}

	_, err := c.qdrantClient.Upsert(ctx, &qdrant.UpsertPoints{
		CollectionName: collection,
		Points: []*qdrant.PointStruct{
			{
				Id:      qdrant.NewID(point.ID),
				Vectors: qdrant.NewVectors(point.Vector...),
				Payload: qdrant.NewValueMap(point.Payload),
			},
		},
	})
	if err != nil {
		return nil, err
	}

	return &UpsertResult{
		Status: UpsertStatusCompleted,
	}, nil
}

func (c *VectorClient) UpsertBatch(ctx context.Context, collection string, points []*VectorPoint) (*UpsertResult, error) {
	if c.qdrantClient == nil {
		return nil, fmt.Errorf("Qdrant client not initialized")
	}

	qdrantPoints := make([]*qdrant.PointStruct, len(points))
	for i, p := range points {
		qdrantPoints[i] = &qdrant.PointStruct{
			Id:      qdrant.NewID(p.ID),
			Vectors: qdrant.NewVectors(p.Vector...),
			Payload: qdrant.NewValueMap(p.Payload),
		}
	}

	_, err := c.qdrantClient.Upsert(ctx, &qdrant.UpsertPoints{
		CollectionName: collection,
		Points:         qdrantPoints,
	})
	if err != nil {
		return nil, err
	}

	return &UpsertResult{
		Status: UpsertStatusCompleted,
	}, nil
}

func (c *VectorClient) Get(ctx context.Context, collection string, id string) (*VectorPoint, error) {
	if c.qdrantClient == nil {
		return nil, fmt.Errorf("Qdrant client not initialized")
	}

	points, err := c.qdrantClient.Get(ctx, &qdrant.GetPoints{
		CollectionName: collection,
		Ids:            []*qdrant.PointId{qdrant.NewID(id)},
		WithPayload:    qdrant.NewWithPayload(true),
		WithVectors:    qdrant.NewWithVectors(true),
	})
	if err != nil {
		return nil, err
	}

	if len(points) == 0 {
		return nil, nil
	}

	return convertRetrievedPoint(points[0]), nil
}

func (c *VectorClient) GetBatch(ctx context.Context, collection string, ids []string) ([]*VectorPoint, error) {
	if c.qdrantClient == nil {
		return nil, fmt.Errorf("Qdrant client not initialized")
	}

	pointIds := make([]*qdrant.PointId, len(ids))
	for i, id := range ids {
		pointIds[i] = qdrant.NewID(id)
	}

	points, err := c.qdrantClient.Get(ctx, &qdrant.GetPoints{
		CollectionName: collection,
		Ids:            pointIds,
		WithPayload:    qdrant.NewWithPayload(true),
		WithVectors:    qdrant.NewWithVectors(false),
	})
	if err != nil {
		return nil, err
	}

	result := make([]*VectorPoint, len(points))
	for i, p := range points {
		result[i] = convertRetrievedPoint(p)
	}

	return result, nil
}

func (c *VectorClient) Delete(ctx context.Context, collection string, id string) (*DeleteResult, error) {
	if c.qdrantClient == nil {
		return nil, fmt.Errorf("Qdrant client not initialized")
	}

	_, err := c.qdrantClient.Delete(ctx, &qdrant.DeletePoints{
		CollectionName: collection,
		Points: &qdrant.PointsSelector{
			PointsSelectorOneOf: &qdrant.PointsSelector_Points{
				Points: &qdrant.PointsIdsList{
					Ids: []*qdrant.PointId{qdrant.NewID(id)},
				},
			},
		},
	})
	if err != nil {
		return nil, err
	}

	return &DeleteResult{
		DeletedCount: 1,
	}, nil
}

func (c *VectorClient) DeleteBatch(ctx context.Context, collection string, ids []string) (*DeleteResult, error) {
	if c.qdrantClient == nil {
		return nil, fmt.Errorf("Qdrant client not initialized")
	}

	pointIds := make([]*qdrant.PointId, len(ids))
	for i, id := range ids {
		pointIds[i] = qdrant.NewID(id)
	}

	_, err := c.qdrantClient.Delete(ctx, &qdrant.DeletePoints{
		CollectionName: collection,
		Points: &qdrant.PointsSelector{
			PointsSelectorOneOf: &qdrant.PointsSelector_Points{
				Points: &qdrant.PointsIdsList{
					Ids: pointIds,
				},
			},
		},
	})
	if err != nil {
		return nil, err
	}

	return &DeleteResult{
		DeletedCount: uint64(len(ids)),
	}, nil
}

func (c *VectorClient) DeleteByFilter(ctx context.Context, collection string, filter *VectorFilter) (*DeleteResult, error) {
	if c.qdrantClient == nil {
		return nil, fmt.Errorf("Qdrant client not initialized")
	}

	_, err := c.qdrantClient.Delete(ctx, &qdrant.DeletePoints{
		CollectionName: collection,
		Points: &qdrant.PointsSelector{
			PointsSelectorOneOf: &qdrant.PointsSelector_Filter{
				Filter: ConvertFilter(filter),
			},
		},
	})
	if err != nil {
		return nil, err
	}

	return &DeleteResult{}, nil
}

func convertRetrievedPoint(point *qdrant.RetrievedPoint) *VectorPoint {
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

	return &VectorPoint{
		ID:      id,
		Vector:  vector,
		Payload: payload,
	}
}
