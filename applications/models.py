"""Catalogue des applications externes hébergées par AMANE (refonte V3,
étape 5, ADR-009).

Une application est un SaaS d'un éditeur tiers, livré en image Docker épinglée
par digest et décrit par son manifeste (`tenants.manifeste`). Elle est créée
par le personnel de l'éditeur AMANE, validée (manifeste conforme, image
présente dans le registre), puis rattachée à la licence des PME qui
l'hébergent. Le portail ne stocke que ce manifeste : aucune donnée de la PME,
aucun secret de sa base (ADR-003).
"""
import uuid

from django.db import models


class Application(models.Model):
    BROUILLON = 'brouillon'
    VALIDEE = 'validee'
    RETIREE = 'retiree'
    STATUTS = [
        (BROUILLON, 'Brouillon'),
        (VALIDEE, 'Validée'),
        (RETIREE, 'Retirée'),
    ]

    id = models.UUIDField(primary_key=True, default=uuid.uuid4, editable=False)
    nom = models.CharField(max_length=120)
    editeur = models.CharField(max_length=120, help_text="Éditeur du SaaS externe")
    version = models.CharField(max_length=40)
    # Forme normalisée renvoyée par `tenants.manifeste.valider`. Immuable une
    # fois l'application validée : changer d'image, c'est une nouvelle version.
    manifeste = models.JSONField()
    statut = models.CharField(max_length=10, choices=STATUTS, default=BROUILLON)
    validee_le = models.DateTimeField(null=True, blank=True)
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        ordering = ['nom', '-created_at']
        constraints = [
            models.UniqueConstraint(fields=['nom', 'version'], name='application_nom_version_unique'),
        ]

    def __str__(self) -> str:
        return f'{self.nom} {self.version}'

    @property
    def type_artefact(self) -> str:
        return self.manifeste.get('type_artefact', 'docker')

    @property
    def image(self) -> str:
        return self.manifeste.get('image', '')
