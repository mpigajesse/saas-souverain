"""`seed_demo_public` — données fictives de la démo en ligne (portfolio).

Deux comptes, mot de passe `DEMO_PASSWORD` :
- `editeur` : vue éditeur (staff, sans superutilisateur) — comptes, licences, parc ;
- `pme`     : vue client, rattachée à la PME fictive « Atlas Distribution ».

Tout est fictif : entreprises, adresses, machines, identifiants d'installation.
Idempotente : si les PME de démo existent déjà, seuls les mots de passe sont remis
à jour. Sur Render, la base SQLite est éphémère : la démo revient donc à cet état
à chaque réveil du service.
"""

from __future__ import annotations

import uuid
from datetime import date, timedelta
from typing import Any

from django.conf import settings
from django.contrib.auth.models import User
from django.core.management.base import BaseCommand, CommandError
from django.db import transaction
from django.utils import timezone

from devices.models import Device
from licenses.models import License
from tenants.models import Tenant

# (nom, e-mail, ville, employés, plan, postes, machines [(hôte, OS, rôle, en ligne)])
PME_FICTIVES = [
    ("Atlas Distribution", "contact@atlas-distribution.example", "Casablanca", 42, "pro", 5, [
        ("atlas-srv-01", "Ubuntu 24.04", "primary", True),
        ("atlas-poste-compta", "Windows 11", "standby", True),
        ("atlas-poste-depot", "Windows 11", "standby", True),
    ]),
    ("Pharmacie du Littoral", "gerance@pharma-littoral.example", "Libreville", 9, "starter", 2, [
        ("littoral-caisse", "Windows 11", "primary", True),
        ("littoral-backoffice", "Windows 11", "standby", False),
    ]),
    ("Cabinet Nord Expertise", "it@nord-expertise.example", "Lille", 18, "pro", 3, [
        ("nord-srv", "Debian 12", "primary", True),
        ("nord-poste-1", "Windows 11", "standby", True),
        ("nord-poste-2", "Windows 11", "standby", True),
        ("nord-poste-3", "Windows 11", "standby", True),  # au-delà de la licence : alerte
    ]),
    ("Transports Ogooué", "dsi@transports-ogooue.example", "Port-Gentil", 120, "enterprise", 10, []),
]


class Command(BaseCommand):
    help = "Crée les comptes et données fictives de la démo publique (idempotent)."

    @transaction.atomic
    def handle(self, *args: Any, **options: Any) -> None:
        mot_de_passe = settings.DEMO_PASSWORD
        if not mot_de_passe:
            raise CommandError("DEMO_PASSWORD manquant : mot de passe des comptes de démo.")

        editeur, _ = User.objects.get_or_create(username="editeur", defaults={"first_name": "Éditeur (démo)"})
        editeur.is_staff = True
        editeur.is_superuser = False
        editeur.set_password(mot_de_passe)
        editeur.save()

        pme, _ = User.objects.get_or_create(username="pme", defaults={"first_name": "Atlas (démo)"})
        pme.is_staff = False
        pme.is_superuser = False
        pme.set_password(mot_de_passe)
        pme.save()

        if Tenant.objects.filter(email__endswith=".example").exists():
            self.stdout.write("Démo déjà en place : mots de passe remis à jour.")
            return

        maintenant = timezone.now()
        for nom, email, ville, employes, plan, postes, machines in PME_FICTIVES:
            champs: dict[str, Any] = {}
            if nom == "Atlas Distribution" and settings.DEMO_REGISTRATION_TOKEN:
                champs["registration_token"] = uuid.UUID(settings.DEMO_REGISTRATION_TOKEN)
            tenant = Tenant.objects.create(
                name=nom, email=email, address=f"{ville} (adresse fictive)",
                phone="+000 00 00 00 00", employee_count=employes,
                user=pme if nom == "Atlas Distribution" else None, **champs,
            )
            License.objects.create(
                tenant=tenant, plan=plan, seats=postes,
                starts_at=date.today() - timedelta(days=120),
                expires_at=date.today() + timedelta(days=245),
            )
            for i, (hote, os_nom, role, en_ligne) in enumerate(machines):
                appareil = Device.objects.create(
                    tenant=tenant, installation_id=uuid.uuid4(), hostname=hote, os=os_nom,
                    node_role=role, node_addr=f"10.0.{len(nom)}.{10 + i}:9001",
                    web_addr=f"10.0.{len(nom)}.{10 + i}:3000",
                    streaming_standby_count=sum(1 for m in machines if m[2] == "standby" and m[3])
                    if role == "primary" else 0,
                )
                if not en_ligne:
                    # last_seen est en auto_now : update() contourne save().
                    Device.objects.filter(pk=appareil.pk).update(last_seen=maintenant - timedelta(hours=6))

        self.stdout.write(self.style.SUCCESS("Démo publique créée : comptes « editeur » et « pme »."))
