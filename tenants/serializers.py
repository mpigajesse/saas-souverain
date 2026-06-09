from rest_framework import serializers
from .models import Tenant


class TenantSerializer(serializers.ModelSerializer):
    class Meta:
        model = Tenant
        fields = [
            'id', 'name', 'email', 'phone', 'address',
            'employee_count', 'created_at', 'is_active',
        ]
        read_only_fields = ['id', 'created_at']
