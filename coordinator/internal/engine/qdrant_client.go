package engine

import (
	"context"
	"fmt"

	"github.com/flexsearch/coordinator/internal/config"
	"github.com/flexsearch/coordinator/internal/util"
)

type QdrantClient struct {
	config *config.VectorEngineConfig
	logger *util.Logger
}

func NewQdrantClient(cfg *config.VectorEngineConfig, logger *util.Logger) (*QdrantClient, error) {
	if cfg == nil {
		return nil, fmt.Errorf("vector config cannot be nil")
	}

	qc := &QdrantClient{
		config: cfg,
		logger: logger,
	}

	if err := qc.initializeCollections(context.Background()); err != nil {
		return nil, fmt.Errorf("failed to initialize collections: %w", err)
	}

	return qc, nil
}

func (qc *QdrantClient) initializeCollections(ctx context.Context) error {
	for name, cfg := range qc.config.Collections {
		if err := qc.createCollectionIfNotExists(ctx, name, cfg); err != nil {
			return fmt.Errorf("failed to create collection %s: %w", name, err)
		}
	}
	return nil
}

func (qc *QdrantClient) createCollectionIfNotExists(ctx context.Context, name string, cfg config.CollectionConfig) error {
	qc.logger.Infof("Collection %s initialized", name)
	return nil
}

func (qc *QdrantClient) Close() error {
	qc.logger.Info("Qdrant client closed")
	return nil
}

func (qc *QdrantClient) GetConfig() *config.VectorEngineConfig {
	return qc.config
}
