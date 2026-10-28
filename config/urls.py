from django.contrib import admin
from django.urls import path, include
from django.conf import settings
from django.conf.urls.static import static

from dashboard.api_stats import dashboard_stats
from dashboard.api_relay import relay_status_view
from dashboard.api_images import (
    node_image_status,
    node_image_download,
    node_image_telecharger_pme,
)
from devices.api_clusters import clusters_view

urlpatterns = [
    path('admin/', admin.site.urls),

    # API REST — console React (session + CSRF). Voir CONTRAT-API.md.
    # La console HTML Django a été retirée en V3 : seules l'API et l'admin
    # Django sont servies par le backend.
    path('api/auth/', include('api.urls')),
    path('api/dashboard/stats/', dashboard_stats, name='api-dashboard-stats'),
    path('api/clusters/', clusters_view, name='api-clusters'),
    path('api/relay/status/', relay_status_view, name='api-relay-status'),
    path('api/node-image/', node_image_status, name='api-node-image'),
    path('api/node-image/download/', node_image_download, name='api-node-image-download'),
    # Voie PME : authentifiée par jeton d'enregistrement, pas par session —
    # l'installeur tourne sur une machine sans compte éditeur.
    path('api/node-image/telecharger/', node_image_telecharger_pme, name='api-node-image-pme'),
    path('api/tenants/', include('tenants.urls_api')),
    path('api/licenses/', include('licenses.urls_api')),
    path('api/devices/', include('devices.urls_api')),
] + static(settings.MEDIA_URL, document_root=settings.MEDIA_ROOT)
