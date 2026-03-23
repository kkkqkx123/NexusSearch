package router

import (
	"context"
	"regexp"
	"strings"
	"time"

	"github.com/flexsearch/coordinator/internal/model"
	"github.com/flexsearch/coordinator/internal/util"
)

type Optimizer struct {
	logger      *util.Logger
	synonyms    map[string][]string
	stopWords   map[string]bool
	stats       *OptimizerStats
}

type OptimizerStats struct {
	TotalQueries       int64
	RewrittenQueries   int64
	SuggestionsGenerated int64
	AverageRewriteTime float64
}

type OptimizedQuery struct {
	OriginalQuery   string
	RewrittenQuery  string
	Suggestions     []string
	Rewritten       bool
	ProcessingTime  time.Duration
}

func NewOptimizer(logger *util.Logger) *Optimizer {
	return &Optimizer{
		logger:    logger,
		synonyms:  loadDefaultSynonyms(),
		stopWords: loadDefaultStopWords(),
		stats:     &OptimizerStats{},
	}
}

func (o *Optimizer) Optimize(ctx context.Context, req *model.SearchRequest) *OptimizedQuery {
	startTime := time.Now()
	
	optimized := &OptimizedQuery{
		OriginalQuery:  req.Query,
		RewrittenQuery: req.Query,
		Suggestions:    []string{},
		Rewritten:      false,
	}

	query := strings.TrimSpace(req.Query)
	
	rewritten := o.rewriteQuery(query)
	if rewritten != query {
		optimized.RewrittenQuery = rewritten
		optimized.Rewritten = true
		o.stats.RewrittenQueries++
	}

	suggestions := o.generateSuggestions(query)
	optimized.Suggestions = suggestions
	if len(suggestions) > 0 {
		o.stats.SuggestionsGenerated++
	}

	optimized.ProcessingTime = time.Since(startTime)
	o.stats.TotalQueries++
	
	o.updateAverageRewriteTime(optimized.ProcessingTime)

	o.logger.Debugw("Query optimized",
		"original", optimized.OriginalQuery,
		"rewritten", optimized.RewrittenQuery,
		"suggestions", len(optimized.Suggestions),
		"processing_time", optimized.ProcessingTime,
	)

	return optimized
}

func (o *Optimizer) rewriteQuery(query string) string {
	query = o.removeStopWords(query)
	query = o.expandSynonyms(query)
	query = o.normalizeQuery(query)
	
	return query
}

func (o *Optimizer) removeStopWords(query string) string {
	words := strings.Fields(query)
	var filtered []string
	
	for _, word := range words {
		lowerWord := strings.ToLower(word)
		if !o.stopWords[lowerWord] {
			filtered = append(filtered, word)
		}
	}
	
	return strings.Join(filtered, " ")
}

func (o *Optimizer) expandSynonyms(query string) string {
	words := strings.Fields(query)
	var expanded []string
	
	for _, word := range words {
		lowerWord := strings.ToLower(word)
		if synonyms, exists := o.synonyms[lowerWord]; exists {
			expanded = append(expanded, word)
			expanded = append(expanded, synonyms...)
		} else {
			expanded = append(expanded, word)
		}
	}
	
	return strings.Join(expanded, " ")
}

func (o *Optimizer) normalizeQuery(query string) string {
	query = strings.ToLower(query)
	query = regexp.MustCompile(`\s+`).ReplaceAllString(query, " ")
	query = strings.TrimSpace(query)
	
	return query
}

func (o *Optimizer) generateSuggestions(query string) []string {
	var suggestions []string
	
	words := strings.Fields(query)
	
	for i, word := range words {
		corrected := o.correctSpelling(word)
		if corrected != word {
			suggestion := make([]string, len(words))
			copy(suggestion, words)
			suggestion[i] = corrected
			suggestions = append(suggestions, strings.Join(suggestion, " "))
		}
	}
	
	if len(words) > 1 {
		for i := 0; i < len(words)-1; i++ {
			phrase := words[i] + " " + words[i+1]
			if synonyms, exists := o.synonyms[strings.ToLower(phrase)]; exists {
				for _, synonym := range synonyms {
					suggestion := make([]string, len(words))
					copy(suggestion, words)
					suggestion[i] = synonym
					suggestion = append(suggestion[:i+1], suggestion[i+2:]...)
					suggestions = append(suggestions, strings.Join(suggestion, " "))
				}
			}
		}
	}
	
	return suggestions
}

func (o *Optimizer) correctSpelling(word string) string {
	lowerWord := strings.ToLower(word)
	
	for key := range o.synonyms {
		distance := levenshteinDistance(lowerWord, key)
		if distance == 1 {
			return key
		}
	}
	
	return word
}

func (o *Optimizer) GetStats() *OptimizerStats {
	return o.stats
}

func (o *Optimizer) updateAverageRewriteTime(duration time.Duration) {
	if o.stats.TotalQueries == 0 {
		o.stats.AverageRewriteTime = float64(duration.Milliseconds())
		return
	}
	
	currentAvg := o.stats.AverageRewriteTime
	newAvg := (currentAvg*float64(o.stats.TotalQueries-1) + float64(duration.Milliseconds())) / float64(o.stats.TotalQueries)
	o.stats.AverageRewriteTime = newAvg
}

func loadDefaultSynonyms() map[string][]string {
	return map[string][]string{
		"search":    {"find", "lookup", "query"},
		"find":      {"search", "locate", "discover"},
		"get":       {"retrieve", "fetch", "obtain"},
		"retrieve":  {"get", "fetch", "obtain"},
		"show":      {"display", "present", "exhibit"},
		"display":   {"show", "present", "render"},
		"list":      {"enumerate", "catalog", "index"},
		"create":    {"make", "build", "construct"},
		"update":    {"modify", "change", "edit"},
		"delete":    {"remove", "erase", "eliminate"},
		"remove":    {"delete", "erase", "eliminate"},
		"add":       {"insert", "append", "include"},
		"insert":    {"add", "append", "include"},
		"machine learning": {"ml", "ai", "artificial intelligence"},
		"ai":        {"artificial intelligence", "machine learning"},
		"database":  {"db", "data store", "repository"},
		"api":       {"interface", "endpoint", "service"},
	}
}

func loadDefaultStopWords() map[string]bool {
	stopWords := []string{
		"a", "an", "the", "and", "or", "but", "is", "are", "was", "were",
		"be", "been", "being", "have", "has", "had", "do", "does", "did",
		"will", "would", "could", "should", "may", "might", "must", "shall",
		"can", "need", "dare", "ought", "used", "to", "of", "in", "for",
		"on", "with", "at", "by", "from", "as", "into", "through", "during",
		"before", "after", "above", "below", "between", "under", "again",
		"further", "then", "once", "here", "there", "when", "where", "why",
		"how", "all", "each", "few", "more", "most", "other", "some", "such",
		"no", "nor", "not", "only", "own", "same", "so", "than", "too", "very",
	}
	
	stopWordMap := make(map[string]bool)
	for _, word := range stopWords {
		stopWordMap[word] = true
	}
	
	return stopWordMap
}

func levenshteinDistance(s1, s2 string) int {
	if len(s1) == 0 {
		return len(s2)
	}
	if len(s2) == 0 {
		return len(s1)
	}

	matrix := make([][]int, len(s1)+1)
	for i := range matrix {
		matrix[i] = make([]int, len(s2)+1)
		matrix[i][0] = i
	}
	for j := range matrix[0] {
		matrix[0][j] = j
	}

	for i := 1; i <= len(s1); i++ {
		for j := 1; j <= len(s2); j++ {
			cost := 0
			if s1[i-1] != s2[j-1] {
				cost = 1
			}
			matrix[i][j] = min(
				matrix[i-1][j]+1,
				matrix[i][j-1]+1,
				matrix[i-1][j-1]+cost,
			)
		}
	}

	return matrix[len(s1)][len(s2)]
}

func min(a, b, c int) int {
	if a < b {
		if a < c {
			return a
		}
		return c
	}
	if b < c {
		return b
	}
	return c
}
