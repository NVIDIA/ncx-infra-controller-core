{{- define "machine-a-tron.name" -}}
machine-a-tron
{{- end -}}

{{- define "machine-a-tron.namespace" -}}
{{- default .Release.Namespace .Values.namespaceOverride -}}
{{- end -}}
