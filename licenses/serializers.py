from rest_framework import serializers
from .models import License


class LicenseSerializer(serializers.ModelSerializer):
    class Meta:
        model = License
        fields = [
            'id', 'tenant', 'plan', 'seats', 'starts_at',
            'expires_at', 'is_active', 'created_at',
        ]
        read_only_fields = ['id', 'created_at']

    def validate(self, attrs: dict) -> dict:
        tenant = attrs.get('tenant') or getattr(self.instance, 'tenant', None)
        seats = attrs.get('seats') if 'seats' in attrs else getattr(self.instance, 'seats', None)

        if tenant is not None and seats is not None:
            if seats > tenant.employee_count:
                raise serializers.ValidationError(
                    {
                        'seats': (
                            f"seats ({seats}) cannot exceed the tenant's "
                            f"employee_count ({tenant.employee_count})."
                        )
                    }
                )
        return attrs
