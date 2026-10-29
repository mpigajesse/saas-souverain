// Espace client PME — remplace templates/tenants/mon-espace.html.
// Données réelles uniquement : GET /api/tenants/me/ (voir CONTRAT-API.md §3).
// Aucun champ non servi par l'API n'est affiché. Le mot de passe pgAdmin n'est
// PAS connu du portail : il est généré sur le réseau de la PME et affiché en
// fin d'installation (refonte V3, ADR-005).

import { useCallback, useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import {
  AlertCircle,
  Building2,
  Calendar,
  ExternalLink,
  Inbox,
  Laptop,
  Loader2,
  Mail,
  MapPin,
  Monitor,
  Phone,
  Radio,
  RotateCw,
  Server,
  ShieldCheck,
  Terminal,
  Users,
  Wifi,
  WifiOff,
} from 'lucide-react'
import { downloadInstallerUrl, getEspacePme, normalizeApiError } from '../api/client'
import { NODE_ROLE_LABELS, PLAN_LABELS, isDeviceOnline } from '../types/index'
import type { Device, EspacePme } from '../types/index'
import StatusBadge from '../components/StatusBadge'

// ---------------------------------------------------------------------------
// États partagés (chargement / erreur)
// ---------------------------------------------------------------------------

function LoadingState() {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: '10px', color: 'var(--text-secondary)', padding: '64px 0', justifyContent: 'center' }}>
      <Loader2 size={20} className="mep-spin" />
      <span>Chargement de votre espace…</span>
      <style>{`.mep-spin { animation: mep-spin 0.8s linear infinite; } @keyframes mep-spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }`}</style>
    </div>
  )
}

function ErrorState({ message, onRetry }: { message: string; onRetry: () => void }) {
  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'flex-start',
        gap: '12px',
        background: '#FDECEC',
        border: '1px solid #F3B7B7',
        color: 'var(--danger)',
        borderRadius: 'var(--radius)',
        padding: '18px 20px',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
        <AlertCircle size={18} />
        <span>{message}</span>
      </div>
      <button
        onClick={onRetry}
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '6px',
          background: '#FFFFFF',
          border: '1px solid var(--danger)',
          color: 'var(--danger)',
          borderRadius: '6px',
          padding: '6px 12px',
          fontSize: '12px',
          fontWeight: 600,
          cursor: 'pointer',
        }}
      >
        <RotateCw size={14} /> Réessayer
      </button>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Carte machine
// ---------------------------------------------------------------------------

/** Pictogramme de la machine selon son OS.
 *  Composant a part entiere plutot qu'une fonction renvoyant un composant :
 *  un composant choisi pendant le rendu est remonte a chaque passe. */
function MachineIcon({ os, size, color }: { os: string; size: number; color: string }) {
  const lower = os.toLowerCase()
  if (lower.includes('win')) return <Monitor size={size} color={color} />
  if (lower.includes('linux') || lower.includes('ubuntu') || lower.includes('debian') || lower.includes('kali')) {
    return <Terminal size={size} color={color} />
  }
  if (lower.includes('server')) return <Server size={size} color={color} />
  return <Laptop size={size} color={color} />
}

function DeviceCard({ device }: { device: Device }) {
  const online = isDeviceOnline(device)
  const roleLabel = NODE_ROLE_LABELS[device.node_role] ?? device.node_role

  return (
    <div
      style={{
        background: '#FFFFFF',
        border: '1px solid var(--border)',
        borderRadius: '12px',
        padding: '20px',
        display: 'flex',
        flexDirection: 'column',
        gap: '12px',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
        <div
          style={{
            width: '40px',
            height: '40px',
            background: '#FFF5F5',
            borderRadius: '10px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            flexShrink: 0,
          }}
        >
          <MachineIcon os={device.os} size={20} color="var(--crimson)" />
        </div>
        <div style={{ minWidth: 0 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '6px', flexWrap: 'wrap' }}>
            <span style={{ fontWeight: 700, fontSize: '0.95rem', color: 'var(--text-primary)' }}>
              {device.hostname || 'Machine sans nom'}
            </span>
            {device.node_role && (
              <span
                style={{
                  background: device.node_role === 'primary' ? '#D1FAE5' : '#FEF3C7',
                  color: device.node_role === 'primary' ? '#065F46' : '#92400E',
                  borderRadius: '6px',
                  padding: '1px 7px',
                  fontSize: '0.7rem',
                  fontWeight: 700,
                }}
              >
                {roleLabel}
              </span>
            )}
          </div>
          <div style={{ fontSize: '0.78rem', color: 'var(--text-muted)' }}>{device.os || 'OS inconnu'}</div>
        </div>
      </div>

      <div style={{ display: 'flex', alignItems: 'center', gap: '6px', fontSize: '0.78rem', color: online ? '#2A7A3B' : 'var(--text-muted)' }}>
        {online ? <Wifi size={13} /> : <WifiOff size={13} />}
        {online ? 'En ligne' : 'Hors ligne'}
        {!device.is_within_license && (
          <span style={{ marginLeft: '6px', color: 'var(--crimson)', fontWeight: 600 }}>· hors quota de licence</span>
        )}
      </div>

      {device.node_addr && (
        <div
          style={{
            fontSize: '0.78rem',
            color: 'var(--text-secondary)',
            fontFamily: 'monospace',
            background: '#F8F8F8',
            border: '1px solid var(--border)',
            borderRadius: '6px',
            padding: '4px 8px',
          }}
        >
          {device.node_addr}
        </div>
      )}

      <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>
        Dernière connexion : {device.last_seen ? new Date(device.last_seen).toLocaleString('fr-FR') : 'jamais'}
      </div>

      <div style={{ marginTop: '4px' }}>
        {device.web_addr ? (
          <a
            href={`http://${device.web_addr}/`}
            target="_blank"
            rel="noopener noreferrer"
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '8px',
              justifyContent: 'center',
              background: 'var(--crimson)',
              color: '#fff',
              border: 'none',
              borderRadius: '8px',
              padding: '10px 18px',
              fontSize: '0.875rem',
              fontWeight: 600,
              textDecoration: 'none',
            }}
          >
            <ExternalLink size={14} /> Lancer le logiciel
          </a>
        ) : (
          <span
            title={device.node_addr ? 'Interface web pas encore annoncée par ce nœud' : 'Machine non encore enregistrée'}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '8px',
              justifyContent: 'center',
              background: 'var(--border)',
              color: 'var(--text-muted)',
              borderRadius: '8px',
              padding: '10px 18px',
              fontSize: '0.875rem',
              fontWeight: 600,
            }}
          >
            {device.node_addr ? 'Démarrage en cours…' : 'En attente d\'enregistrement'}
          </span>
        )}
      </div>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Page
// ---------------------------------------------------------------------------

export default function MonEspacePage() {
  const [data, setData] = useState<EspacePme | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const espace = await getEspacePme()
      setData(espace)
    } catch (err) {
      setError(normalizeApiError(err).message)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    // Chargement initial differe d'une micro-tache : appeler load() dans la
    // phase synchrone de l'effet declenche des rendus en cascade (React 19,
    // regle react-hooks/set-state-in-effect).
    queueMicrotask(() => {
      void load()
    })
  }, [load])

  if (loading) return <LoadingState />
  if (error) return <ErrorState message={error} onRetry={() => void load()} />
  if (!data) return null

  const { tenant, active_license, devices, pgadmin, besoin_temoin, etat_cluster, relay_url } = data
  const seatsUsedPercent =
    active_license && active_license.seats > 0
      ? Math.min(100, Math.round((devices.length / active_license.seats) * 100))
      : 0

  return (
    <div>
      <div style={{ marginBottom: '28px' }}>
        <h1 style={{ fontSize: '22px', fontWeight: 700, color: 'var(--text-primary)' }}>
          Mon espace <span style={{ color: 'var(--crimson)' }}>{tenant.name}</span>
        </h1>
        <p style={{ color: 'var(--text-secondary)', marginTop: '4px' }}>
          Accédez au logiciel de gestion sur chacune de vos machines.
        </p>
      </div>

      {/* Identité du tenant */}
      <div
        style={{
          background: '#FFFFFF',
          border: '1px solid var(--border)',
          borderRadius: '12px',
          padding: '20px 24px',
          marginBottom: '20px',
          display: 'flex',
          flexWrap: 'wrap',
          gap: '20px',
          alignItems: 'center',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '0.85rem', color: 'var(--text-secondary)' }}>
          <Building2 size={14} /> {tenant.name}
        </div>
        {tenant.email && (
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '0.85rem', color: 'var(--text-secondary)' }}>
            <Mail size={14} /> {tenant.email}
          </div>
        )}
        {tenant.phone && (
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '0.85rem', color: 'var(--text-secondary)' }}>
            <Phone size={14} /> {tenant.phone}
          </div>
        )}
        {tenant.address && (
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '0.85rem', color: 'var(--text-secondary)' }}>
            <MapPin size={14} /> {tenant.address}
          </div>
        )}
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '0.85rem', color: 'var(--text-secondary)' }}>
          <Users size={14} /> {tenant.employee_count} employé{tenant.employee_count > 1 ? 's' : ''}
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '0.85rem', color: 'var(--text-secondary)' }}>
          <Calendar size={14} /> client depuis le {new Date(tenant.created_at).toLocaleDateString('fr-FR')}
        </div>
        <div style={{ marginLeft: 'auto' }}>
          <StatusBadge active={tenant.is_active} />
        </div>
      </div>

      {/* Licence */}
      {active_license ? (
        <div
          style={{
            background: '#FFFFFF',
            border: '1px solid var(--border)',
            borderRadius: '12px',
            padding: '20px 24px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            marginBottom: '32px',
            flexWrap: 'wrap',
            gap: '12px',
          }}
        >
          <div>
            <div style={{ fontSize: '0.78rem', fontWeight: 600, textTransform: 'uppercase', letterSpacing: '.06em', color: 'var(--text-muted)' }}>
              Licence
            </div>
            <div style={{ fontSize: '1rem', fontWeight: 700, color: 'var(--text-primary)' }}>
              {PLAN_LABELS[active_license.plan] ?? active_license.plan}
            </div>
          </div>
          <div>
            <div style={{ fontSize: '0.78rem', fontWeight: 600, textTransform: 'uppercase', letterSpacing: '.06em', color: 'var(--text-muted)' }}>
              Échéance
            </div>
            <div style={{ fontSize: '1rem', fontWeight: 700, color: 'var(--text-primary)' }}>
              {active_license.expires_at ? new Date(active_license.expires_at).toLocaleDateString('fr-FR') : 'sans échéance'}
            </div>
          </div>
          <div>
            <div style={{ fontSize: '0.78rem', fontWeight: 600, textTransform: 'uppercase', letterSpacing: '.06em', color: 'var(--text-muted)' }}>
              Postes
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <span style={{ fontWeight: 700 }}>{devices.length} / {active_license.seats}</span>
              <div style={{ width: '120px', height: '8px', background: 'var(--border)', borderRadius: '4px', overflow: 'hidden' }}>
                <div style={{ width: `${seatsUsedPercent}%`, height: '100%', background: 'var(--crimson)', borderRadius: '4px' }} />
              </div>
            </div>
          </div>
        </div>
      ) : (
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '10px',
            background: '#FEE8E8',
            border: '1px solid var(--crimson)',
            color: 'var(--crimson)',
            borderRadius: 'var(--radius)',
            padding: '14px 18px',
            marginBottom: '32px',
            fontWeight: 600,
            fontSize: '0.9rem',
          }}
        >
          <AlertCircle size={16} /> Aucune licence active pour le moment.
        </div>
      )}

      {/* Identifiants pgAdmin */}
      <div
        style={{
          background: '#F0FDF4',
          border: '1px solid #86EFAC',
          borderRadius: '10px',
          padding: '16px 20px',
          marginBottom: '24px',
          fontSize: '0.875rem',
        }}
      >
        <h3 style={{ fontSize: '0.875rem', fontWeight: 700, color: '#166534', margin: '0 0 12px 0', display: 'flex', alignItems: 'center', gap: '8px' }}>
          <ShieldCheck size={16} /> Accès à votre base de données (pgAdmin)
        </h3>
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <tbody>
            <tr>
              <td style={{ padding: '4px 0', color: '#166534', fontWeight: 600, width: '130px', verticalAlign: 'top' }}>URL pgAdmin</td>
              <td style={{ padding: '4px 0' }}>
                <span
                  style={{
                    fontFamily: 'monospace',
                    background: '#DCFCE7',
                    border: '1px solid #86EFAC',
                    borderRadius: '4px',
                    padding: '2px 8px',
                    letterSpacing: '0.03em',
                    userSelect: 'all',
                  }}
                >
                  {pgadmin.url || 'Non disponible pour le moment'}
                </span>
              </td>
            </tr>
            <tr>
              <td style={{ padding: '4px 0', color: '#166534', fontWeight: 600, verticalAlign: 'top' }}>Identifiant</td>
              <td style={{ padding: '4px 0' }}>
                <span
                  style={{
                    fontFamily: 'monospace',
                    background: '#DCFCE7',
                    border: '1px solid #86EFAC',
                    borderRadius: '4px',
                    padding: '2px 8px',
                    letterSpacing: '0.03em',
                    userSelect: 'all',
                  }}
                >
                  {pgadmin.email}
                </span>
              </td>
            </tr>
            <tr>
              <td style={{ padding: '4px 0', color: '#166534', fontWeight: 600, verticalAlign: 'top' }}>Mot de passe</td>
              <td style={{ padding: '4px 0', color: '#166534' }}>
                Affiché en fin d'installation sur votre premier serveur. Il est généré
                chez vous et n'est jamais transmis au portail.
              </td>
            </tr>
          </tbody>
        </table>
        <p style={{ marginTop: '10px', fontSize: '0.78rem', color: '#166534', opacity: 0.8 }}>
          Conservez ces identifiants en lieu sûr. Cliquez sur une valeur pour la sélectionner.
        </p>
      </div>

      {/* État du cluster remonté par les nœuds (étape 2d) */}
      {(etat_cluster === 'sous_le_seuil' || etat_cluster === 'perdu') && (
        <div
          style={{
            background: '#FEE8E8',
            border: '1px solid var(--crimson)',
            color: 'var(--crimson)',
            borderRadius: '10px',
            padding: '14px 18px',
            marginBottom: '24px',
            fontSize: '0.875rem',
          }}
        >
          <strong style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '6px' }}>
            <AlertCircle size={16} />
            {etat_cluster === 'perdu' ? 'Votre cluster ne peut plus écrire' : 'Vos données ne sont plus protégées'}
          </strong>
          {etat_cluster === 'perdu'
            ? "Aucun serveur primaire n'est disponible : rallumez ou reconnectez vos serveurs."
            : "Un seul serveur détient vos données : si celui-ci tombe, les écritures récentes peuvent être perdues. Rallumez ou reconnectez l'autre serveur."}
        </div>
      )}

      {/* Témoin etcd conseillé (ADR-004, décision 1) */}
      {besoin_temoin && (
        <div
          style={{
            background: '#FFFBEB',
            border: '1px solid #FCD34D',
            borderRadius: '10px',
            padding: '16px 20px',
            marginBottom: '24px',
            fontSize: '0.875rem',
            color: '#92400E',
          }}
        >
          <h3 style={{ fontSize: '0.875rem', fontWeight: 700, margin: '0 0 8px 0', display: 'flex', alignItems: 'center', gap: '8px' }}>
            <AlertCircle size={16} /> Installez un témoin sur une troisième machine
          </h3>
          <p style={{ margin: '0 0 12px 0' }}>
            Vos deux serveurs ont besoin d'un troisième avis pour décider seuls d'une bascule :
            sans témoin, la panne de l'un d'eux bloque toutes les écritures. Le témoin ne stocke
            aucune donnée, ne consomme pas de poste de licence, et n'importe quel PC suffit.
          </p>
          <div style={{ display: 'flex', gap: '10px', flexWrap: 'wrap' }}>
            <a
              href={downloadInstallerUrl('windows', 'temoin')}
              style={{
                display: 'flex', alignItems: 'center', gap: '8px', padding: '8px 14px',
                border: '1px solid #D97706', borderRadius: '8px', background: '#fff',
                color: '#92400E', fontSize: '0.85rem', fontWeight: 600, textDecoration: 'none',
              }}
            >
              <Monitor size={14} /> Installeur témoin Windows
            </a>
            <a
              href={downloadInstallerUrl('linux', 'temoin')}
              style={{
                display: 'flex', alignItems: 'center', gap: '8px', padding: '8px 14px',
                border: '1px solid #D97706', borderRadius: '8px', background: '#fff',
                color: '#92400E', fontSize: '0.85rem', fontWeight: 600, textDecoration: 'none',
              }}
            >
              <Terminal size={14} /> Installeur témoin Linux
            </a>
          </div>
        </div>
      )}

      {/* Machines */}
      <h2 style={{ fontSize: '1rem', fontWeight: 700, marginBottom: '16px', color: 'var(--text-primary)' }}>
        Vos machines ({devices.length})
      </h2>

      {devices.length > 0 ? (
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))', gap: '16px', marginBottom: '32px' }}>
          {devices.map((device) => (
            <DeviceCard key={device.id} device={device} />
          ))}
        </div>
      ) : (
        <div style={{ textAlign: 'center', padding: '48px 24px', color: 'var(--text-muted)' }}>
          <Inbox size={40} style={{ marginBottom: '16px', opacity: 0.4 }} />
          <p>Aucune machine connectée pour l'instant.</p>
          <Link
            to="/bienvenue"
            style={{
              display: 'inline-flex',
              alignItems: 'center',
              gap: '6px',
              marginTop: '12px',
              color: 'var(--crimson)',
              fontWeight: 600,
              textDecoration: 'none',
              fontSize: '0.85rem',
            }}
          >
            Voir les instructions d'installation
          </Link>
        </div>
      )}

      {/* Relais */}
      {relay_url && (
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '0.78rem', color: 'var(--text-muted)' }}>
          <Radio size={13} /> Relais zero-knowledge : <span style={{ fontFamily: 'monospace' }}>{relay_url}</span>
        </div>
      )}
    </div>
  )
}
