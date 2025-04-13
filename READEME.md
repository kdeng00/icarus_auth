

# Getting started
Take notice of the .env.sample file and create copies without the .sample in the name.

`.env.sample` -> `.env`

Ensure that all variables are populated and is correct.

## Docker

Build the images
```
docker compose build --ssh default auth_api
```

Bring it up
```
docker compose up -d --force-recreate auth_api
```