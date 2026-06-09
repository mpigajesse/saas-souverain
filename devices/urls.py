from django.urls import path

from . import views_web

urlpatterns = [
    path('', views_web.device_list, name='device-list'),
    path('<uuid:pk>/toggle/', views_web.device_toggle, name='device-toggle'),
]
