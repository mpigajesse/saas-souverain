import psycopg2

# Connexion à la base postgres (maintenance), pas à saas_souverain
conn = psycopg2.connect(
    dbname='postgres',
    user='postgres',
    password='admin',
    host='localhost',
    port=5432,
)
conn.autocommit = True
cur = conn.cursor()

print("1. Terminaison de toutes les connexions sur saas_souverain...")
cur.execute("""
    SELECT pg_terminate_backend(pid)
    FROM pg_stat_activity
    WHERE datname = 'saas_souverain'
      AND pid <> pg_backend_pid();
""")
n = len(cur.fetchall())
print(f"   {n} connexion(s) terminée(s)")

print("2. Suppression de saas_souverain...")
cur.execute("DROP DATABASE IF EXISTS saas_souverain;")
print("   OK")

print("3. Recréation de saas_souverain...")
cur.execute("CREATE DATABASE saas_souverain OWNER postgres;")
print("   OK")

cur.close()
conn.close()
print("\nBase recréée proprement. Lance maintenant : python manage.py migrate")
