import uuid
from django.db import models
from applications.models import Application
from tenants.models import Tenant


class License(models.Model):
    PLAN_CHOICES = [
        ('starter', 'Starter'),
        ('pro', 'Pro'),
        ('enterprise', 'Enterprise'),
    ]

    id = models.UUIDField(primary_key=True, default=uuid.uuid4, editable=False)
    tenant = models.ForeignKey(Tenant, on_delete=models.CASCADE, related_name='licenses')
    plan = models.CharField(max_length=20, choices=PLAN_CHOICES)
    seats = models.PositiveIntegerField()
    starts_at = models.DateField()
    expires_at = models.DateField(null=True, blank=True)
    is_active = models.BooleanField(default=True)
    # Application externe hébergée par les machines de la PME (étape 5,
    # ADR-009). PROTECT : une application hébergée se retire, elle ne se
    # supprime pas.
    application = models.ForeignKey(
        Application, on_delete=models.PROTECT, null=True, blank=True,
        related_name='licences',
    )
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        ordering = ['-created_at']

    def __str__(self) -> str:
        return f"{self.tenant.name} — {self.plan} ({self.seats} seats)"
