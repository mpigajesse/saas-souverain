from django.urls import path

from . import views, views_relay, views_architecture

urlpatterns = [
    path('', views.dashboard, name='dashboard'),
    path('relay/', views_relay.relay_monitor, name='relay-monitor'),
    path('architecture/', views_architecture.architecture, name='architecture'),
    path('schema/', views_architecture.schema_soutenance, name='schema-soutenance'),
]
