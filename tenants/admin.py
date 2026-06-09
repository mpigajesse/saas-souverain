from django.contrib import admin
from .models import Tenant


@admin.register(Tenant)
class TenantAdmin(admin.ModelAdmin):
    list_display = ['name', 'email', 'employee_count', 'is_active', 'created_at']
    list_filter = ['is_active']
    search_fields = ['name', 'email']
    readonly_fields = ['id', 'created_at', 'registration_token']
