# ogis Helm chart

Deploy [OGIS](https://github.com/twangodev/ogis), the OpenGraph image generation service, to Kubernetes.

## Quick start

```bash
helm install ogis oci://ghcr.io/twangodev/charts/ogis
```

Or from a local checkout:

```bash
helm install ogis ./charts/ogis
```

## Common configurations

### Behind cert-manager + nginx

```yaml
ingress:
  enabled: true
  className: nginx
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
  hosts:
    - host: og.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: ogis-tls
      hosts: [og.example.com]
```

### Enable HMAC authentication

Bring your own Secret (recommended for production):

```bash
kubectl create secret generic ogis-hmac --from-literal=secret="$(openssl rand -hex 32)"

helm install ogis ./charts/ogis \
  --set hmac.enabled=true \
  --set hmac.existingSecret=ogis-hmac
```

Or let the chart manage it:

```bash
helm install ogis ./charts/ogis \
  --set hmac.enabled=true \
  --set hmac.value="$(openssl rand -hex 32)"
```

### Autoscale on CPU

```yaml
autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 20
  targetCPUUtilizationPercentage: 70
```

### Lock down egress

The in-process SSRF blocklist already blocks RFC1918 ranges. Enable the
NetworkPolicy as a k8s-side belt-and-suspenders (no-op on clusters without
a CNI that enforces it):

```yaml
networkPolicy:
  enabled: true
```

### Set OGIS_* options not surfaced in `values.yaml`

Anything from `src/config.rs` not exposed under `config:` can be set via
`extraEnv`:

```yaml
extraEnv:
  - name: OGIS_CACHE_MAX_BYTES
    value: "4GB"
  - name: OGIS_LOGO_TOTAL_TIMEOUT
    value: "15"
  - name: OGIS_OTEL_ENDPOINT
    value: https://otel-collector.observability.svc:4317
```

### Custom templates / fonts

The image bundles default templates and fonts at `/app/templates`,
`/app/fonts`, and `/app/gradients`. Override at runtime by mounting your
own ConfigMaps via `volumes` + `volumeMounts`. Fonts will not fit in a
single ConfigMap (1 MiB limit) — split per-family or use a PVC.

## Structure

This chart is generated from `helm create` and stays close to that
template. OGIS-specific additions:

- `templates/secret.yaml` — chart-managed HMAC Secret
- `templates/networkpolicy.yaml` — egress lockdown matching the SSRF blocklist
- `templates/pdb.yaml` — PodDisruptionBudget (auto-skipped at replicaCount=1)
- `cache.backend` in `values.yaml` — extension point for future Redis / memcached
- `hmac.*` and `config.*` blocks for OGIS-specific env wiring

See [`values.yaml`](values.yaml) for the full set of knobs.

## Uninstall

```bash
helm uninstall ogis
```
