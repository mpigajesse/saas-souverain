from django.contrib import admin
from .models import Device


@admin.register(Device)
class DeviceAdmin(admin.ModelAdmin):
    list_display = ['hostname', 'tenant', 'os', 'is_active', 'last_seen', 'registered_at']
    list_filter = ['is_active', 'os']
    search_fields = ['hostname', 'installation_id', 'tenant__name']
    readonly_fields = ['id', 'registered_at', 'last_seen']
    raw_id_fields = ['tenant']
