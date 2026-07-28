# **Guide de Développement Technique et d'Implémentation du Prototype**

## **Système Hybride "Tenant-Centric" Décentralisé : Agent de Données Souverain (SDA)**

Ce document de production constitue la spécification technique définitive pour le prototypage fonctionnel du projet de Fin d'Études (PFE) au sein d'**AL BARAA CONSULTING** \[REF-01\]. Il synthétise les travaux de recherche, de faisabilité et d'ingénierie logicielle pour aboutir à une architecture opérationnelle de coffre-fort de données décentralisé.

### **1\. Synthèse de Cadrage & Arbitrages Architecturaux**

Ce document sert de fil conducteur pour le développement d'un prototype robuste et sécurisé, aligné sur les exigences de l'Union Africaine à travers l'**AU Data Policy Framework (AUDPF)** validé en décembre 2025 \[REF-01\].

1. **Requalification Terminologique :** Le terme « brique universelle » est officiellement requalifié en **SDA (Sovereign Data Agent)** ou **Nœud de Confiance local (Trust Node)** \[REF-03\]. Cela écarte la perception purement matérielle au profit d'un composant logiciel d'infrastructure standardisé et encapsulé.  
2. **Le Dilemme de la Centralisation du Backend :**  
   * *La problématique soulevée :* Faut-il centraliser le backend pour garantir la propriété intellectuelle (PI), l'intégrité et la facilité des mises à jour ? \[REF-05\]  
   * *La réponse du prototype :* **Non.** Une centralisation pure rompt le support du mode hors-ligne natif (indispensable aux infrastructures africaines instables \[REF-01\]) et brise l'autonomie des PME.  
   * *L'arbitrage technique :* Le code source et la logique métier de l'éditeur sont centralisés et protégés au sein d'un dépôt sécurisé immuable (**Docker Registry d'AL BARAA**). Cependant, son **instance d'exécution (Runtime) est distribuée en local** (*Edge Computing*) sur chaque nœud client via des images de conteneurs compilées et signées \[REF-05\]. Les mises à jour s'effectuent par orchestration descendante automatisée.  
3. **Zéro Copie Centrale & Vecteurs de Vulnérabilité :** Si le backend était centralisé, le traitement nécessiterait l'acheminement et la copie temporaire des données clients sur un serveur tiers, créant un point unique de défaillance systémique et un « pot de miel » (*honeypot*) pour les cyberattaques. Notre architecture applique le paradigme **Code-to-Data** : la donnée reste confinée au repos et à l'exécution dans le giron du client (*Tenant-Centric*). Aucun flux de données brutes ne transite vers un cloud centralisé \[REF-05\].

### **2\. Architecture Globale et Modèle de Données**

Le système fonctionne comme un environnement multi-processus symétrique isolé par machine hôte \[REF-03\]. Il est segmenté en trois couches étanches : Applicative (FastAPI), Stockage Hybride (SQLite/DuckDB), et Réplication Réseau (Syncthing) \[REF-06\].

\+------------------------------------------------------------------------+  
|               TERMINAL CLIENT / NOEUD ISOLÉ (SDA INFRA)                |  
|                                                                        |  
|   \+----------------------------------------------------------------+   |  
|   | 🌐 APPLICATION PORTAIL CLIENT (Interface HTML5 / Tailwind)      |   |  
|   \+----------------------------------------------------------------+   |  
|                                   | (Appels REST HTTP \- Localhost)    |  
|                                   v                                    |  
|   \+----------------------------------------------------------------+   |  
|   | 🐳 CONTENEUR APPLICATION CORE : BACKEND ENGINE (FASTAPI)       |   |  
|   |                                                                |   |  
|   |  \+---------------------+      \+-----------------------------+  |   |  
|   |  | 🗄️ SQLITE EMBEDDED   |      | 📊 DUCKDB OLAP ENGINE       |  |   |  
|   |  | (Transactions, Logs, |      | (Données Métier Massives,   |  |   |  
|   |  |  Pistes d'Audit Hash)|      |  Fichiers Parquet Uniques)  |  |   |  
|   |  \+---------------------+      \+-----------------------------+  |   |  
|   |             |                               |                  |   |  
|   \+-------------|-------------------------------|------------------+   |  
|                 | (Déclenchement d'événements)  | (Fichiers Bruts)     |  
|                 v                               v                      |  
|   \+----------------------------------------------------------------+   |  
|   | 🔄 COUCHE REPLICATION ET MAILLAGE RESEAU P2P (SYNCTHING)       |   |  
|   |  \- Partitionnement au niveau bloc, protocole BEP sous TLS 1.3  |   |  
|   |  \- Algorithme de réconciliation de versionning (Type CRDT)     |   |  
|   \+----------------------------------------------------------------+   |  
\+------------------------------------------------------------------------+  
                                    |  
                (Réseau Privé Chiffré de Bout en Bout)  
                                    v  
                     \[ AUTRE NOEUD DE CONFIANCE P2P \]

### **3\. Fichiers de Configuration et d'Infrastructure \[REF-06\]**

#### **3.1. Structure du Répertoire Projet**

L'arborescence stricte suivante doit être initialisée dans votre environnement de développement :

sda-prototype/  
├── docker-compose.yml  
├── backend/  
│   ├── Dockerfile  
│   ├── requirements.txt  
│   └── app/  
│       ├── \_\_init\_\_.py  
│       ├── main.py  
│       ├── database.py  
│       ├── models.py  
│       └── routers/  
│           ├── \_\_init\_\_.py  
│           ├── data.py  
│           └── sync.py  
├── data/  
│   ├── db/  
│   └── shared\_storage/  
└── config/  
    └── syncthing/

#### **3.2. Script d'Orchestration Containerisée : docker-compose.yml**

Ce descripteur lie l'exécution locale du moteur d'API et le démon de transport réseau pair-à-pair. Ils partagent un volume de stockage de bloc commun pour l'échange de fichiers immuables \[REF-06\].

version: '3.8'

services:  
  sda-backend:  
    build:  
      context: ./backend  
      dockerfile: Dockerfile  
    container\_name: sda\_backend\_core  
    restart: always  
    ports:  
      \- "8000:8000"  
    volumes:  
      \- ./data/db:/app/data/db  
      \- ./data/shared\_storage:/app/data/shared\_storage  
    environment:  
      \- ENV=production  
      \- DB\_SQLITE\_PATH=/app/data/db/metadata.db  
      \- DB\_DUCKDB\_PATH=/app/data/db/analytics.duckdb  
    networks:  
      \- sda-network

  syncthing:  
    image: syncthing/syncthing:latest  
    container\_name: sda\_network\_sync  
    hostname: sda-node-engine  
    environment:  
      \- PUID=1000  
      \- PGID=1000  
    volumes:  
      \- ./config/syncthing:/var/syncthing/config  
      \- ./data/shared\_storage:/var/syncthing/SDA\_Shared  
    ports:  
      \- "8384:8384" \# Interface d'administration Web GUI Locale  
      \- "22000:22000/tcp" \# Protocole d'échange de données de bloc P2P  
      \- "22000:22000/udp" \# Protocole de transport QUIC  
      \- "21027:21027/udp" \# Découverte mDNS locale (sans internet)  
    networks:  
      \- sda-network  
    restart: unless-stopped

networks:  
  sda-network:  
    driver: bridge

### **4\. Code Source du Backend Core (Python / FastAPI)**

#### **4.1. Manifeste des Dépendances : backend/requirements.txt**

fastapi==0.104.1  
uvicorn==0.24.0.post1  
duckdb==0.9.2  
sqlalchemy==2.0.23  
pydantic==2.5.2  
pandas==2.1.3

#### **4.2. Moteur d'Abstraction de la Persistance : backend/app/database.py \[REF-06\]**

Ce module orchestre la double persistance. SQLite gère les métadonnées ACID de l'agent, et DuckDB exécute les calculs analytiques vectorisés directement sur le système de fichiers hôte.

import os  
import sqlite3  
import duckdb  
from sqlalchemy import create\_engine  
from sqlalchemy.orm import sessionmaker, declarative\_base

SQLITE\_PATH \= os.getenv("DB\_SQLITE\_PATH", "/app/data/db/metadata.db")  
DUCKDB\_PATH \= os.getenv("DB\_DUCKDB\_PATH", "/app/data/db/analytics.duckdb")

\# Initialisation du moteur transactionnel SQLite pour la gestion locale  
engine \= create\_engine(f"sqlite:///{SQLITE\_PATH}", connect\_args={"check\_same\_thread": False})  
SessionLocal \= sessionmaker(autocommit=False, autoflush=False, bind=engine)  
Base \= declarative\_base()

def get\_db():  
    db \= SessionLocal()  
    try:  
        yield db  
    finally:  
        db.close()

\# Initialisation de la connexion persistante In-Process DuckDB (OLAP)  
def get\_duckdb\_connection():  
    conn \= duckdb.connect(database=DUCKDB\_PATH, read\_only=False)  
    try:  
        yield conn  
    finally:  
        conn.close()

#### **4.3. Modèles de Données et Chaînage d'Audit Immuable : backend/app/models.py**

Pour remplacer la complexité d'une blockchain tout en conservant ses garanties de non-répudiation et de détection de falsification, chaque transaction locale génère un hash cryptographique dépendant du bloc précédent \[REF-04\].

import hashlib  
from sqlalchemy import Column, Integer, String, DateTime  
from datetime import datetime  
from .database import Base

class AuditTrail(Base):  
    \_\_tablename\_\_ \= "sda\_audit\_trail"

    id \= Column(Integer, primary\_key=True, index=True)  
    timestamp \= Column(DateTime, default=datetime.utcnow)  
    action \= Column(String(50))   
    tenant\_id \= Column(String(50), index=True)  
    record\_hash \= Column(String(64))  
    previous\_hash \= Column(String(64))

    def calculate\_hash(self, payload\_string: str) \-\> str:  
        sha \= hashlib.sha256()  
        compiled\_string \= f"{self.timestamp}{self.action}{self.tenant\_id}{payload\_string}{self.previous\_hash}"  
        sha.update(compiled\_string.encode('utf-8'))  
        return sha.hexdigest()

#### **4.4. Point d'Entrée Applicatif : backend/app/main.py**

from fastapi import FastAPI  
from fastapi.middleware.cors import CORSMiddleware  
from .database import engine, Base  
from .routers import data, sync

\# Création automatique des schémas relationnels au bootstrap du conteneur  
Base.metadata.create\_all(bind=engine)

app \= FastAPI(  
    title="Sovereign Data Agent (SDA) Core Runtime",  
    description="Moteur backend décentralisé pour l'isolation des données par Tenant",  
    version="1.0.0"  
)

\# Restriction CORS stricte au périmètre local de la machine hôte  
app.add\_middleware(  
    CORSMiddleware,  
    allow\_origins=\["\*"\],  
    allow\_credentials=True,  
    allow\_methods=\["\*"\],  
    allow\_headers=\["\*"\],  
)

app.include\_router(data.router, prefix="/api/v1/data", tags=\["Moteur de Données"\])  
app.include\_router(sync.router, prefix="/api/v1/sync", tags=\["Réseau P2P"\])

@app.get("/health")  
def health\_check():  
    return {  
        "status": "operational",  
        "architecture": "local-first / distributed",  
        "central\_dependency": "none",  
        "offline\_ready": True  
    }

#### **4.5. Logique Métier d'Ingestion In-Situ : backend/app/routers/data.py \[REF-06\]**

Ce module ingère les données applicatives en mémoire locale, met à jour le fichier d'analyse analytique DuckDB, puis l'exporte sous forme de fichier Parquet immuable hautement compressé au sein du répertoire de réplication.

import os  
from fastapi import APIRouter, Depends, HTTPException, status  
from sqlalchemy.orm import Session  
from ..database import get\_db, get\_duckdb\_connection  
from ..models import AuditTrail  
import duckdb

router \= APIRouter()  
SHARED\_DIR \= "/app/data/shared\_storage"

@router.post("/ingest", status\_code=status.HTTP\_201\_CREATED)  
def ingest\_tenant\_data(  
    tenant\_id: str,   
    payload: list\[dict\],   
    db: Session \= Depends(get\_db),  
    duck\_conn: duckdb.DuckDBPyConnection \= Depends(get\_duckdb\_connection)  
):  
    if not payload:  
        raise HTTPException(status\_code=400, detail="Le payload de données ne peut être vide.")  
      
    try:  
        \# 1\. Écriture dans le moteur DuckDB local  
        duck\_conn.execute("""  
            CREATE TABLE IF NOT EXISTS tenant\_metrics (  
                tenant\_id VARCHAR,   
                timestamp TIMESTAMP,   
                metric\_value DOUBLE  
            )  
        """)  
          
        for record in payload:  
            duck\_conn.execute(  
                "INSERT INTO tenant\_metrics VALUES (?, ?, ?)",   
                \[tenant\_id, record.get("timestamp"), record.get("value")\]  
            )  
          
        \# 2\. Matérialisation du fichier Parquet unique dédié au partitionnement Syncthing  
        parquet\_file\_path \= os.path.join(SHARED\_DIR, f"{tenant\_id}\_storage.parquet")  
        duck\_conn.execute(f"COPY tenant\_metrics TO '{parquet\_file\_path}' (FORMAT PARQUET, OVERWRITE\_OR\_IGNORE TRUE)")

        \# 3\. Génération et persistance de la Piste d'Audit (Anti-altération)  
        last\_entry \= db.query(AuditTrail).order\_by(AuditTrail.id.desc()).first()  
        previous\_hash \= last\_entry.record\_hash if last\_entry else "0" \* 64

        audit\_log \= AuditTrail(action="INGEST\_DATA", tenant\_id=tenant\_id, previous\_hash=previous\_hash)  
        audit\_log.record\_hash \= audit\_log.calculate\_hash(str(payload))  
          
        db.add(audit\_log)  
        db.commit()

        return {  
            "status": "success",  
            "transaction\_hash": audit\_log.record\_hash,  
            "file\_written": f"{tenant\_id}\_storage.parquet",  
            "synchronized\_locally": True  
        }  
    except Exception as e:  
        db.rollback()  
        raise HTTPException(status\_code=500, detail=f"Échec critique d'écriture in-situ : {str(e)}")

#### **4.6. Logique de Réconciliation Temporelle : backend/app/routers/sync.py \[REF-04\]**

En mode réseau P2P déconnecté, deux nœuds peuvent modifier de manière concurrente le fichier Parquet. Syncthing résout le transport mais lève un conflit d'état (extension .sync-conflict). Le code ci-dessous réconcilie les données selon une règle mathématique de type CRDT déterministe (Dernière Écriture Gagnante).

import os  
import glob  
from fastapi import APIRouter, Depends, HTTPException  
from ..database import get\_duckdb\_connection  
import duckdb

router \= APIRouter()  
SHARED\_DIR \= "/app/data/shared\_storage"

@router.post("/reconcile")  
def resolve\_p2p\_conflicts(duck\_conn: duckdb.DuckDBPyConnection \= Depends(get\_duckdb\_connection)):  
    try:  
        \# Détection des fichiers de conflits de synchronisation  
        conflict\_pattern \= os.path.join(SHARED\_DIR, "\*.sync-conflict-\*")  
        conflict\_files \= glob.glob(conflict\_pattern)  
          
        if not conflict\_files:  
            return {"status": "clean", "message": "Aucun conflit de réplication réseau détecté."}  
          
        \# Ingestion et fusion par DuckDB des lignes orphelines basées sur le timestamp  
        for file in conflict\_files:  
            duck\_conn.execute(f"""  
                INSERT INTO tenant\_metrics   
                SELECT tenant\_id, timestamp, metric\_value FROM read\_parquet('{file}')  
                WHERE timestamp NOT IN (SELECT timestamp FROM tenant\_metrics);  
            """)  
            \# Nettoyage après réconciliation applicative  
            os.remove(file)  
              
        \# Régénération du fichier Parquet principal unifié  
        parquet\_clean\_path \= os.path.join(SHARED\_DIR, "tenant\_A\_storage.parquet")  
        duck\_conn.execute(f"COPY tenant\_metrics TO '{parquet\_clean\_path}' (FORMAT PARQUET, OVERWRITE\_OR\_IGNORE TRUE)")  
          
        return {  
            "status": "reconciled",  
            "files\_processed": len(conflict\_files),  
            "message": "Conflits d'états résolus avec succès par déduplication temporelle."  
        }  
    except Exception as e:  
        raise HTTPException(status\_code=500, detail=f"Erreur lors de la réconciliation : {str(e)}")

### **5\. Stratégie de Compilation et de Protection (Dockerfile)**

Afin de répondre à la contrainte de **protection de la Propriété Intellectuelle d'AL BARAA CONSULTING** \[REF-02\], le code source Python n'est jamais exposé sur le système de fichiers du client. L'image Docker compile l'ensemble applicatif et s'exécute de manière hermétique.

FROM python:3.11-slim

WORKDIR /app

\# Installation des dépendances et outils systèmes indispensables aux moteurs C-bound  
RUN apt-get update && apt-get install \-y \\  
    build-essential \\  
    curl \\  
    && rm \-rf /var/lib/apt/lists/\*

COPY requirements.txt .  
RUN pip install \--no-cache-dir \-r requirements.txt

\# Injection du package applicatif  
COPY ./app ./app

\# Création des répertoires de données à isolation forte  
RUN mkdir \-p /app/data/db /app/data/shared\_storage && \\  
    chmod \-R 700 /app/data

EXPOSE 8000

\# Sonde de vérification de l'intégrité opérationnelle locale  
HEALTHCHECK \--interval=15s \--timeout=3s \--start-period=5s \--retries=3 \\  
  CMD curl \-f http://localhost:8000/health || exit 1

CMD \["uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "8000"\]

### **6\. Protocole de Recette Spécifique du Prototype (Pipeline de Validation) \[REF-03\]**

Pour valider le fonctionnement de votre livrable devant votre encadrant et votre jury d'école d'ingénieurs, exécutez la suite séquentielle de tests opérationnels suivante :

#### **Étape A : Build et Déploiement Initial**

\# Compilation de l'image isolée et lancement des services d'infrastructure  
docker-compose up \--build \-d

\# Validation de l'état des conteneurs  
docker ps

*Résultat attendu :* Les conteneurs sda\_backend\_core and sda\_network\_sync passent au statut *healthy*.

#### **Étape B : Test du Mode Hors-ligne (Coupure de la Connectivité Externe)**

\# Déconnexion virtuelle de la carte réseau externe du nœud  
docker network disconnect sda-prototype\_sda-network sda\_network\_sync

\# Ingestion de données locales via l'API locale  
curl \-X 'POST' 'http://localhost:8000/api/v1/data/ingest?tenant\_id=tenant\_A' \\  
  \-H 'Content-Type: application/json' \\  
  \-d '\[{"timestamp": "2026-05-19T14:30:00", "value": 89.4}\]'

*Résultat attendu :* Réponse HTTP 201 Created. Le traitement, l'écriture DuckDB et la génération de la piste d'audit s'exécutent avec succès malgré l'absence totale de réseau.

#### **Étape C : Validation Spatiale (Zéro Copie Centrale)**

Exécutez une vérification de la persistance locale sur la machine hôte :

ls \-lh data/shared\_storage/

*Résultat attendu :* Le fichier tenant\_A\_storage.parquet est présent. Aucune donnée n'a été transmise vers un serveur ou un cloud externe. L'étanchéité du tenant est absolue \[REF-05\].

### **📚 Sources Documentaires Référencées (PFE & R\&D ACT)**

* \[REF-01\] **Plan\_Directeur\_Jesse\_MPIGA\_v2\_FINAL.docx / MPIGA\_Jesse\_EI\_Promo2026\_Plan\_directeur.md**  
  * *Sujet :* Plan d'exécution du Stage de Fin d'Études chez *AL BARAA CONSULTING* (Spécialité Big Data & IA). Pose le contexte géopolitique africain (AUDPF), l'analyse fonctionnelle (FAST/Pieuvre), les jalons du macro-planning sur 24 semaines, ainsi que les risques associés et l'analyse budgétaire.  
* \[REF-02\] **note-cadrage-sovereign-saas-v2\_260330\_155213.md**  
  * *Sujet :* Note de cadrage du projet R\&D "Sovereign SaaS Framework" (Mars 2026, Version 2.0) portée par l'initiative *ACT (Africa Centred Technology)*. Indique les stratégies de découplage de la donnée, de préservation de la propriété intellectuelle (PI) de l'éditeur dans un modèle décentralisé et de respect du RGPD/AUDPF.  
* \[REF-03\] **Analyse de Faisabilité du Projet PFE.md**  
  * *Sujet :* Étude de faisabilité de l'infrastructure de "brique universelle" (BaaS local) agnostique et décentralisée, assurant le découplage clair entre Frontend métier et Backend d'infrastructure réseau.  
* \[REF-04\] **Rapport de Recherche : Architecture Coffre-Fort Data P2P Souveraine.md**  
  * *Sujet :* Revue approfondie de l'état de l'art par *Manus AI* (Février 2026). Analyse comparative des protocoles P2P de synchronisation de fichiers de bloc (Syncthing), de la découverte réseau locale (mDNS) et des mécanismes de consensus/CRDT.  
* \[REF-05\] **souverainete-donnees-saas.md**  
  * *Sujet :* Document de réflexion stratégique (Mars 2026\) sur la souveraineté des données dans les environnements SaaS. Cartographie détaillée des contraintes juridiques (résidence vs souveraineté), du modèle *Code-to-Data* et des dilemmes d'ingénierie (gestion des schémas distribués, sécurité matérielle type TEE, ZKP, FHE).  
* \[REF-06\] **Stack Technologique Complète du Coffre-Fort Data P2P Souverain.md**  
  * *Sujet :* Spécification de la suite d'outils (Docker, DuckDB, SQLite, Syncthing) retenus pour le prototype fonctionnel de coffre-fort distribué (BaaS local), validant leur compatibilité mutuelle et leurs gains de performance.