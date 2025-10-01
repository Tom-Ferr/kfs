IMG = kfs-qemu

up:
	docker compose up -d

down:
	docker compose down

run: up
	@open "http://localhost:8080" > /dev/null 2>&1;

clean:
	docker exec -d qemu make clean
	docker compose down --rmi local

fclean:
	docker exec -d qemu make fclean
	docker compose down --rmi all --volumes
	docker system prune -f --filter label=${IMG}

local:
	@cd code && make run

re: clean up

.PHONY: up down clean fclean re run local