from rest_framework import serializers
from .models import Device


class DeviceSerializer(serializers.ModelSerializer):
    is_within_license = serializers.BooleanField(read_only=True)
    tenant_name = serializers.CharField(source='tenant.name', read_only=True)

    class Meta:
        model = Device
        fields = [
            'id', 'tenant', 'tenant_name', 'installation_id', 'mac_address',
            'hostname', 'os', 'node_role', 'node_kind', 'node_addr', 'web_addr',
            'registered_at', 'last_seen', 'is_active', 'is_within_license',
        ]
        read_only_fields = ['id', 'registered_at', 'last_seen']
