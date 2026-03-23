package merger

import (
	"sort"
	"time"

	"github.com/flexsearch/coordinator/internal/model"
	"github.com/flexsearch/coordinator/internal/util"
)

type Merger interface {
	Merge(results map[string]*model.EngineResult) *model.SearchResponse
	Sort(results []*ResultWithScore)
	Deduplicate(results []*model.SearchResult) []*model.SearchResult
}

type MergerConfig struct {
	Strategy    string
	RRFK        int
	Weights     map[string]float64
	TopK        int
}

type RRFMerger struct {
	config *MergerConfig
	logger *util.Logger
}

type WeightedMerger struct {
	config *MergerConfig
	logger *util.Logger
}

type ResultWithScore struct {
	Result *model.SearchResult
	Score  float64
}

func NewRRFMerger(config *MergerConfig, logger *util.Logger) *RRFMerger {
	if config.RRFK <= 0 {
		config.RRFK = 60
	}
	return &RRFMerger{
		config: config,
		logger: logger,
	}
}

func NewWeightedMerger(config *MergerConfig, logger *util.Logger) *WeightedMerger {
	return &WeightedMerger{
		config: config,
		logger: logger,
	}
}

func (m *RRFMerger) Merge(results map[string]*model.EngineResult) *model.SearchResponse {
	startTime := time.Now()
	
	var allResults []*model.SearchResult
	var enginesUsed []string
	var totalTook float64
	
	for engine, result := range results {
		if result != nil && len(result.Results) > 0 {
			enginesUsed = append(enginesUsed, engine)
			totalTook += result.Took
			
			for i := range result.Results {
				allResults = append(allResults, &result.Results[i])
			}
		}
	}
	
	deduplicated := m.Deduplicate(allResults)
	scores := m.calculateRRFScores(results)
	
	var scoredResults []*ResultWithScore
	for _, result := range deduplicated {
		if score, exists := scores[result.ID]; exists {
			scoredResults = append(scoredResults, &ResultWithScore{
				Result: result,
				Score:  score,
			})
		}
	}
	
	m.Sort(scoredResults)
	
	topK := m.config.TopK
	if topK <= 0 {
		topK = 100
	}
	
	var finalResults []model.SearchResult
	for i, sr := range scoredResults {
		if i >= topK {
			break
		}
		sr.Result.Score = sr.Score
		sr.Result.Rank = int32(i + 1)
		finalResults = append(finalResults, *sr.Result)
	}
	
	response := &model.SearchResponse{
		Results:     finalResults,
		Total:       int64(len(finalResults)),
		Took:        float64(time.Since(startTime).Milliseconds()),
		EnginesUsed: enginesUsed,
		CacheHit:    false,
	}
	
	m.logger.Debugw("RRF merge completed",
		"engines", len(enginesUsed),
		"results", len(finalResults),
		"took_ms", response.Took,
	)
	
	return response
}

func (m *RRFMerger) calculateRRFScores(results map[string]*model.EngineResult) map[string]float64 {
	scores := make(map[string]float64)
	
	for _, result := range results {
		if result == nil {
			continue
		}
		
		for rank, item := range result.Results {
			rrfScore := 1.0 / float64(m.config.RRFK+rank+1)
			scores[item.ID] += rrfScore
		}
	}
	
	return scores
}

func (m *RRFMerger) Sort(results []*ResultWithScore) {
	sort.Slice(results, func(i, j int) bool {
		return results[i].Score > results[j].Score
	})
}

func (m *RRFMerger) Deduplicate(results []*model.SearchResult) []*model.SearchResult {
	seen := make(map[string]bool)
	var deduplicated []*model.SearchResult
	
	for _, result := range results {
		if !seen[result.ID] {
			seen[result.ID] = true
			deduplicated = append(deduplicated, result)
		}
	}
	
	return deduplicated
}

func (m *WeightedMerger) Merge(results map[string]*model.EngineResult) *model.SearchResponse {
	startTime := time.Now()
	
	var allResults []*model.SearchResult
	var enginesUsed []string
	var totalTook float64
	
	for engine, result := range results {
		if result != nil && len(result.Results) > 0 {
			enginesUsed = append(enginesUsed, engine)
			totalTook += result.Took
			
			for i := range result.Results {
				allResults = append(allResults, &result.Results[i])
			}
		}
	}
	
	deduplicated := m.Deduplicate(allResults)
	scores := m.calculateWeightedScores(results)
	
	var scoredResults []*ResultWithScore
	for _, result := range deduplicated {
		if score, exists := scores[result.ID]; exists {
			scoredResults = append(scoredResults, &ResultWithScore{
				Result: result,
				Score:  score,
			})
		}
	}
	
	m.Sort(scoredResults)
	
	topK := m.config.TopK
	if topK <= 0 {
		topK = 100
	}
	
	var finalResults []model.SearchResult
	for i, sr := range scoredResults {
		if i >= topK {
			break
		}
		sr.Result.Score = sr.Score
		sr.Result.Rank = int32(i + 1)
		finalResults = append(finalResults, *sr.Result)
	}
	
	response := &model.SearchResponse{
		Results:     finalResults,
		Total:       int64(len(finalResults)),
		Took:        float64(time.Since(startTime).Milliseconds()),
		EnginesUsed: enginesUsed,
		CacheHit:    false,
	}
	
	m.logger.Debugw("Weighted merge completed",
		"engines", len(enginesUsed),
		"results", len(finalResults),
		"took_ms", response.Took,
	)
	
	return response
}

func (m *WeightedMerger) calculateWeightedScores(results map[string]*model.EngineResult) map[string]float64 {
	scores := make(map[string]float64)
	engineMaxScores := make(map[string]float64)
	
	for engine, result := range results {
		if result == nil {
			continue
		}
		
		maxScore := 0.0
		for _, item := range result.Results {
			if item.Score > maxScore {
				maxScore = item.Score
			}
		}
		engineMaxScores[engine] = maxScore
	}
	
	for engine, result := range results {
		if result == nil {
			continue
		}
		
		weight := m.config.Weights[engine]
		if weight <= 0 {
			weight = 1.0 / float64(len(results))
		}
		
		maxScore := engineMaxScores[engine]
		if maxScore == 0 {
			maxScore = 1.0
		}
		
		for _, item := range result.Results {
			normalizedScore := item.Score / maxScore
			scores[item.ID] += normalizedScore * weight
		}
	}
	
	return scores
}

func (m *WeightedMerger) Sort(results []*ResultWithScore) {
	sort.Slice(results, func(i, j int) bool {
		return results[i].Score > results[j].Score
	})
}

func (m *WeightedMerger) Deduplicate(results []*model.SearchResult) []*model.SearchResult {
	seen := make(map[string]bool)
	var deduplicated []*model.SearchResult
	
	for _, result := range results {
		if !seen[result.ID] {
			seen[result.ID] = true
			deduplicated = append(deduplicated, result)
		}
	}
	
	return deduplicated
}

func NewMerger(strategy string, config *MergerConfig, logger *util.Logger) Merger {
	config.Strategy = strategy
	
	switch strategy {
	case "rrf":
		return NewRRFMerger(config, logger)
	case "weighted":
		return NewWeightedMerger(config, logger)
	default:
		return NewRRFMerger(config, logger)
	}
}
