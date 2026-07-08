# ============================================================================
# Kubernetes Commands
# ============================================================================

K8S_DIR := .workspace/infrastructure/k8s
K8S_DIR_ALT := infrastructure/k8s

.PHONY: k8s-apply k8s-delete k8s-status

k8s-apply:
	@echo "☸️ Applying Kubernetes manifests..."
	@if [ -d $(K8S_DIR) ]; then \
		kubectl apply -f $(K8S_DIR) 2>/dev/null; \
	elif [ -d $(K8S_DIR_ALT) ]; then \
		kubectl apply -f $(K8S_DIR_ALT) 2>/dev/null; \
	else \
		echo "  ⚠️  No k8s manifests directory found — create one at $(K8S_DIR)"; \
	fi

k8s-delete:
	@echo "☸️ Deleting Kubernetes resources..."
	@if [ -d $(K8S_DIR) ]; then \
		kubectl delete -f $(K8S_DIR) 2>/dev/null; \
	elif [ -d $(K8S_DIR_ALT) ]; then \
		kubectl delete -f $(K8S_DIR_ALT) 2>/dev/null; \
	else \
		echo "  ⚠️  No k8s manifests directory found"; \
	fi

k8s-status:
	@echo "☸️ Kubernetes cluster status..."
	@kubectl cluster-info 2>/dev/null || echo "  ⚠️  kubectl not configured — is a cluster running?"
