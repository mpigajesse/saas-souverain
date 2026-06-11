from django.urls import path

from . import views_web

urlpatterns = [
    path('', views_web.device_list, name='device-list'),
    path('clusters/', views_web.cluster_overview, name='cluster-overview'),
    path('<uuid:pk>/toggle/', views_web.device_toggle, name='device-toggle'),
]
