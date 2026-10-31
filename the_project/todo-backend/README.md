## TODO backend

Keeps the todo items in memory and is only reachable inside the cluster. The todo-app pod calls it through its Service.

### Endpoints

- `GET /todos` returns the list of todos, e.g. `[{"id":1,"content":"Learn Kubernetes basics"}]`
- `POST /todos` with `{"content":"..."}` creates a todo and answers `201` with it. Empty todos or todos over 140 characters get `400`

### Environment variables

- `PORT` port to listen on, defaults to `3000`

### Deploy 

Create cluster `k3d cluster create --port 8082:30080@agent:0 -p 8081:80@loadbalancer --agents 2`

Create local path in node `docker exec k3d-k3s-default-agent-0 mkdir -p /tmp/kube`

Enter to ../../local_storage and run `kubectl apply -f manifests`

Enter to ../the_project/ directory and run `kubectl apply -f manifests`

Go to todo-app directory and run `kubectl apply -f manifests`

Enter to todo-backend directory and run `kubectl apply -f manifests`

Go to the Browser and type `http://localhost:8081/`

### Install monitoring 

Go to ../monitoring/README.md and follow the instruccions to install monitoring stack

### Verify 2.10 exercise

1. `kubectl port-forward --namespace monitoring svc/grafana 3000:80`
2. `kubectl port-forward -n project svc/todo-backend-service 8083:80`
3. `curl -i -X POST localhost:8083/todos -H 'Content-Type: application/json' -d "{\"content\":\"$(printf 'a%.0s' {1..141})\"}"`
4. Enter http://127.0.0.1:3000 ## Grafana
5. Go to explore, select loki datasource and use code and paste `{namespace="project"}`
6. Execute/Run query.

### Debug Database 

Execute a temporal pod with psql `kubectl run -it -n project -n exercises --rm --restart=Never --image postgres psql-for-debugging sh`

Connect to postgres db `psql postgres://postgres:<pwd>@todo-postgres-svc:5432/postgres`

change to todos db `\c todos`

check tables `\dt`

select todos table `select * from todos;`

