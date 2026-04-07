package vector

import (
	"context"
	"fmt"

	"github.com/qdrant/go-client/qdrant"
)

func (c *VectorClient) CreateCollection(ctx context.Context, config *CollectionConfig) error {
	if c.qdrantClient == nil {
		return fmt.Errorf("Qdrant client not initialized")
	}

	vectorsConfig := &qdrant.VectorParams{
		Size:     uint64(config.VectorSize),
		Distance: convertDistance(config.Distance),
	}

	err := c.qdrantClient.CreateCollection(ctx, &qdrant.CreateCollection{
		CollectionName: config.Name,
		VectorsConfig: &qdrant.VectorsConfig{
			Config: &qdrant.VectorsConfig_Params{
				Params: vectorsConfig,
			},
		},
	})

	return err
}

func (c *VectorClient) DeleteCollection(ctx context.Context, name string) error {
	if c.qdrantClient == nil {
		return fmt.Errorf("Qdrant client not initialized")
	}

	return c.qdrantClient.DeleteCollection(ctx, name)
}

func (c *VectorClient) CollectionExists(ctx context.Context, name string) (bool, error) {
	if c.qdrantClient == nil {
		return false, fmt.Errorf("Qdrant client not initialized")
	}

	collections, err := c.qdrantClient.ListCollections(ctx)
	if err != nil {
		return false, err
	}

	for _, col := range collections {
		if col == name {
			return true, nil
		}
	}

	return false, nil
}

func (c *VectorClient) CollectionInfo(ctx context.Context, name string) (*CollectionInfo, error) {
	if c.qdrantClient == nil {
		return nil, fmt.Errorf("Qdrant client not initialized")
	}

	info, err := c.qdrantClient.GetCollectionInfo(ctx, name)
	if err != nil {
		return nil, err
	}

	result := &CollectionInfo{
		Name:          name,
		PointsCount:   *info.PointsCount,
		SegmentsCount: info.SegmentsCount,
	}

	return result, nil
}

func (c *VectorClient) ListCollections(ctx context.Context) ([]string, error) {
	if c.qdrantClient == nil {
		return nil, fmt.Errorf("Qdrant client not initialized")
	}

	return c.qdrantClient.ListCollections(ctx)
}

func (c *VectorClient) UpdateCollection(ctx context.Context, name string, params *CollectionUpdateParams) error {
	if c.qdrantClient == nil {
		return fmt.Errorf("Qdrant client not initialized")
	}

	updateParams := &qdrant.UpdateCollection{
		CollectionName: name,
	}

	err := c.qdrantClient.UpdateCollection(ctx, updateParams)
	return err
}

func convertDistance(distance DistanceType) qdrant.Distance {
	switch distance {
	case DistanceCosine:
		return qdrant.Distance_Cosine
	case DistanceEuclidean:
		return qdrant.Distance_Euclid
	case DistanceDotProduct:
		return qdrant.Distance_Dot
	default:
		return qdrant.Distance_Cosine
	}
}
