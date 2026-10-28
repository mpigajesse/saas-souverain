from django.urls import path

from . import auth

urlpatterns = [
    path('csrf/', auth.csrf_view, name='api-auth-csrf'),
    path('login/', auth.login_view, name='api-auth-login'),
    path('logout/', auth.logout_view, name='api-auth-logout'),
    path('me/', auth.me_view, name='api-auth-me'),
    path('register/', auth.register_view, name='api-auth-register'),
]
