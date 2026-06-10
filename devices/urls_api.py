from django.urls import path, include
from rest_framework.routers import DefaultRouter

from .views import DeviceViewSet
from .api_register import device_register
from .api_cluster_status import cluster_status

router = DefaultRouter()
router.register(r'', DeviceViewSet, basename='device-api')

urlpatterns = [
    path('register/', device_register, name='device-register'),
    path('cluster-status/', cluster_status, name='device-cluster-status'),
    path('', include(router.urls)),
]
