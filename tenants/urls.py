from django.urls import path

from . import views_web

urlpatterns = [
    path('', views_web.tenant_list, name='tenant-list'),
    path('inscription/', views_web.inscription, name='inscription'),
    path('bienvenue/', views_web.bienvenue, name='bienvenue'),
    path('telecharger/compose/', views_web.telecharger_compose, name='telecharger-compose'),
    path('telecharger/installeur/<str:os_type>/', views_web.telecharger_installeur, name='telecharger-installeur'),
    path('<uuid:pk>/', views_web.tenant_detail, name='tenant-detail'),
    path('nouveau/', views_web.tenant_create, name='tenant-create'),
    path('<uuid:pk>/modifier/', views_web.tenant_update, name='tenant-update'),
    path('<uuid:pk>/supprimer/', views_web.tenant_delete, name='tenant-delete'),
]
