docker run --name pg-dev \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=banking \
  -p 5432:5432 \
  -d postgres:16