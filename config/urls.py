from django.contrib import admin
from django.urls import path, include
from django.contrib.auth import views as auth_views
from django.conf import settings
from django.conf.urls.static import static

urlpatterns = [
    path('admin/', admin.site.urls),
    path('login/', auth_views.LoginView.as_view(), name='login'),
    path('logout/', auth_views.LogoutView.as_view(), name='logout'),
    path('', include('dashboard.urls')),
    path('tenants/', include('tenants.urls')),
    path('licenses/', include('licenses.urls')),
    path('devices/', include('devices.urls')),
    path('api/tenants/', include('tenants.urls_api')),
    path('api/licenses/', include('licenses.urls_api')),
    path('api/devices/', include('devices.urls_api')),
] + static(settings.MEDIA_URL, document_root=settings.MEDIA_ROOT)
