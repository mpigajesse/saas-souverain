from rest_framework import serializers

from tenants.manifeste import ManifesteInvalide, valider

from .models import Application


class ApplicationSerializer(serializers.ModelSerializer):
    type_artefact = serializers.CharField(read_only=True)
    image = serializers.CharField(read_only=True)
    # PME qui l'hébergent (licences actives qui la désignent).
    licences_actives = serializers.SerializerMethodField()

    class Meta:
        model = Application
        fields = [
            'id', 'nom', 'editeur', 'version', 'manifeste', 'type_artefact',
            'image', 'statut', 'validee_le', 'licences_actives',
            'created_at', 'updated_at',
        ]
        read_only_fields = ['id', 'statut', 'validee_le', 'created_at', 'updated_at']

    def get_licences_actives(self, obj) -> int:
        return obj.licences.filter(is_active=True).count()

    def validate_manifeste(self, valeur):
        try:
            normalise = valider(valeur)
        except ManifesteInvalide as e:
            raise serializers.ValidationError(str(e))
        if (self.instance is not None and self.instance.statut != Application.BROUILLON
                and normalise != self.instance.manifeste):
            raise serializers.ValidationError(
                "Manifeste figé : l'application a été validée. Créez une nouvelle version."
            )
        return normalise


class ApplicationResumeSerializer(serializers.ModelSerializer):
    """Ce qu'une PME voit de l'application qu'elle héberge."""

    class Meta:
        model = Application
        fields = ['id', 'nom', 'editeur', 'version', 'statut']
        read_only_fields = fields
