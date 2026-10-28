// Page de présentation de l'architecture — remplace templates/dashboard/architecture.html.
// Contenu statique et explicatif uniquement : aucune donnée n'est prétendue provenir de l'API.

import type { ComponentType, ReactNode } from 'react'
import {
  ArrowRightLeft,
  Building2,
  Database,
  GitBranch,
  KeyRound,
  Lock,
  Network,
  QrCode,
  Radio,
  Server,
  ShieldCheck,
  Zap,
} from 'lucide-react'

function ZoneCard({
  icon: Icon,
  iconColor,
  iconBg,
  title,
  detail,
  tagText,
  tagOk,
}: {
  icon: ComponentType<{ size?: number }>
  iconColor: string
  iconBg: string
  title: string
  detail: string
  tagText: string
  tagOk: boolean
}) {
  return (
    <div
      style={{
        background: '#FFFFFF',
        border: '1px solid var(--border)',
        borderRadius: '10px',
        padding: '14px',
        display: 'flex',
        alignItems: 'flex-start',
        gap: '12px',
      }}
    >
      <div
        style={{
          width: '36px',
          height: '36px',
          borderRadius: '9px',
          background: iconBg,
          color: iconColor,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          flexShrink: 0,
        }}
      >
        <Icon size={17} />
      </div>
      <div>
        <div style={{ fontSize: '.85rem', fontWeight: 700, color: 'var(--text-primary)', marginBottom: '3px' }}>{title}</div>
        <div style={{ fontSize: '.75rem', color: 'var(--text-secondary)', lineHeight: 1.5 }}>{detail}</div>
        <span
          style={{
            display: 'inline-block',
            fontSize: '.65rem',
            fontWeight: 700,
            padding: '2px 8px',
            borderRadius: '20px',
            marginTop: '6px',
            background: tagOk ? '#D1FAE5' : '#FEE2E2',
            color: tagOk ? '#065F46' : '#991B1B',
          }}
        >
          {tagText}
        </span>
      </div>
    </div>
  )
}

function Zone({
  label,
  labelColor,
  borderColor,
  background,
  children,
  footer,
  footerLabel,
  footerColor,
}: {
  label: string
  labelColor: string
  borderColor: string
  background: string
  children: ReactNode
  footer: string[]
  footerLabel: string
  footerColor: string
}) {
  return (
    <div
      style={{
        background,
        border: `1.5px solid ${borderColor}`,
        borderRadius: '16px',
        padding: '18px',
        display: 'flex',
        flexDirection: 'column',
        gap: '12px',
      }}
    >
      <div style={{ fontSize: '.68rem', fontWeight: 800, textTransform: 'uppercase', letterSpacing: '.1em', color: labelColor }}>{label}</div>
      {children}
      <div style={{ marginTop: 'auto', paddingTop: '10px', borderTop: `1px solid ${borderColor}`, fontSize: '.72rem', color: 'var(--text-secondary)', lineHeight: 1.8 }}>
        <div style={{ fontWeight: 700, color: footerColor, marginBottom: '4px' }}>{footerLabel}</div>
        {footer.map((line, idx) => (
          <div key={idx}>{line}</div>
        ))}
      </div>
    </div>
  )
}

function PanelTitle({ icon: Icon, color, children }: { icon: ComponentType<{ size?: number }>; color: string; children: ReactNode }) {
  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        fontSize: '.78rem',
        fontWeight: 800,
        textTransform: 'uppercase',
        letterSpacing: '.08em',
        color,
        marginBottom: '14px',
        paddingBottom: '10px',
        borderBottom: '1px solid var(--border)',
      }}
    >
      <Icon size={15} />
      {children}
    </div>
  )
}

export default function ArchitecturePage() {
  return (
    <div>
      <div style={{ textAlign: 'center', marginBottom: '36px' }}>
        <span
          style={{
            display: 'inline-block',
            fontSize: '.68rem',
            fontWeight: 700,
            letterSpacing: '.12em',
            textTransform: 'uppercase',
            padding: '4px 14px',
            borderRadius: '20px',
            marginBottom: '12px',
            background: 'rgba(160,21,32,0.08)',
            border: '1px solid rgba(160,21,32,0.25)',
            color: 'var(--crimson)',
          }}
        >
          Architecture souveraine
        </span>
        <h1 style={{ fontSize: '24px', fontWeight: 800, color: 'var(--text-primary)', marginBottom: '8px' }}>
          Architecture SaaS Souverain
        </h1>
        <p style={{ fontSize: '13px', color: 'var(--text-secondary)' }}>
          Flux zero-knowledge · Données souveraines sur le cluster PME · Réplication PostgreSQL distribuée
        </p>
      </div>

      {/* Les trois acteurs */}
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: '20px', marginBottom: '32px' }}>
        <Zone
          label="① Serveur éditeur SaaS"
          labelColor="var(--crimson)"
          borderColor="rgba(160,21,32,0.3)"
          background="rgba(160,21,32,0.04)"
          footer={['UUID nœuds & rôle cluster', 'Dernier heartbeat reçu', 'Jamais les données métier', 'Jamais la clé DEK']}
          footerLabel="Ce que voit l'éditeur"
          footerColor="var(--crimson)"
        >
          <ZoneCard
            icon={Server}
            iconColor="#6EE7B7"
            iconBg="#092E20"
            title="Django + React / TypeScript"
            detail="Portail web SaaS · gestion des tenants, des licences et suivi du parc installé."
            tagText="Voit : comptes & licences"
            tagOk
          />
          <ZoneCard
            icon={Database}
            iconColor="#93C5FD"
            iconBg="#17375F"
            title="PostgreSQL éditeur"
            detail="Tenants · appareils · sessions. Identifiant d'installation (UUID authentifié), jamais l'adresse MAC."
            tagText="Jamais les données métier PME"
            tagOk={false}
          />
        </Zone>

        <Zone
          label="② Relais zero-knowledge"
          labelColor="#4F46E5"
          borderColor="rgba(99,102,241,0.3)"
          background="rgba(99,102,241,0.04)"
          footer={['UUID nœuds (metadata)', 'Blobs chiffrés opaques', 'Jamais la DEK en clair', 'Jamais les clés privées X25519']}
          footerLabel="Garantie zero-knowledge"
          footerColor="#4F46E5"
        >
          <ZoneCard
            icon={Radio}
            iconColor="#FBBF24"
            iconBg="#4F1C14"
            title="Service stateless (Rust/Go)"
            detail="Point de rendez-vous multi-sites · enregistrement des nœuds PME à chaque démarrage."
            tagText="Ne lit jamais le contenu"
            tagOk={false}
          />
          <ZoneCard
            icon={Lock}
            iconColor="#FCA5A5"
            iconBg="#4A0F1C"
            title="Stockage objet S3-compatible"
            detail="Blobs DEK chiffrés (XChaCha20-Poly1305) · blob de récupération protégé par Argon2id."
            tagText="Blobs opaques — jamais déchiffrés"
            tagOk
          />
        </Zone>

        <Zone
          label="③ Cluster PME — périmètre souverain"
          labelColor="#059669"
          borderColor="rgba(16,185,129,0.3)"
          background="rgba(16,185,129,0.05)"
          footer={['Données métier (stock, paie…)', 'Journal CBOR chiffré (DEK)', 'DEK déchiffrée en RAM uniquement', 'Failover auto ≥ 3 nœuds · fencing par époque']}
          footerLabel="Périmètre souverain"
          footerColor="#059669"
        >
          <ZoneCard
            icon={ShieldCheck}
            iconColor="#34D399"
            iconBg="#064E3B"
            title="Nœud primaire"
            detail="Exécute le logiciel métier · PostgreSQL maître · DEK déchiffrée en RAM uniquement."
            tagText="Données en clair — périmètre souverain uniquement"
            tagOk
          />
          <ZoneCard
            icon={GitBranch}
            iconColor="#FCD34D"
            iconBg="#451A03"
            title="Nœud(s) standby"
            detail="PostgreSQL standby · réplication streaming synchrone · prêt au failover."
            tagText="Réplique synchrone"
            tagOk
          />
        </Zone>
      </div>

      {/* Panneaux inférieurs */}
      <div style={{ display: 'grid', gridTemplateColumns: '1.1fr 1fr 1fr', gap: '18px', marginBottom: '28px' }}>
        {/* Hiérarchie des clés */}
        <div style={{ background: '#FFFFFF', border: '1px solid var(--border)', borderRadius: '14px', padding: '18px 20px', boxShadow: 'var(--shadow)' }}>
          <PanelTitle icon={KeyRound} color="#4F46E5">Hiérarchie des clés</PanelTitle>
          <div style={{ fontFamily: 'monospace', fontSize: '.76rem', lineHeight: 2, color: 'var(--text-secondary)' }}>
            <div><strong style={{ color: '#4F46E5' }}>DEK</strong> — symétrique, unique par entreprise</div>
            <div style={{ paddingLeft: '14px' }}>├▶ chiffre données métier + journal CBOR</div>
            <div style={{ paddingLeft: '14px' }}>├▶ <strong style={{ color: '#059669' }}>sealed box X25519</strong></div>
            <div style={{ paddingLeft: '28px' }}>└▶ un blob par appareil autorisé</div>
            <div style={{ paddingLeft: '14px' }}>└▶ <strong style={{ color: '#059669' }}>Argon2id</strong></div>
            <div style={{ paddingLeft: '28px' }}>└▶ blob de récupération, stocké chiffré sur le relais</div>
          </div>
          <div style={{ marginTop: '14px', fontFamily: 'monospace', fontSize: '.76rem', lineHeight: 2, color: 'var(--text-secondary)' }}>
            <div><strong style={{ color: '#4F46E5' }}>Enrôlement d'un nouveau nœud</strong></div>
            <div style={{ paddingLeft: '14px' }}>1. le nœud génère sa paire X25519, affiche sa clé publique en QR</div>
            <div style={{ paddingLeft: '14px' }}>2. un appareil autorisé scanne le QR</div>
            <div style={{ paddingLeft: '14px' }}>3. il emballe la DEK (sealed box) pour le nouveau nœud</div>
            <div style={{ paddingLeft: '14px' }}>4. le nouveau nœud ouvre le blob avec sa clé privée</div>
            <div style={{ paddingLeft: '14px' }}>5. le jeton d'invitation est consommé (usage unique)</div>
          </div>
        </div>

        {/* Règles de bascule */}
        <div style={{ background: '#FFFFFF', border: '1px solid var(--border)', borderRadius: '14px', padding: '18px 20px', boxShadow: 'var(--shadow)' }}>
          <PanelTitle icon={Zap} color="#059669">Règles de bascule</PanelTitle>
          {[
            { color: '#10B981', title: '≥ 3 nœuds — bascule automatique par quorum', sub: 'Supervision type Patroni · promotion de standby' },
            { color: '#EF4444', title: '2 nœuds — bascule manuelle uniquement', sub: 'Intervention de la PME requise' },
            { color: '#F59E0B', title: 'Fencing — jeton d\'époque monotone', sub: 'Ancien primaire isolé · split-brain impossible' },
            { color: '#F59E0B', title: 'Alerte 3 → 2 nœuds — signalée au SaaS', sub: 'La PME est informée que seule la bascule manuelle reste possible' },
            { color: '#EF4444', title: 'Sinistre total — récupération assistée', sub: 'Blob relais + code de récupération Argon2id → DEK retrouvée' },
          ].map((item, idx, arr) => (
            <div key={idx} style={{ display: 'flex', alignItems: 'flex-start', gap: '10px', padding: '8px 0', borderBottom: idx < arr.length - 1 ? '1px solid var(--border)' : 'none' }}>
              <span style={{ width: '9px', height: '9px', borderRadius: '50%', background: item.color, flexShrink: 0, marginTop: '5px' }} />
              <div>
                <div style={{ fontSize: '.79rem', color: 'var(--text-primary)', lineHeight: 1.4 }}>{item.title}</div>
                <div style={{ fontSize: '.68rem', color: 'var(--text-muted)', marginTop: '2px' }}>{item.sub}</div>
              </div>
            </div>
          ))}
        </div>

        {/* Flux de données */}
        <div style={{ background: '#FFFFFF', border: '1px solid var(--border)', borderRadius: '14px', padding: '18px 20px', boxShadow: 'var(--shadow)' }}>
          <PanelTitle icon={ArrowRightLeft} color="var(--gold)">Flux entre acteurs</PanelTitle>
          {[
            { tag: 'TLS', tagColor: '#4F46E5', label: 'Nœud PME → Éditeur SaaS', desc: 'UUID · rôle · heartbeat de topologie' },
            { tag: 'ZK', tagColor: '#059669', label: 'Nœud PME → Relais', desc: 'DEK chiffrée X25519 · blob de récupération Argon2id' },
            { tag: 'WAL', tagColor: 'var(--gold)', label: 'Primaire PG → Standby PG', desc: 'Réplication streaming synchrone' },
            { tag: 'RAM', tagColor: 'var(--gold)', label: 'Application → PostgreSQL local', desc: 'Données en clair · exclusivement LAN PME' },
            { tag: 'TLS', tagColor: '#4F46E5', label: 'Éditeur → Nœud PME', desc: 'Instructions d\'installation · docker-compose.yml' },
          ].map((item, idx, arr) => (
            <div key={idx} style={{ display: 'flex', alignItems: 'flex-start', gap: '9px', padding: '8px 0', borderBottom: idx < arr.length - 1 ? '1px solid var(--border)' : 'none' }}>
              <span style={{ fontSize: '.6rem', fontWeight: 700, padding: '2px 8px', borderRadius: '20px', whiteSpace: 'nowrap', flexShrink: 0, marginTop: '2px', background: `${item.tagColor}22`, color: item.tagColor }}>
                {item.tag}
              </span>
              <div>
                <div style={{ fontSize: '.73rem', fontWeight: 600, color: 'var(--text-primary)' }}>{item.label}</div>
                <div style={{ fontSize: '.69rem', color: 'var(--text-muted)', marginTop: '2px', lineHeight: 1.5 }}>{item.desc}</div>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Frontière de confiance */}
      <div style={{ background: '#FFFFFF', border: '1px solid var(--border)', borderRadius: '14px', padding: '20px 24px', boxShadow: 'var(--shadow)', marginBottom: '28px' }}>
        <PanelTitle icon={Network} color="var(--crimson)">Frontière de confiance</PanelTitle>
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: '16px' }}>
          <div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '6px', fontWeight: 700, fontSize: '.82rem', color: 'var(--text-primary)', marginBottom: '6px' }}>
              <Building2 size={14} color="var(--crimson)" /> Éditeur
            </div>
            <p style={{ fontSize: '.76rem', color: 'var(--text-secondary)', lineHeight: 1.6 }}>
              Gère comptes et licences. Voit uniquement les données de compte et de licence — jamais les données métier.
            </p>
          </div>
          <div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '6px', fontWeight: 700, fontSize: '.82rem', color: 'var(--text-primary)', marginBottom: '6px' }}>
              <QrCode size={14} color="#4F46E5" /> Relais
            </div>
            <p style={{ fontSize: '.76rem', color: 'var(--text-secondary)', lineHeight: 1.6 }}>
              Stockage chiffré des données métier et point de rendez-vous multi-sites. Ne voit jamais le clair, même compromis ou saisi.
            </p>
          </div>
          <div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '6px', fontWeight: 700, fontSize: '.82rem', color: 'var(--text-primary)', marginBottom: '6px' }}>
              <ShieldCheck size={14} color="#059669" /> Cluster PME
            </div>
            <p style={{ fontSize: '.76rem', color: 'var(--text-secondary)', lineHeight: 1.6 }}>
              Exécute le logiciel métier, détient et sérialise les données en clair — exclusivement dans le périmètre souverain de la PME.
            </p>
          </div>
        </div>
      </div>

      {/* Bandeau final */}
      <div
        style={{
          background: 'linear-gradient(135deg, rgba(99,102,241,0.08), rgba(16,185,129,0.06))',
          border: '1px solid rgba(99,102,241,0.2)',
          borderRadius: '14px',
          padding: '18px 24px',
          display: 'flex',
          alignItems: 'flex-start',
          gap: '14px',
        }}
      >
        <Lock size={22} color="#4F46E5" style={{ flexShrink: 0, marginTop: '2px' }} />
        <p style={{ fontSize: '.82rem', color: 'var(--text-secondary)', lineHeight: 1.7, margin: 0 }}>
          <strong style={{ color: 'var(--text-primary)' }}>Garantie zero-knowledge</strong> — L'éditeur gère les comptes et
          licences sans jamais voir les données métier. Le relais stocke des blobs chiffrés opaques qu'il ne peut pas
          déchiffrer. Les données métier ne sortent jamais en clair du périmètre PME, même en cas de saisie du relais ou de
          compromission du serveur éditeur.
        </p>
      </div>
    </div>
  )
}
