## Ping-pong app

Create cluster `k3d cluster create --port 8082:30080@agent:0 -p 8081:80@loadbalancer --agents 2`

Deploy with `kubectl apply -f manifests`

Enter to ../the_project/ directory and run `kubectl apply -f manifests`

Go to the Browser and type `http://localhost:8081/pingpong`

