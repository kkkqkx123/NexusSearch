package redis

import (
	"sync"
)

type PoolManager struct {
	pools map[string]*Client
	mu    sync.RWMutex
}

var (
	instance *PoolManager
	once     sync.Once
)

func GetPoolManager() *PoolManager {
	once.Do(func() {
		instance = &PoolManager{
			pools: make(map[string]*Client),
		}
	})
	return instance
}

func (pm *PoolManager) GetClient(name string, config *Config) (*Client, error) {
	pm.mu.RLock()
	if client, exists := pm.pools[name]; exists {
		pm.mu.RUnlock()
		return client, nil
	}
	pm.mu.RUnlock()

	pm.mu.Lock()
	defer pm.mu.Unlock()

	if client, exists := pm.pools[name]; exists {
		return client, nil
	}

	client, err := NewClient(config)
	if err != nil {
		return nil, err
	}

	pm.pools[name] = client
	return client, nil
}

func (pm *PoolManager) CloseClient(name string) error {
	pm.mu.Lock()
	defer pm.mu.Unlock()

	if client, exists := pm.pools[name]; exists {
		delete(pm.pools, name)
		return client.Close()
	}

	return nil
}

func (pm *PoolManager) CloseAll() error {
	pm.mu.Lock()
	defer pm.mu.Unlock()

	var lastErr error
	for name, client := range pm.pools {
		if err := client.Close(); err != nil {
			lastErr = err
		}
		delete(pm.pools, name)
	}

	return lastErr
}

func (pm *PoolManager) Stats() map[string]PoolStats {
	pm.mu.RLock()
	defer pm.mu.RUnlock()

	stats := make(map[string]PoolStats)
	for name, client := range pm.pools {
		poolStats := client.PoolStats()
		stats[name] = PoolStats{
			Name:         name,
			Hits:         poolStats.Hits,
			Misses:       poolStats.Misses,
			Timeouts:     poolStats.Timeouts,
			TotalConns:   poolStats.TotalConns,
			IdleConns:    poolStats.IdleConns,
			StaleConns:   poolStats.StaleConns,
		}
	}

	return stats
}

type PoolStats struct {
	Name       string
	Hits       uint32
	Misses     uint32
	Timeouts   uint32
	TotalConns uint32
	IdleConns  uint32
	StaleConns uint32
}

func (pm *PoolManager) ListClients() []string {
	pm.mu.RLock()
	defer pm.mu.RUnlock()

	names := make([]string, 0, len(pm.pools))
	for name := range pm.pools {
		names = append(names, name)
	}
	return names
}

func (pm *PoolManager) HasClient(name string) bool {
	pm.mu.RLock()
	defer pm.mu.RUnlock()

	_, exists := pm.pools[name]
	return exists
}
