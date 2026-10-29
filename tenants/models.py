import uuid
from django.contrib.auth.models import User
from django.db import models


class Tenant(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid.uuid4, editable=False)
    registration_token = models.UUIDField(default=uuid.uuid4, unique=True, editable=False)
    user = models.OneToOneField(
        User,
        null=True,
        blank=True,
        on_delete=models.SET_NULL,
        related_name='tenant',
    )
    name = models.CharField(max_length=200)
    email = models.EmailField(unique=True)
    phone = models.CharField(max_length=20, blank=True)
    address = models.TextField(blank=True)
    employee_count = models.PositiveIntegerField()
    created_at = models.DateTimeField(auto_now_add=True)
    is_active = models.BooleanField(default=True)

    class Meta:
        ordering = ['-created_at']

    def __str__(self) -> str:
        return self.name

    @property
    def etat_cluster(self):
        """Pire état du cluster remonté par les serveurs en ligne (étape 2d) :
        « perdu », « sous_le_seuil » ou « sain » ; None sans remontée récente.
        Un nœud hors ligne depuis plus de 2 minutes n'est pas pris en compte."""
        from datetime import timedelta

        from django.utils import timezone

        recents = self.devices.filter(
            is_active=True, node_kind='serveur', last_seen__gte=timezone.now() - timedelta(minutes=2),
        ).exclude(etat_quorum='').values_list('etat_quorum', flat=True)
        for etat in ('perdu', 'sous_le_seuil', 'sain'):
            if etat in recents:
                return etat
        return None

    @property
    def licence_status(self):
        """Retourne un dict avec l'état de conformité licence."""
        active_license = self.licenses.filter(is_active=True).order_by('-created_at').first()
        # Un témoin etcd ne consomme pas de poste (ADR-005).
        active_devices = self.devices.filter(is_active=True).exclude(node_kind='temoin').count()
        if active_license is None:
            return {'ok': False, 'reason': 'no_license', 'used': active_devices, 'total': 0}
        return {
            'ok': active_devices <= active_license.seats,
            'reason': 'over_limit' if active_devices > active_license.seats else None,
            'used': active_devices,
            'total': active_license.seats,
            'plan': active_license.get_plan_display(),
        }
