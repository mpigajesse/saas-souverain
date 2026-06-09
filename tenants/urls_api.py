from rest_framework.routers import DefaultRouter

from .views import TenantViewSet

router = DefaultRouter()
router.register(r'', TenantViewSet, basename='tenant-api')

urlpatterns = router.urls
