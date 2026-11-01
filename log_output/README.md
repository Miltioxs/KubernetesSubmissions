## Log output app

Create cluster `k3d cluster create --port 8082:30080@agent:0 -p 8081:80@loadbalancer --agents 2`

Create local path in node `docker exec k3d-k3s-default-agent-0 mkdir -p /tmp/kube`

Enter to ../local_storage and run `kubectl apply -f manifests`

return to log_output directory and deploy with `kubectl apply -f manifests`

Go to ../pingpong directory and deploy `kubectl apply -f manifests`

Go to the Browser and type `http://localhost:8081/status` or `http://localhost:8081/pingpong`


## Deploy in GKE Cluster

0. Create namespace exercises `kubectl create namespace exercises`
1. Deploy pvc ../local_storage/pvc-exercises.yaml `kubectl apply -f pvc-exercises.yaml`
2. Deploy log_ouput `kubectl apply -f manifests` here I am deploying the ingress for log_output and pingpong 
3. Deploy pingpong ../pingpong/manifests `kubectl apply -f manifests`
4. Test paths `http://<ingress-address>/status` for log_output and `http://<ingress-address>/pingpong` for pinpong 