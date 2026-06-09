import os
import sys
import django

os.environ.setdefault('DJANGO_SETTINGS_MODULE', 'config.settings')
django.setup()

from django.db import connection
from django.db.migrations.executor import MigrationExecutor

RESET  = '\033[0m'
GREEN  = '\033[92m'
RED    = '\033[91m'
YELLOW = '\033[93m'

def ok(msg):  print(f"  {GREEN}OK{RESET}  {msg}")
def fail(msg): print(f"  {RED}FAIL{RESET} {msg}"); return False
def section(title): print(f"\n{YELLOW}{title}{RESET}")

all_ok = True

# ── 1. Connexion PostgreSQL ──────────────────────────────────────────────────
section("1. Connexion PostgreSQL")
try:
    with connection.cursor() as cur:
        cur.execute("SELECT version();")
        version = cur.fetchone()[0].split(',')[0]
    ok(version)
except Exception as e:
    all_ok = fail(f"Connexion échouée : {e}")

# ── 2. Migrations appliquées ─────────────────────────────────────────────────
section("2. Migrations")
executor = MigrationExecutor(connection)
plan = executor.migration_plan(executor.loader.graph.leaf_nodes())
if plan:
    for migration, _ in plan:
        all_ok = fail(f"Non appliquée : {migration}")
else:
    ok("Toutes les migrations sont appliquées")

# ── 3. Tables existantes ─────────────────────────────────────────────────────
section("3. Tables")
with connection.cursor() as cur:
    cur.execute("""
        SELECT tablename FROM pg_tables
        WHERE schemaname = 'public'
        ORDER BY tablename;
    """)
    tables = [r[0] for r in cur.fetchall()]

expected = ['tenants_tenant', 'licenses_license', 'devices_device']
for t in expected:
    if t in tables:
        ok(t)
    else:
        all_ok = fail(f"Table manquante : {t}")

# ── 4. Modèles Django ────────────────────────────────────────────────────────
section("4. Modèles ORM")
try:
    from tenants.models import Tenant
    ok(f"Tenant — champs : {[f.name for f in Tenant._meta.get_fields()]}")
except Exception as e:
    all_ok = fail(f"Tenant : {e}")

try:
    from licenses.models import License
    ok(f"License — champs : {[f.name for f in License._meta.get_fields()]}")
except Exception as e:
    all_ok = fail(f"License : {e}")

try:
    from devices.models import Device
    ok(f"Device — champs : {[f.name for f in Device._meta.get_fields()]}")
except Exception as e:
    all_ok = fail(f"Device : {e}")

# ── 5. Écriture / lecture DB ─────────────────────────────────────────────────
section("5. Écriture / lecture DB")
try:
    from tenants.models import Tenant
    t = Tenant.objects.create(
        name="PME Test",
        email="test@verify.local",
        employee_count=3,
    )
    ok(f"Tenant créé  id={t.id}")
    found = Tenant.objects.get(pk=t.id)
    ok(f"Tenant lu    name={found.name}")
    t.delete()
    ok("Tenant supprimé (cleanup)")
except Exception as e:
    all_ok = fail(f"CRUD Tenant : {e}")

# ── 6. URLs / sérialiseurs ───────────────────────────────────────────────────
section("6. Sérialiseurs")
try:
    from tenants.serializers import TenantSerializer
    from licenses.serializers import LicenseSerializer
    from devices.serializers import DeviceSerializer
    ok("TenantSerializer, LicenseSerializer, DeviceSerializer importés")
except Exception as e:
    all_ok = fail(f"Sérialiseurs : {e}")

# ── Résultat final ───────────────────────────────────────────────────────────
print()
if all_ok:
    print(f"{GREEN}✓ Tout est opérationnel.{RESET}")
else:
    print(f"{RED}✗ Des problèmes ont été détectés — voir ci-dessus.{RESET}")
    sys.exit(1)
