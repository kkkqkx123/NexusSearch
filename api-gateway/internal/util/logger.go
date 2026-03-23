package util

import (
	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
	"sync"
)

type Logger struct {
	*zap.Logger
	sugar *zap.SugaredLogger
	mu    sync.RWMutex
	level zapcore.Level
}

var (
	defaultLogger *Logger
	once          sync.Once
)

func NewLogger(level string, format string, output string) (*Logger, error) {
	var zapLevel zapcore.Level
	switch level {
	case "debug":
		zapLevel = zapcore.DebugLevel
	case "info":
		zapLevel = zapcore.InfoLevel
	case "warn", "warning":
		zapLevel = zapcore.WarnLevel
	case "error":
		zapLevel = zapcore.ErrorLevel
	case "fatal":
		zapLevel = zapcore.FatalLevel
	default:
		zapLevel = zapcore.InfoLevel
	}

	var config zap.Config
	if format == "json" {
		config = zap.NewProductionConfig()
	} else {
		config = zap.NewDevelopmentConfig()
		config.EncoderConfig.EncodeLevel = zapcore.CapitalColorLevelEncoder
	}

	config.Level = zap.NewAtomicLevelAt(zapLevel)
	config.OutputPaths = []string{output}
	config.ErrorOutputPaths = []string{output}

	config.EncoderConfig.EncodeTime = zapcore.ISO8601TimeEncoder
	config.EncoderConfig.EncodeDuration = zapcore.StringDurationEncoder

	logger, err := config.Build()
	if err != nil {
		return nil, err
	}

	l := &Logger{
		Logger: logger,
		sugar:  logger.Sugar(),
		level:  zapLevel,
	}

	return l, nil
}

func (l *Logger) With(args ...interface{}) *zap.SugaredLogger {
	return l.sugar.With(args...)
}

func (l *Logger) Debug(args ...interface{}) {
	l.sugar.Debug(args...)
}

func (l *Logger) Info(args ...interface{}) {
	l.sugar.Info(args...)
}

func (l *Logger) Warn(args ...interface{}) {
	l.sugar.Warn(args...)
}

func (l *Logger) Error(args ...interface{}) {
	l.sugar.Error(args...)
}

func (l *Logger) Fatal(args ...interface{}) {
	l.sugar.Fatal(args...)
}

func (l *Logger) Debugf(template string, args ...interface{}) {
	l.sugar.Debugf(template, args...)
}

func (l *Logger) Infof(template string, args ...interface{}) {
	l.sugar.Infof(template, args...)
}

func (l *Logger) Warnf(template string, args ...interface{}) {
	l.sugar.Warnf(template, args...)
}

func (l *Logger) Errorf(template string, args ...interface{}) {
	l.sugar.Errorf(template, args...)
}

func (l *Logger) Fatalf(template string, args ...interface{}) {
	l.sugar.Fatalf(template, args...)
}

func (l *Logger) Fatalw(msg string, keysAndValues ...interface{}) {
	l.sugar.Fatalw(msg, keysAndValues...)
}

func (l *Logger) Errorw(msg string, keysAndValues ...interface{}) {
	l.sugar.Errorw(msg, keysAndValues...)
}

func (l *Logger) Warnw(msg string, keysAndValues ...interface{}) {
	l.sugar.Warnw(msg, keysAndValues...)
}

func (l *Logger) Infow(msg string, keysAndValues ...interface{}) {
	l.sugar.Infow(msg, keysAndValues...)
}

func (l *Logger) Debugw(msg string, keysAndValues ...interface{}) {
	l.sugar.Debugw(msg, keysAndValues...)
}

func (l *Logger) Sync() {
	l.sugar.Sync()
	l.Logger.Sync()
}

func (l *Logger) SetLevel(level string) error {
	l.mu.Lock()
	defer l.mu.Unlock()

	var zapLevel zapcore.Level
	switch level {
	case "debug":
		zapLevel = zapcore.DebugLevel
	case "info":
		zapLevel = zapcore.InfoLevel
	case "warn", "warning":
		zapLevel = zapcore.WarnLevel
	case "error":
		zapLevel = zapcore.ErrorLevel
	case "fatal":
		zapLevel = zapcore.FatalLevel
	default:
		return nil
	}

	l.level = zapLevel
	return nil
}

func GetDefaultLogger() *Logger {
	once.Do(func() {
		var err error
		defaultLogger, err = NewLogger("info", "json", "stdout")
		if err != nil {
			defaultLogger, _ = NewLogger("info", "json", "stdout")
		}
	})
	return defaultLogger
}

type RequestLogger struct {
	logger *Logger
}

func NewRequestLogger(logger *Logger) *RequestLogger {
	return &RequestLogger{logger: logger}
}

func (rl *RequestLogger) LogRequest(method, path, ip string, statusCode int, latency float64, requestID string) {
	rl.logger.Infow("HTTP request completed",
		"method", method,
		"path", path,
		"ip", ip,
		"status_code", statusCode,
		"latency_ms", latency,
		"request_id", requestID,
	)
}

func (rl *RequestLogger) LogError(method, path, ip string, err error, requestID string) {
	rl.logger.Errorw("HTTP request error",
		"method", method,
		"path", path,
		"ip", ip,
		"error", err.Error(),
		"request_id", requestID,
	)
}
