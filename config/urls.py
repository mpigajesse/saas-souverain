from django.contrib import admin
from django.urls import path, include
from django.contrib.auth import views as auth_views
from django.conf import settings
from django.conf.urls.static import static

# En démo publique, l'administration Django est retirée : un visiteur pourrait
# y changer le mot de passe du compte de démo et bloquer les suivants.
urlpatterns = [] if settings.DEMO_PUBLIQUE else [path('admin/', admin.site.urls)]

urlpatterns += [
    path('login/', auth_views.LoginView.as_view(
        extra_context={'demo_password': settings.DEMO_PASSWORD},
    ), name='login'),
    path('logout/', auth_views.LogoutView.as_view(), name='logout'),
    path('', include('dashboard.urls')),
    path('tenants/', include('tenants.urls')),
    path('licenses/', include('licenses.urls')),
    path('devices/', include('devices.urls')),
    path('api/tenants/', include('tenants.urls_api')),
    path('api/licenses/', include('licenses.urls_api')),
    path('api/devices/', include('devices.urls_api')),
] + static(settings.MEDIA_URL, document_root=settings.MEDIA_ROOT)
