from django.urls import path

from . import views_web

urlpatterns = [
    path('', views_web.license_list, name='license-list'),
    path('nouvelle/', views_web.license_create, name='license-create'),
    path('<uuid:pk>/modifier/', views_web.license_update, name='license-update'),
    path('<uuid:pk>/supprimer/', views_web.license_delete, name='license-delete'),
]
