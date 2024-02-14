SHELL = /bin/bash
.ONESHELL:
.DEFAULT_GOAL: help

help: ## Prints available commands
	@awk 'BEGIN {FS = ":.*##"; printf "Usage: make \033[36m<target>\033[0m\n"} /^[.a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-25s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

cargo.build:
	@docker-compose run rust cargo build --release

up: ## Start
	@docker-compose up -d nginx

down: ## Stop
	@docker-compose down

start.dev: ## Start the rinha in Dev
	@docker-compose up -d nginx

start.prod: ## Start the rinha in Prod
	@docker-compose -f docker-compose-prod.yml up -d nginx

docker.stats: ## Show docker stats
	@docker stats --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.MemPerc}}"

health.check: ## Check the stack is healthy
	@curl -v http://localhost:9999/clientes/1/extrato

stress.it: ## Run stress tests
	@sh stress-test/run-test.sh

docker.build: ## Build the docker image
	@docker build -t leandronsp/quokka --target prod .

docker.push: ## Push the docker image
	@docker push leandronsp/quokka
