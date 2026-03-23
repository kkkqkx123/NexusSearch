package middleware

import (
	"fmt"
	"net/http"
	"runtime/debug"

	"github.com/gin-gonic/gin"
)

func RecoveryMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		defer func() {
			if err := recover(); err != nil {
				stack := debug.Stack()
				gin.DefaultWriter.Write([]byte(fmt.Sprintf("Panic recovered: %v\n%s", err, stack)))

				c.JSON(http.StatusInternalServerError, gin.H{
					"error":   "Internal server error",
					"details": "An unexpected error occurred",
				})
				c.Abort()
			}
		}()

		c.Next()
	}
}
