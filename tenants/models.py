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
    def licence_status(self):
        """Retourne un dict avec l'état de conformité licence."""
        active_license = self.licenses.filter(is_active=True).order_by('-created_at').first()
        active_devices = self.devices.filter(is_active=True).count()
        if active_license is None:
            return {'ok': False, 'reason': 'no_license', 'used': active_devices, 'total': 0}
        return {
            'ok': active_devices <= active_license.seats,
            'reason': 'over_limit' if active_devices > active_license.seats else None,
            'used': active_devices,
            'total': active_license.seats,
            'plan': active_license.get_plan_display(),
        }
