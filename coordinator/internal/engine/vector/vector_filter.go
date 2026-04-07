package vector

import (
	"github.com/qdrant/go-client/qdrant"
)

type GeoPoint struct {
	Lat float64
	Lon float64
}

func NewGeoPoint(lat, lon float64) *GeoPoint {
	return &GeoPoint{Lat: lat, Lon: lon}
}

type GeoRadius struct {
	Center *GeoPoint
	Radius float64
}

func NewGeoRadius(lat, lon, radius float64) *GeoRadius {
	return &GeoRadius{
		Center: NewGeoPoint(lat, lon),
		Radius: radius,
	}
}

type GeoBoundingBox struct {
	TopLeft     *GeoPoint
	BottomRight *GeoPoint
}

func NewGeoBoundingBox(topLeftLat, topLeftLon, bottomRightLat, bottomRightLon float64) *GeoBoundingBox {
	return &GeoBoundingBox{
		TopLeft:     NewGeoPoint(topLeftLat, topLeftLon),
		BottomRight: NewGeoPoint(bottomRightLat, bottomRightLon),
	}
}

type RangeCondition struct {
	Gt  float64
	Gte float64
	Lt  float64
	Lte float64
}

func NewRangeCondition() *RangeCondition {
	return &RangeCondition{}
}

func (r *RangeCondition) WithGt(v float64) *RangeCondition {
	r.Gt = v
	return r
}

func (r *RangeCondition) WithGte(v float64) *RangeCondition {
	r.Gte = v
	return r
}

func (r *RangeCondition) WithLt(v float64) *RangeCondition {
	r.Lt = v
	return r
}

func (r *RangeCondition) WithLte(v float64) *RangeCondition {
	r.Lte = v
	return r
}

type ValuesCountCondition struct {
	Gt  uint64
	Gte uint64
	Lt  uint64
	Lte uint64
}

func NewValuesCountCondition() *ValuesCountCondition {
	return &ValuesCountCondition{}
}

func (v *ValuesCountCondition) WithGt(val uint64) *ValuesCountCondition {
	v.Gt = val
	return v
}

func (v *ValuesCountCondition) WithGte(val uint64) *ValuesCountCondition {
	v.Gte = val
	return v
}

func (v *ValuesCountCondition) WithLt(val uint64) *ValuesCountCondition {
	v.Lt = val
	return v
}

func (v *ValuesCountCondition) WithLte(val uint64) *ValuesCountCondition {
	v.Lte = val
	return v
}

type ConditionType interface {
	isConditionType()
}

type MatchCondition struct {
	Value string
}

func (MatchCondition) isConditionType() {}

type MatchAnyCondition struct {
	Values []string
}

func (MatchAnyCondition) isConditionType() {}

type RangeCond struct {
	Range *RangeCondition
}

func (RangeCond) isConditionType() {}

type IsEmptyCondition struct{}

func (IsEmptyCondition) isConditionType() {}

type IsNullCondition struct{}

func (IsNullCondition) isConditionType() {}

type HasIDCondition struct {
	IDs []string
}

func (HasIDCondition) isConditionType() {}

type GeoRadiusCondition struct {
	Radius *GeoRadius
}

func (GeoRadiusCondition) isConditionType() {}

type GeoBoundingBoxCondition struct {
	Box *GeoBoundingBox
}

func (GeoBoundingBoxCondition) isConditionType() {}

type ValuesCountCond struct {
	Count *ValuesCountCondition
}

func (ValuesCountCond) isConditionType() {}

type NestedCondition struct {
	Filter *VectorFilter
}

func (NestedCondition) isConditionType() {}

type ContainsCondition struct {
	Value string
}

func (ContainsCondition) isConditionType() {}

type FilterCondition struct {
	Field     string
	Condition ConditionType
}

func Match(field, value string) *FilterCondition {
	return &FilterCondition{
		Field:     field,
		Condition: MatchCondition{Value: value},
	}
}

func MatchAny(field string, values []string) *FilterCondition {
	return &FilterCondition{
		Field:     field,
		Condition: MatchAnyCondition{Values: values},
	}
}

func Range(field string, r *RangeCondition) *FilterCondition {
	return &FilterCondition{
		Field:     field,
		Condition: RangeCond{Range: r},
	}
}

func IsEmpty(field string) *FilterCondition {
	return &FilterCondition{
		Field:     field,
		Condition: IsEmptyCondition{},
	}
}

func IsNull(field string) *FilterCondition {
	return &FilterCondition{
		Field:     field,
		Condition: IsNullCondition{},
	}
}

func HasID(ids []string) *FilterCondition {
	return &FilterCondition{
		Field:     "_id",
		Condition: HasIDCondition{IDs: ids},
	}
}

func GeoRadiusFilter(field string, radius *GeoRadius) *FilterCondition {
	return &FilterCondition{
		Field:     field,
		Condition: GeoRadiusCondition{Radius: radius},
	}
}

func GeoBoundingBoxFilter(field string, box *GeoBoundingBox) *FilterCondition {
	return &FilterCondition{
		Field:     field,
		Condition: GeoBoundingBoxCondition{Box: box},
	}
}

func ValuesCountFilter(field string, count *ValuesCountCondition) *FilterCondition {
	return &FilterCondition{
		Field:     field,
		Condition: ValuesCountCond{Count: count},
	}
}

func Contains(field, value string) *FilterCondition {
	return &FilterCondition{
		Field:     field,
		Condition: ContainsCondition{Value: value},
	}
}

func Nested(field string, filter *VectorFilter) *FilterCondition {
	return &FilterCondition{
		Field:     field,
		Condition: NestedCondition{Filter: filter},
	}
}

type MinShouldCondition struct {
	Conditions []*FilterCondition
	MinCount   int
}

type VectorFilter struct {
	Must      []*FilterCondition
	MustNot   []*FilterCondition
	Should    []*FilterCondition
	MinShould *MinShouldCondition
}

func NewVectorFilter() *VectorFilter {
	return &VectorFilter{}
}

func (f *VectorFilter) AddMust(condition *FilterCondition) *VectorFilter {
	f.Must = append(f.Must, condition)
	return f
}

func (f *VectorFilter) AddMustNot(condition *FilterCondition) *VectorFilter {
	f.MustNot = append(f.MustNot, condition)
	return f
}

func (f *VectorFilter) AddShould(condition *FilterCondition) *VectorFilter {
	f.Should = append(f.Should, condition)
	return f
}

func (f *VectorFilter) WithMinShould(minShould *MinShouldCondition) *VectorFilter {
	f.MinShould = minShould
	return f
}

func ConvertFilter(filter *VectorFilter) *qdrant.Filter {
	if filter == nil {
		return nil
	}

	qdrantFilter := &qdrant.Filter{}

	for _, cond := range filter.Must {
		if qdrantCond := convertCondition(cond); qdrantCond != nil {
			qdrantFilter.Must = append(qdrantFilter.Must, qdrantCond)
		}
	}

	for _, cond := range filter.MustNot {
		if qdrantCond := convertCondition(cond); qdrantCond != nil {
			qdrantFilter.MustNot = append(qdrantFilter.MustNot, qdrantCond)
		}
	}

	for _, cond := range filter.Should {
		if qdrantCond := convertCondition(cond); qdrantCond != nil {
			qdrantFilter.Should = append(qdrantFilter.Should, qdrantCond)
		}
	}

	return qdrantFilter
}

func convertCondition(cond *FilterCondition) *qdrant.Condition {
	if cond == nil {
		return nil
	}

	switch c := cond.Condition.(type) {
	case MatchCondition:
		return qdrant.NewMatch(cond.Field, c.Value)

	case MatchAnyCondition:
		return qdrant.NewMatchKeywords(cond.Field, c.Values...)

	case RangeCond:
		if c.Range == nil {
			return nil
		}
		r := &qdrant.Range{}
		if c.Range.Gt != 0 {
			r.Gt = ptrFloat(c.Range.Gt)
		}
		if c.Range.Gte != 0 {
			r.Gte = ptrFloat(c.Range.Gte)
		}
		if c.Range.Lt != 0 {
			r.Lt = ptrFloat(c.Range.Lt)
		}
		if c.Range.Lte != 0 {
			r.Lte = ptrFloat(c.Range.Lte)
		}
		return qdrant.NewRange(cond.Field, r)

	case IsEmptyCondition:
		return qdrant.NewIsEmpty(cond.Field)

	case IsNullCondition:
		return qdrant.NewIsNull(cond.Field)

	case HasIDCondition:
		ids := make([]*qdrant.PointId, len(c.IDs))
		for i, id := range c.IDs {
			ids[i] = qdrant.NewID(id)
		}
		return qdrant.NewHasID(ids...)

	case GeoRadiusCondition:
		if c.Radius == nil || c.Radius.Center == nil {
			return nil
		}
		return qdrant.NewGeoRadius(cond.Field, c.Radius.Center.Lat, c.Radius.Center.Lon, float32(c.Radius.Radius))

	case GeoBoundingBoxCondition:
		if c.Box == nil || c.Box.TopLeft == nil || c.Box.BottomRight == nil {
			return nil
		}
		return qdrant.NewGeoBoundingBox(cond.Field,
			c.Box.TopLeft.Lat, c.Box.TopLeft.Lon,
			c.Box.BottomRight.Lat, c.Box.BottomRight.Lon)

	case ValuesCountCond:
		if c.Count == nil {
			return nil
		}
		vc := &qdrant.ValuesCount{}
		if c.Count.Gt != 0 {
			vc.Gt = ptrUint64(c.Count.Gt)
		}
		if c.Count.Gte != 0 {
			vc.Gte = ptrUint64(c.Count.Gte)
		}
		if c.Count.Lt != 0 {
			vc.Lt = ptrUint64(c.Count.Lt)
		}
		if c.Count.Lte != 0 {
			vc.Lte = ptrUint64(c.Count.Lte)
		}
		return qdrant.NewValuesCount(cond.Field, vc)

	case ContainsCondition:
		return qdrant.NewMatch(cond.Field, c.Value)

	case NestedCondition:
		if c.Filter == nil {
			return nil
		}
		return qdrant.NewNestedFilter(cond.Field, ConvertFilter(c.Filter))
	}

	return nil
}

func ptrFloat(v float64) *float64 {
	return &v
}

func ptrUint64(v uint64) *uint64 {
	return &v
}

type PayloadSelector struct {
	Include []string
	Exclude []string
}

func PayloadSelectorInclude(fields ...string) *PayloadSelector {
	return &PayloadSelector{Include: fields}
}

func PayloadSelectorExclude(fields ...string) *PayloadSelector {
	return &PayloadSelector{Exclude: fields}
}

func PayloadSelectorAll() *PayloadSelector {
	return &PayloadSelector{}
}
