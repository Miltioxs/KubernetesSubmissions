## Log output app

Create cluster `k3d cluster create --port 8082:30080@agent:0 -p 8081:80@loadbalancer --agents 2`

Create local path in node `docker exec k3d-k3s-default-agent-0 mkdir -p /tmp/kube`

Enter to ../local_storage and run `kubectl apply -f manifests`

return to log_output directory and deploy with `kubectl apply -f manifests`

Go to ../pingpong directory and deploy `kubectl apply -f manifests`

Enter to ../the_project/ directory and run `kubectl apply -f manifests`

Go to the Browser and type `http://localhost:8081/status` or `http://localhost:8081/pingpong`
