from django.urls import include, path
from rest_framework.routers import DefaultRouter

from .views import TenantViewSet
from . import api_espace, views_web

router = DefaultRouter()
router.register(r'', TenantViewSet, basename='tenant-api')

urlpatterns = [
    # Espace PME — doit être déclaré avant le router : sinon 'me' serait
    # capturé comme <pk> par la route de détail du ViewSet.
    path('me/', api_espace.tenant_me, name='api-tenant-me'),
    path('me/onboarding/', api_espace.tenant_onboarding, name='api-tenant-onboarding'),
    path('me/download/compose/', api_espace.download_compose, name='api-tenant-download-compose'),
    path('me/download/installer/<str:os_type>/', api_espace.download_installer, name='api-tenant-download-installer'),
    # Voie PME : authentifiée par jeton d'enregistrement, pas par session —
    # les installeurs tournent sur une machine sans compte éditeur. Déclarée
    # avant le router, sinon 'pme' serait capturé comme <pk> par la route de
    # détail du ViewSet.
    path('pme/compose/', views_web.compose_telecharger_pme, name='api-tenant-compose-pme'),
    path('', include(router.urls)),
]
