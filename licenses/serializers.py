from rest_framework import serializers

from applications.models import Application

from .models import License


class LicenseSerializer(serializers.ModelSerializer):
    # Nom du tenant aplati : la console React affiche le nom, pas l'UUID.
    tenant_name = serializers.CharField(source='tenant.name', read_only=True)
    application_nom = serializers.SerializerMethodField()

    class Meta:
        model = License
        fields = [
            'id', 'tenant', 'tenant_name', 'plan', 'seats', 'starts_at',
            'expires_at', 'is_active', 'application', 'application_nom', 'created_at',
        ]
        read_only_fields = ['id', 'created_at']

    def get_application_nom(self, obj):
        return str(obj.application) if obj.application_id else None

    def validate_application(self, application):
        # Seule une application validée (image vérifiée dans le registre)
        # peut être rattachée ; `null` détache.
        if application is not None and application.statut != Application.VALIDEE:
            raise serializers.ValidationError(
                f"Application « {application} » non validée (statut : {application.statut})."
            )
        return application

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
