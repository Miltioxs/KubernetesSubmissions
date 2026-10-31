## Install Monitoring tools

### Run the following commands but before install helm and download repos

`helm repo add prometheus-community https://prometheus-community.github.io/helm-charts`
`helm repo add grafana https://grafana.github.io/helm-charts`
`helm repo update`

helm upgrade --install prom prometheus-community/prometheus \
  --namespace monitoring \
  --create-namespace \
  --values prom-values.yaml

helm upgrade --install loki grafana/loki \
  --namespace monitoring \
  --values loki-values.yaml

helm upgrade --install k8smon grafana/k8s-monitoring \
  --namespace monitoring \
  --values k8smon-values.yaml

helm upgrade --install grafana grafana/grafana \
  --namespace monitoring \
  --values grafana-values.yaml