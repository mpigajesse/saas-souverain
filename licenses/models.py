import uuid
from django.db import models
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
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        ordering = ['-created_at']

    def __str__(self) -> str:
        return f"{self.tenant.name} — {self.plan} ({self.seats} seats)"
