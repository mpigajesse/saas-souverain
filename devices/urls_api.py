from django.urls import path, include
from rest_framework.routers import DefaultRouter

from .views import DeviceViewSet
from .api_register import device_register

router = DefaultRouter()
router.register(r'', DeviceViewSet, basename='device-api')

urlpatterns = [
    path('register/', device_register, name='device-register'),
    path('', include(router.urls)),
]
