package vector

import (
	"context"
	"fmt"

	"github.com/qdrant/go-client/qdrant"
)

func (c *VectorClient) SetPayload(ctx context.Context, collection string, ids []string, payload map[string]interface{}) error {
	if c.qdrantClient == nil {
		return fmt.Errorf("Qdrant client not initialized")
	}

	pointIds := make([]*qdrant.PointId, len(ids))
	for i, id := range ids {
		pointIds[i] = qdrant.NewID(id)
	}

	_, err := c.qdrantClient.SetPayload(ctx, &qdrant.SetPayloadPoints{
		CollectionName: collection,
		PointsSelector: qdrant.NewPointsSelector(pointIds...),
		Payload:        qdrant.NewValueMap(payload),
	})

	return err
}

func (c *VectorClient) SetPayloadByFilter(ctx context.Context, collection string, filter *VectorFilter, payload map[string]interface{}) error {
	if c.qdrantClient == nil {
		return fmt.Errorf("Qdrant client not initialized")
	}

	_, err := c.qdrantClient.SetPayload(ctx, &qdrant.SetPayloadPoints{
		CollectionName: collection,
		PointsSelector: qdrant.NewPointsSelectorFilter(ConvertFilter(filter)),
		Payload:        qdrant.NewValueMap(payload),
	})

	return err
}

func (c *VectorClient) DeletePayload(ctx context.Context, collection string, ids []string, keys []string) error {
	if c.qdrantClient == nil {
		return fmt.Errorf("Qdrant client not initialized")
	}

	pointIds := make([]*qdrant.PointId, len(ids))
	for i, id := range ids {
		pointIds[i] = qdrant.NewID(id)
	}

	_, err := c.qdrantClient.DeletePayload(ctx, &qdrant.DeletePayloadPoints{
		CollectionName: collection,
		PointsSelector: qdrant.NewPointsSelector(pointIds...),
		Keys:           keys,
	})

	return err
}

func (c *VectorClient) DeletePayloadByFilter(ctx context.Context, collection string, filter *VectorFilter, keys []string) error {
	if c.qdrantClient == nil {
		return fmt.Errorf("Qdrant client not initialized")
	}

	_, err := c.qdrantClient.DeletePayload(ctx, &qdrant.DeletePayloadPoints{
		CollectionName: collection,
		PointsSelector: qdrant.NewPointsSelectorFilter(ConvertFilter(filter)),
		Keys:           keys,
	})

	return err
}

func (c *VectorClient) CreatePayloadIndex(ctx context.Context, collection string, field string, schemaType PayloadSchemaType) error {
	if c.qdrantClient == nil {
		return fmt.Errorf("Qdrant client not initialized")
	}

	var fieldType qdrant.FieldType
	switch schemaType {
	case PayloadSchemaKeyword:
		fieldType = qdrant.FieldType_FieldTypeKeyword
	case PayloadSchemaInteger:
		fieldType = qdrant.FieldType_FieldTypeInteger
	case PayloadSchemaFloat:
		fieldType = qdrant.FieldType_FieldTypeFloat
	case PayloadSchemaText:
		fieldType = qdrant.FieldType_FieldTypeText
	case PayloadSchemaBool:
		fieldType = qdrant.FieldType_FieldTypeBool
	case PayloadSchemaGeo:
		fieldType = qdrant.FieldType_FieldTypeGeo
	case PayloadSchemaDatetime:
		fieldType = qdrant.FieldType_FieldTypeDatetime
	default:
		fieldType = qdrant.FieldType_FieldTypeKeyword
	}

	_, err := c.qdrantClient.CreateFieldIndex(ctx, &qdrant.CreateFieldIndexCollection{
		CollectionName: collection,
		FieldName:      field,
		FieldType:      qdrant.PtrOf(fieldType),
	})

	return err
}

func (c *VectorClient) DeletePayloadIndex(ctx context.Context, collection string, field string) error {
	if c.qdrantClient == nil {
		return fmt.Errorf("Qdrant client not initialized")
	}

	_, err := c.qdrantClient.DeleteFieldIndex(ctx, &qdrant.DeleteFieldIndexCollection{
		CollectionName: collection,
		FieldName:      field,
	})

	return err
}
