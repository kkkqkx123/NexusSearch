package cache

import (
	"context"
	"fmt"
	"log"
	"sort"
	"sync"
	"time"

	"github.com/redis/go-redis/v9"
)

type WarmupTask struct {
	Name     string
	Key      string
	Loader   func(ctx context.Context) (interface{}, error)
	Priority int
}

type CacheWarmer struct {
	tasks    []WarmupTask
	client   *redis.Client
	parallel int
	timeout  time.Duration
}

func NewCacheWarmer(client *redis.Client, parallel int, timeout time.Duration) *CacheWarmer {
	return &CacheWarmer{
		client:   client,
		parallel: parallel,
		timeout:  timeout,
		tasks:    make([]WarmupTask, 0),
	}
}

func (cw *CacheWarmer) AddTask(task WarmupTask) {
	cw.tasks = append(cw.tasks, task)
}

func (cw *CacheWarmer) AddTasks(tasks []WarmupTask) {
	cw.tasks = append(cw.tasks, tasks...)
}

func (cw *CacheWarmer) Warmup(ctx context.Context) error {
	sort.Slice(cw.tasks, func(i, j int) bool {
		return cw.tasks[i].Priority < cw.tasks[j].Priority
	})

	taskChan := make(chan WarmupTask, len(cw.tasks))
	errChan := make(chan error, len(cw.tasks))
	var wg sync.WaitGroup

	for i := 0; i < cw.parallel; i++ {
		wg.Add(1)
		go func(workerID int) {
			defer wg.Done()
			for task := range taskChan {
				if err := cw.executeTask(ctx, task); err != nil {
					errChan <- fmt.Errorf("worker %d: task %s failed: %w", workerID, task.Name, err)
				}
			}
		}(i)
	}

	for _, task := range cw.tasks {
		taskChan <- task
	}
	close(taskChan)

	wg.Wait()
	close(errChan)

	var errors []error
	for err := range errChan {
		errors = append(errors, err)
	}

	if len(errors) > 0 {
		return fmt.Errorf("warmup completed with %d errors", len(errors))
	}

	log.Printf("Cache warmup completed successfully: %d tasks", len(cw.tasks))
	return nil
}

func (cw *CacheWarmer) executeTask(ctx context.Context, task WarmupTask) error {
	start := time.Now()
	log.Printf("Starting warmup task: %s", task.Name)

	taskCtx, cancel := context.WithTimeout(ctx, cw.timeout)
	defer cancel()

	exists, err := cw.client.Exists(taskCtx, task.Key).Result()
	if err != nil {
		return fmt.Errorf("check cache existence failed: %w", err)
	}

	if exists > 0 {
		log.Printf("Warmup task %s skipped (cache hit)", task.Name)
		return nil
	}

	data, err := task.Loader(taskCtx)
	if err != nil {
		return fmt.Errorf("load data failed: %w", err)
	}

	if err := cw.client.Set(taskCtx, task.Key, data, 1*time.Hour).Err(); err != nil {
		return fmt.Errorf("set cache failed: %w", err)
	}

	duration := time.Since(start)
	log.Printf("Warmup task %s completed in %v", task.Name, duration)
	return nil
}

func (cw *CacheWarmer) ClearTasks() {
	cw.tasks = make([]WarmupTask, 0)
}

func (cw *CacheWarmer) TaskCount() int {
	return len(cw.tasks)
}
