from django.contrib import admin

from .models import Application


@admin.register(Application)
class ApplicationAdmin(admin.ModelAdmin):
    list_display = ['nom', 'version', 'editeur', 'statut', 'validee_le', 'created_at']
    list_filter = ['statut']
    search_fields = ['nom', 'editeur']
    # Le statut ne change que par l'API (validation contre le registre).
    readonly_fields = ['id', 'statut', 'validee_le', 'created_at', 'updated_at']
