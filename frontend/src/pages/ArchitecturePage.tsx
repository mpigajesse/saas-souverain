// Page de présentation de l'architecture V3 : AMANE, enveloppe d'exécution
// souveraine pour SaaS externes (docs/refonte-v3/AMANE-Refonte-V3.md).
// Contenu statique et explicatif uniquement : aucune donnée n'est prétendue provenir de l'API.

import type { ComponentType, ReactNode } from 'react'
import {
  AppWindow,
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
          AMANE — enveloppe d'exécution souveraine
        </h1>
        <p style={{ fontSize: '13px', color: 'var(--text-secondary)' }}>
          Un SaaS externe s'exécute dans le cluster de la PME, sans changer son code · zero-knowledge · haute disponibilité etcd + Patroni
        </p>
      </div>

      {/* Les trois acteurs */}
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: '20px', marginBottom: '32px' }}>
        <Zone
          label="① Serveur éditeur SaaS"
          labelColor="var(--crimson)"
          borderColor="rgba(160,21,32,0.3)"
          background="rgba(160,21,32,0.04)"
          footer={['UUID d\'installation, rôle, état du quorum', 'Santé de l\'application, chiffrement du disque', 'Jamais les données métier', 'Jamais la DEK ni les secrets de base']}
          footerLabel="Ce que voit l'éditeur"
          footerColor="var(--crimson)"
        >
          <ZoneCard
            icon={Server}
            iconColor="#6EE7B7"
            iconBg="#092E20"
            title="Django + React / TypeScript"
            detail="Tenants, licences, parc installé et catalogue des applications hébergées (manifestes, images par digest). Génère la pile et les installeurs de chaque PME."
            tagText="Voit : comptes, licences, manifestes"
            tagOk
          />
          <ZoneCard
            icon={Database}
            iconColor="#93C5FD"
            iconBg="#17375F"
            title="Registre d'images"
            detail="Images de la pile PME et des applications, épinglées par digest : la PME exécute exactement ce qui a été validé."
            tagText="Aucune donnée de la PME"
            tagOk={false}
          />
        </Zone>

        <Zone
          label="② Relais zero-knowledge"
          labelColor="#4F46E5"
          borderColor="rgba(99,102,241,0.3)"
          background="rgba(99,102,241,0.04)"
          footer={['Topologie des nœuds (metadata)', 'Sauvegardes chiffrées opaques', 'Jamais la DEK', 'Jamais les clés privées X25519']}
          footerLabel="Garantie zero-knowledge"
          footerColor="#4F46E5"
        >
          <ZoneCard
            icon={Radio}
            iconColor="#FBBF24"
            iconBg="#4F1C14"
            title="Relais (Rust)"
            detail="Point de rendez-vous : chaque nœud s'y annonce au démarrage et y découvre ses pairs."
            tagText="Ne lit jamais le contenu"
            tagOk={false}
          />
          <ZoneCard
            icon={Lock}
            iconColor="#FCA5A5"
            iconBg="#4A0F1C"
            title="Stockage de blobs"
            detail="Sauvegardes quotidiennes des bases, chiffrées par la DEK avant de quitter la PME · rotation sur 7 jours."
            tagText="Blobs opaques — jamais déchiffrés"
            tagOk
          />
        </Zone>

        <Zone
          label="③ Cluster PME — périmètre souverain"
          labelColor="#059669"
          borderColor="rgba(16,185,129,0.3)"
          background="rgba(16,185,129,0.05)"
          footer={['Données de l\'application en clair', 'DEK sur les seuls serveurs enrôlés', 'Disque chiffré vérifié (BitLocker, LUKS)', 'Bascule par etcd + Patroni · fencing par timeline']}
          footerLabel="Périmètre souverain"
          footerColor="#059669"
        >
          <ZoneCard
            icon={AppWindow}
            iconColor="#34D399"
            iconBg="#064E3B"
            title="Application externe hébergée"
            detail="Image de l'éditeur tiers, inchangée · réseau interne sans sortie vers Internet · interface relayée par le nœud · base dédiée jointe par HAProxy."
            tagText="Ne peut rien envoyer à son éditeur"
            tagOk
          />
          <ZoneCard
            icon={ShieldCheck}
            iconColor="#93C5FD"
            iconBg="#17375F"
            title="Nœud AMANE (Rust)"
            detail="Console d'administration, enrôlement des serveurs, sauvegardes chiffrées, garde de quorum en lecture."
            tagText="Seul à détenir la DEK"
            tagOk
          />
          <ZoneCard
            icon={GitBranch}
            iconColor="#FCD34D"
            iconBg="#451A03"
            title="Pile haute disponibilité"
            detail="etcd (Raft) + Patroni / PostgreSQL 17 + HAProxy, pilotés par l'orchestrateur Go · témoin etcd sur une troisième machine."
            tagText="Réplication synchrone"
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
            <div><strong style={{ color: '#4F46E5' }}>DEK</strong> — symétrique, unique par cluster</div>
            <div style={{ paddingLeft: '14px' }}>├▶ chiffre le journal du nœud et les sauvegardes</div>
            <div style={{ paddingLeft: '14px' }}>├▶ <strong style={{ color: '#059669' }}>sealed box X25519</strong></div>
            <div style={{ paddingLeft: '28px' }}>└▶ une par serveur enrôlé</div>
            <div style={{ paddingLeft: '14px' }}>└▶ <strong style={{ color: '#059669' }}>KEK — Shamir K=2/N=3</strong></div>
            <div style={{ paddingLeft: '28px' }}>└▶ récupération d'urgence, sans tiers de confiance</div>
          </div>
          <div style={{ marginTop: '14px', fontFamily: 'monospace', fontSize: '.76rem', lineHeight: 2, color: 'var(--text-secondary)' }}>
            <div><strong style={{ color: '#4F46E5' }}>Enrôlement d'un nouveau serveur</strong> (opération humaine)</div>
            <div style={{ paddingLeft: '14px' }}>1. un administrateur génère un code dans la console d'un serveur enrôlé</div>
            <div style={{ paddingLeft: '14px' }}>2. il le saisit sur le nouveau serveur (ss-node enroler)</div>
            <div style={{ paddingLeft: '14px' }}>3. la DEK est scellée pour la clé publique du nouveau serveur</div>
            <div style={{ paddingLeft: '14px' }}>4. le nouveau serveur l'ouvre avec sa clé privée</div>
            <div style={{ paddingLeft: '14px' }}>5. le code est consommé (usage unique, 15 minutes)</div>
          </div>
        </div>

        {/* Règles de bascule */}
        <div style={{ background: '#FFFFFF', border: '1px solid var(--border)', borderRadius: '14px', padding: '18px 20px', boxShadow: 'var(--shadow)' }}>
          <PanelTitle icon={Zap} color="#059669">Règles de bascule</PanelTitle>
          {[
            { color: '#10B981', title: 'Bascule automatique par etcd + Patroni', sub: 'Quorum etcd à 3 membres : 2 serveurs + témoin, ou 3 serveurs · panne détectée en moins de 5 s' },
            { color: '#10B981', title: '≥ 3 serveurs — réplication synchrone stricte', sub: 'Aucune écriture validée sans copie sur un second serveur' },
            { color: '#F59E0B', title: '2 serveurs — synchrone non strict, alerte', sub: 'Un serveur seul continue d\'écrire ; la PME est prévenue que ses données ne sont plus protégées' },
            { color: '#EF4444', title: '2 serveurs sans témoin — pas de quorum', sub: 'La panne de l\'un bloque les écritures : le portail demande d\'installer un témoin' },
            { color: '#F59E0B', title: 'Fencing — timeline PostgreSQL', sub: 'Un ancien primaire revenu ne peut plus écrire · split-brain impossible' },
            { color: '#EF4444', title: 'Sinistre — restauration depuis le relais', sub: 'Sauvegarde chiffrée récupérée et déchiffrée par un serveur enrôlé' },
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
            { tag: 'META', tagColor: '#4F46E5', label: 'Nœud PME → Éditeur', desc: 'UUID · rôle · état du quorum · santé de l\'application · chiffrement du disque' },
            { tag: 'ZK', tagColor: '#059669', label: 'Nœud PME → Relais', desc: 'Annonce de topologie · sauvegardes chiffrées par la DEK' },
            { tag: 'WAL', tagColor: 'var(--gold)', label: 'Primaire PG → Standby PG', desc: 'Réplication streaming, synchrone selon la taille du cluster' },
            { tag: 'LAN', tagColor: 'var(--gold)', label: 'Application → sa base', desc: 'Réseau interne sans sortie · HAProxy mène toujours au primaire' },
            { tag: 'PILE', tagColor: '#4F46E5', label: 'Éditeur → PME', desc: 'docker-compose et installeurs · images par digest · manifeste de l\'application' },
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
              Gère comptes, licences et le catalogue des applications. Voit des métadonnées d'état — jamais les données
              de la PME, ni la DEK, ni les mots de passe de ses bases.
            </p>
          </div>
          <div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '6px', fontWeight: 700, fontSize: '.82rem', color: 'var(--text-primary)', marginBottom: '6px' }}>
              <QrCode size={14} color="#4F46E5" /> Relais
            </div>
            <p style={{ fontSize: '.76rem', color: 'var(--text-secondary)', lineHeight: 1.6 }}>
              Point de rendez-vous et stockage des sauvegardes, chiffrées avant de quitter la PME. Ne voit jamais le
              clair, même compromis ou saisi.
            </p>
          </div>
          <div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '6px', fontWeight: 700, fontSize: '.82rem', color: 'var(--text-primary)', marginBottom: '6px' }}>
              <ShieldCheck size={14} color="#059669" /> Cluster PME
            </div>
            <p style={{ fontSize: '.76rem', color: 'var(--text-secondary)', lineHeight: 1.6 }}>
              Exécute le SaaS externe et détient les données en clair, exclusivement dans son périmètre. L'application
              hébergée tourne sans accès à Internet : même son éditeur ne peut rien en extraire.
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
          <strong style={{ color: 'var(--text-primary)' }}>Garantie zero-knowledge</strong> — Personne d'autre que la PME ne
          voit ses données en clair : ni l'éditeur AMANE, qui gère comptes, licences et applications ; ni le relais, qui ne
          stocke que des sauvegardes chiffrées ; ni l'éditeur du SaaS hébergé, dont l'application n'a aucune sortie réseau.
          La DEK et toutes les clés restent dans le cluster de la PME, et leur transmission reste une opération humaine.
        </p>
      </div>
    </div>
  )
}
