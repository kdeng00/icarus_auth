

# Getting Started
Copy the `.env.sample` file to `.env` and ensure that the variables are populated. This project
can be used with regular hosting or with docker. For the sake of getting up to speed quickly,
Docker will be covered. Make sure docker is running and your ssh identity has been loaded.

Build image
```
docker compose build
```

Start images
```
docker compose up -d --force-recreate
```

Bring it down
```
docker compose down -v
```

Pruning
```
docker system prune -a
```
