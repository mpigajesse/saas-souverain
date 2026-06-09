from rest_framework.routers import DefaultRouter

from .views import LicenseViewSet

router = DefaultRouter()
router.register(r'', LicenseViewSet, basename='license-api')

urlpatterns = router.urls
