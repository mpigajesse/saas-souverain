from django.contrib import admin
from .models import License


@admin.register(License)
class LicenseAdmin(admin.ModelAdmin):
    list_display = ['tenant', 'plan', 'seats', 'starts_at', 'expires_at', 'is_active', 'created_at']
    list_filter = ['plan', 'is_active']
    search_fields = ['tenant__name']
    readonly_fields = ['id', 'created_at']
    raw_id_fields = ['tenant']
