import uuid
from django.db import models
from tenants.models import Tenant


class Device(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid.uuid4, editable=False)
    tenant = models.ForeignKey(Tenant, on_delete=models.CASCADE, related_name='devices')
    installation_id = models.UUIDField(unique=True)
    mac_address = models.CharField(max_length=17, blank=True)
    hostname = models.CharField(max_length=200, blank=True)
    os = models.CharField(max_length=50, blank=True)
    node_addr = models.CharField(max_length=255, blank=True, help_text="IP:port annoncée par le nœud (ex: 192.168.200.130:9001)")
    web_addr = models.CharField(max_length=255, blank=True, help_text="IP:port de l'interface web PME (ex: 192.168.200.130:3000)")
    node_role = models.CharField(
        max_length=10, blank=True,
        choices=[('primary', 'Primaire'), ('standby', 'Standby')],
        help_text='Rôle PostgreSQL de ce nœud dans le cluster PME',
    )
    registered_at = models.DateTimeField(auto_now_add=True)
    last_seen = models.DateTimeField(auto_now=True)
    is_active = models.BooleanField(default=True)

    class Meta:
        ordering = ['-registered_at']

    def __str__(self) -> str:
        return f"{self.hostname or self.installation_id} ({self.tenant.name})"

    @property
    def is_within_license(self) -> bool:
        active_license = (
            self.tenant.licenses
            .filter(is_active=True)
            .order_by('-created_at')
            .first()
        )
        if active_license is None:
            return False
        active_device_count = (
            self.tenant.devices
            .filter(is_active=True)
            .count()
        )
        return active_device_count <= active_license.seats
