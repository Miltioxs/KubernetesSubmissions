## TODO app

Create cluster `k3d cluster create --port 8082:30080@agent:0 -p 8081:80@loadbalancer --agents 2`

Deploy with `kubectl apply -f manifests`

### Deploy 

Deploy project namespace `kubectl apply -f namespaces/manifests`

Deploy pvc and pv `kubectl apply -f local_storage/manifests`

Deploy todo-app ingress `kubectl apply -f the_project/manifests`

Deploy todo app `kubectl apply -f the_project/todo-app/manifests`

Deploy todo backend `kubectl apply -f the_project/todo-backend/manifests`

Go to the Browser and type `http://localhost:8081/`