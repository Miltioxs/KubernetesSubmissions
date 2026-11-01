## Ping-pong app

Create cluster `k3d cluster create --port 8082:30080@agent:0 -p 8081:80@loadbalancer --agents 2`

Deploy with `kubectl apply -f manifests`

Enter to ../log_output/ directory and run `kubectl apply -f manifests` 

Go to the Browser and type `http://localhost:8081/pingpong`

### Inspect stored data in postgres 

Execute a temporal pod with psql `kubectl run -it -n exercises --rm --restart=Never --image postgres psql-for-debugging sh`

Connect to postgres db `psql postgres://postgres:<pwd>@postgres-svc:5432/postgres`

`\l              -- list databases`
`\dt             -- list tables`
`\d table_name   -- show a table's columns`
`\q              -- quit`

select table `SELECT * FROM pingpong_counter;`

## Deploy Ping-pong application into GKE. 3.1

1. Create namespace exercises `kubectl create namespace exercises`
2. Deploy manifest `kubectl apply -f manifests`
3. Wait a moment, and then verify all is installed `kubectl get po -nexercises` `kubectl get svc -nexercises`
4. Use the IP of loadbalancer to check de aplication is response as expected `http://<replace with loadbalance-ip>/pingpong`