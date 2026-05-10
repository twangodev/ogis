{{/*
Expand the name of the chart.
*/}}
{{- define "ogis.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "ogis.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "ogis.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "ogis.labels" -}}
helm.sh/chart: {{ include "ogis.chart" . }}
{{ include "ogis.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "ogis.selectorLabels" -}}
app.kubernetes.io/name: {{ include "ogis.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "ogis.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "ogis.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
HMAC secret name — user-provided existingSecret if set, else chart-managed.
*/}}
{{- define "ogis.hmacSecretName" -}}
{{- default (printf "%s-hmac" (include "ogis.fullname" .)) .Values.hmac.existingSecret }}
{{- end }}

{{/*
Cache-backend env. Memory backend contributes nothing today. When Redis or
memcached lands, extend this single helper — that's the chart's promised
abstraction boundary for cache backends.
*/}}
{{- define "ogis.cacheEnv" -}}
{{- end }}

{{/*
Composed env list for the ogis container.
*/}}
{{- define "ogis.env" -}}
- name: RUST_LOG
  value: {{ .Values.config.logLevel | quote }}
{{- with .Values.config.defaults.title }}
- name: OGIS_DEFAULT_TITLE
  value: {{ . | quote }}
{{- end }}
{{- with .Values.config.defaults.description }}
- name: OGIS_DEFAULT_DESCRIPTION
  value: {{ . | quote }}
{{- end }}
{{- with .Values.config.defaults.subtitle }}
- name: OGIS_DEFAULT_SUBTITLE
  value: {{ . | quote }}
{{- end }}
{{- with .Values.config.defaults.logo }}
- name: OGIS_DEFAULT_LOGO
  value: {{ . | quote }}
{{- end }}
{{- if .Values.hmac.enabled }}
- name: OGIS_HMAC_SECRET
  valueFrom:
    secretKeyRef:
      name: {{ include "ogis.hmacSecretName" . }}
      key: {{ if .Values.hmac.existingSecret }}{{ .Values.hmac.existingSecretKey | default "secret" }}{{ else }}secret{{ end }}
{{- end }}
{{- include "ogis.cacheEnv" . }}
{{- with .Values.extraEnv }}
{{ toYaml . }}
{{- end }}
{{- end }}
