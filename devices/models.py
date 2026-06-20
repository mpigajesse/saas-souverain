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
    last_failover_at = models.DateTimeField(
        null=True, blank=True,
        help_text='Dernier failover automatique détecté sur ce nœud',
    )
    failover_count = models.IntegerField(default=0)
    streaming_standby_count = models.IntegerField(
        default=0,
        help_text="Nb de standbys réellement en streaming, rapporté par le primaire "
                  "(pg_stat_replication). Source de vérité de la réplication.",
    )
    epoch = models.IntegerField(
        default=0,
        help_text="Époque du nœud = timeline PostgreSQL (incrémenté à chaque promotion). "
                  "Sert au fencing : un primaire avec une époque périmée est clôturé.",
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


class ClusterMetricSample(models.Model):
    """
    Échantillon de métriques techniques capturé à chaque heartbeat d'un nœud.

    Constitue la série temporelle (Big Data) exploitée par l'agent de
    supervision IA. Ne contient AUCUNE donnée métier — uniquement des
    métriques d'infrastructure — ce qui préserve la promesse zero-knowledge.
    """
    tenant = models.ForeignKey(Tenant, on_delete=models.CASCADE, related_name='metric_samples')
    device = models.ForeignKey(
        Device, on_delete=models.SET_NULL, null=True, blank=True,
        related_name='metric_samples',
    )
    captured_at = models.DateTimeField(auto_now_add=True, db_index=True)
    node_role = models.CharField(max_length=10, blank=True)
    streaming_standby_count = models.IntegerField(default=0)
    failover_count = models.IntegerField(default=0)

    class Meta:
        ordering = ['-captured_at']
        indexes = [
            models.Index(fields=['tenant', '-captured_at']),
        ]

    def __str__(self) -> str:
        return f"{self.tenant.name} · {self.node_role or '?'} @ {self.captured_at:%d/%m %H:%M:%S}"


class AgentVerdict(models.Model):
    """
    Résultat horodaté d'une analyse de l'agent de supervision IA pour un cluster.

    Persisté pour alimenter la page « Agent de supervision » et l'export de
    rapport. Ne contient que des métriques techniques + le diagnostic — jamais
    de donnée métier (promesse zero-knowledge).
    """
    RISK_CHOICES = [
        ('sain', 'Sain'),
        ('surveiller', 'À surveiller'),
        ('critique', 'Critique'),
    ]
    SOURCE_CHOICES = [
        ('mistral', 'Mistral AI'),
        ('local', 'Analyse locale (repli)'),
    ]

    tenant = models.ForeignKey(Tenant, on_delete=models.CASCADE, related_name='agent_verdicts')
    created_at = models.DateTimeField(auto_now_add=True, db_index=True)
    health = models.CharField(max_length=20, blank=True)
    risk_level = models.CharField(max_length=12, choices=RISK_CHOICES, default='surveiller')
    anomaly_score = models.IntegerField(default=0)
    summary = models.TextField(blank=True)
    recommendation = models.TextField(blank=True)
    details = models.TextField(blank=True)
    source = models.CharField(max_length=12, choices=SOURCE_CHOICES, default='local')
    features = models.JSONField(default=dict, blank=True)

    class Meta:
        ordering = ['-created_at']
        indexes = [
            models.Index(fields=['tenant', '-created_at']),
        ]

    def __str__(self) -> str:
        return f"{self.tenant.name} · {self.risk_level} ({self.anomaly_score}) @ {self.created_at:%d/%m %H:%M}"
