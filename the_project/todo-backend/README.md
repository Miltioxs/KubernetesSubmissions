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

