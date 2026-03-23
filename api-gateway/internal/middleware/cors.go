package middleware

import (
	"net/http"

	"github.com/gin-gonic/gin"
)

type CORSConfig struct {
	AllowOrigins     []string
	AllowMethods     []string
	AllowHeaders     []string
	AllowCredentials bool
}

func CORSMiddleware(config CORSConfig) gin.HandlerFunc {
	return func(c *gin.Context) {
		origin := c.Request.Header.Get("Origin")

		if len(config.AllowOrigins) > 0 {
			allowed := false
			var allowedOrigin string
			for _, allowedOrigin = range config.AllowOrigins {
				if allowedOrigin == "*" {
					allowed = true
					break
				}
				if allowedOrigin == origin {
					allowed = true
					break
				}
			}
			if allowed {
				if allowedOrigin == "*" {
					if origin != "" {
						c.Writer.Header().Set("Access-Control-Allow-Origin", origin)
					} else {
						c.Writer.Header().Set("Access-Control-Allow-Origin", "*")
					}
				} else {
					c.Writer.Header().Set("Access-Control-Allow-Origin", origin)
				}
			}
		} else {
			c.Writer.Header().Set("Access-Control-Allow-Origin", "*")
		}

		if config.AllowCredentials {
			c.Writer.Header().Set("Access-Control-Allow-Credentials", "true")
		}

		if len(config.AllowHeaders) > 0 {
			headers := ""
			for i, header := range config.AllowHeaders {
				if i > 0 {
					headers += ", "
				}
				headers += header
			}
			c.Writer.Header().Set("Access-Control-Allow-Headers", headers)
		} else {
			c.Writer.Header().Set("Access-Control-Allow-Headers", "Content-Type, Content-Length, Accept-Encoding, X-CSRF-Token, Authorization, accept, origin, Cache-Control, X-Requested-With")
		}

		if len(config.AllowMethods) > 0 {
			methods := ""
			for i, method := range config.AllowMethods {
				if i > 0 {
					methods += ", "
				}
				methods += method
			}
			c.Writer.Header().Set("Access-Control-Allow-Methods", methods)
		} else {
			c.Writer.Header().Set("Access-Control-Allow-Methods", "POST, OPTIONS, GET, PUT, DELETE")
		}

		if c.Request.Method == "OPTIONS" {
			c.AbortWithStatus(http.StatusNoContent)
			return
		}

		c.Next()
	}
}
