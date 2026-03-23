.PHONY: proto clean build run test

proto:
	protoc --go_out=. --go_opt=paths=source_relative \
		--go-grpc_out=. --go-grpc_opt=paths=source_relative \
		proto/coordinator.proto

clean:
	rm -rf bin/
	go clean

build:
	go build -o bin/coordinator ./cmd/main.go

run:
	go run cmd/main.go

test:
	go test -v ./...

test-coverage:
	go test -v -coverprofile=coverage.out ./...
	go tool cover -html=coverage.out -o coverage.html

lint:
	golangci-lint run

fmt:
	go fmt ./...

tidy:
	go mod tidy

deps:
	go mod download

docker-build:
	docker build -t flexsearch-coordinator .

docker-run:
	docker-compose up -d

docker-stop:
	docker-compose down

docker-logs:
	docker-compose logs -f
