.PHONY: up down ps logs-redpanda clean prune health

RedPanda=docker/single.docker-compose.yml

# ======================== Cleanup ========================

clean:
	@echo "🧹 Cleaning project data..."
	rm -rf docker/redpanda

prune:
	@echo "💣 Pruning Docker system (DANGEROUS)..."
	docker system prune -a -f

# ======================== Redpanda ========================

up:
	@echo "🐳 Starting Redpanda..."
	mkdir -p docker/redpanda
	chmod -R 777 docker/redpanda
	docker compose -f $(RedPanda) up --force-recreate -d
	@echo "✅ Redpanda started"

down:
	@echo "🛑 Stopping Redpanda..."
	docker compose -f $(RedPanda) down
	@echo "✅ Stopped"

ps:
	@echo "📋 Container status:"
	docker ps -a --filter "name=redpanda"

logs-redpanda:
	@echo "📝 Redpanda logs:"
	docker compose -f $(RedPanda) logs -f redpanda

health:
	@echo "⏳ Checking Redpanda health..."
	rpk cluster info --brokers localhost:19092 || echo "❌ Not ready"

version:
	@echo "📦 Redpanda version:"
	rpk version

test-topic:
	@echo "🔍 Testing topic creation..."
	rpk topic create test-topic --brokers localhost:19092 || echo "❌ Failed to create topic"

list-topics:
	@echo "📋 Listing topics:"
	rpk topic list --brokers localhost:19092