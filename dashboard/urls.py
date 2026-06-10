from django.urls import path

from . import views, views_relay

urlpatterns = [
    path('', views.dashboard, name='dashboard'),
    path('relay/', views_relay.relay_monitor, name='relay-monitor'),
]
