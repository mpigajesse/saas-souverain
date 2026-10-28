from rest_framework import serializers
from .models import Tenant


class TenantSerializer(serializers.ModelSerializer):
    class Meta:
        model = Tenant
        fields = [
            'id', 'name', 'email', 'phone', 'address',
            'employee_count', 'created_at', 'is_active',
            # Jeton que l'editeur communique a la PME pour l'installation du
            # noeud : affiche dans la fiche tenant, jamais modifiable via l'API.
            'registration_token',
        ]
        read_only_fields = ['id', 'created_at', 'registration_token']
