// Supervision des clusters — remplace templates/devices/clusters.html.
// Données réelles via GET /api/clusters/.

import { useCallback, useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import {
  AlertCircle,
  AlertTriangle,
  Loader2,
  Network,
  RotateCw,
  ShieldAlert,
} from 'lucide-react'
import { getClusterOverview, normalizeApiError, toggleDeviceActive } from '../api/client'
import type { ClusterEntry, ClusterHealth, ClusterNodeStatus, ClusterOverview } from '../types/index'

const HEALTH_LABELS: Record<ClusterHealth, string> = {
  ok: 'Cluster sain',
  empty: 'Aucun nœud',
  no_primary: 'Pas de primaire',
  primary_offline: 'Primaire hors ligne',
  no_standby: 'Bascule manuelle uniquement',
  over_license: 'Hors licence',
}

const HEALTH_COLORS: Record<ClusterHealth, { bg: string; fg: string }> = {
  ok: { bg: '#D1FAE5', fg: '#065F46' },
  empty: { bg: '#F3F4F6', fg: '#6B7280' },
  no_primary: { bg: '#FEE2E2', fg: '#991B1B' },
  primary_offline: { bg: '#FEE2E2', fg: '#991B1B' },
  no_standby: { bg: '#FEF3C7', fg: '#92400E' },
  over_license: { bg: '#FEF3C7', fg: '#92400E' },
}

const HEALTH_MESSAGES: Partial<Record<ClusterHealth, string>> = {
  no_standby: 'Un seul nœud — bascule manuelle uniquement en cas de panne',
  primary_offline: 'Le nœud primaire ne répond plus — intervention requise',
  no_primary: 'Aucun primaire détecté — le cluster ne peut pas accepter d\'écritures',
  over_license: 'Nombre de machines actives supérieur au quota de la licence',
}

function formatDateTime(iso: string): string {
  if (!iso) return '—'
  return new Date(iso).toLocaleString('fr-FR')
}

function LoadingState() {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: '10px', color: 'var(--text-secondary)', padding: '48px 0' }}>
      <Loader2 size={18} className="cl-spin" />
      <span>Chargement des clusters...</span>
      <style>{`.cl-spin { animation: cl-spin 0.8s linear infinite; } @keyframes cl-spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }`}</style>
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

function NodeRow({ node, kind }: { node: ClusterNodeStatus; kind: 'primary' | 'standby' }) {
  const label = kind === 'primary' ? 'Primaire' : 'Standby'
  const roleColor = kind === 'primary' ? { bg: '#F0FDF4', border: '#6EE7B7' } : { bg: '#FFFBEB', border: '#FCD34D' }
  const offline = !node.online

  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: '10px',
        padding: '9px 12px',
        borderRadius: '9px',
        background: offline ? '#FEF2F2' : roleColor.bg,
        border: `1px solid ${offline ? '#FECACA' : roleColor.border}`,
        opacity: offline ? 0.85 : 1,
      }}
    >
      <span
        style={{
          width: '8px',
          height: '8px',
          borderRadius: '50%',
          flexShrink: 0,
          background: offline ? '#EF4444' : kind === 'primary' ? '#10B981' : '#F59E0B',
        }}
      />
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ fontFamily: 'monospace', fontSize: '13px', fontWeight: 600, color: 'var(--crimson)' }}>
          {node.device.node_addr || node.device.hostname || 'Inconnu'}
        </div>
        <div style={{ fontSize: '11px', color: 'var(--text-muted)', marginTop: '2px' }}>
          {node.online ? 'En ligne' : 'Hors ligne'} · {formatDateTime(node.device.last_seen)}
        </div>
      </div>
      <span
        style={{
          fontSize: '11px',
          fontWeight: 700,
          padding: '2px 8px',
          borderRadius: '10px',
          whiteSpace: 'nowrap',
          background: offline ? '#FEE2E2' : kind === 'primary' ? '#D1FAE5' : '#FEF3C7',
          color: offline ? '#991B1B' : kind === 'primary' ? '#065F46' : '#92400E',
        }}
      >
        {offline ? 'Hors ligne' : label}
      </span>
    </div>
  )
}

function ClusterCard({ entry, onToggle, togglingId }: { entry: ClusterEntry; onToggle: (deviceId: string, currentlyActive: boolean) => void; togglingId: string | null }) {
  const colors = HEALTH_COLORS[entry.health]
  const message = HEALTH_MESSAGES[entry.health]
  const pct = entry.license.total > 0 ? Math.min(100, Math.round((entry.license.used / entry.license.total) * 100)) : 0

  return (
    <div
      style={{
        background: '#FFFFFF',
        border: `1px solid ${entry.is_alert ? '#FECACA' : 'var(--border)'}`,
        boxShadow: entry.is_alert ? '0 0 0 2px rgba(220,38,38,0.08)' : 'var(--shadow)',
        borderRadius: '14px',
        overflow: 'hidden',
        display: 'flex',
        flexDirection: 'column',
      }}
    >
      <div style={{ padding: '14px 18px 12px', borderBottom: '1px solid var(--border)', display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: '8px' }}>
        <Link to={`/tenants/${entry.tenant.id}`} style={{ fontWeight: 700, fontSize: '15px', color: 'var(--crimson)', textDecoration: 'none' }}>
          {entry.tenant.name}
        </Link>
        <span style={{ display: 'inline-flex', alignItems: 'center', gap: '5px', borderRadius: '20px', padding: '3px 10px', fontSize: '12px', fontWeight: 700, background: colors.bg, color: colors.fg }}>
          {HEALTH_LABELS[entry.health]}
        </span>
      </div>

      <div style={{ padding: '14px 18px', flex: 1 }}>
        {entry.total === 0 ? (
          <div style={{ textAlign: 'center', padding: '20px', color: 'var(--text-muted)', fontSize: '13px' }}>
            Aucune machine enregistrée pour ce tenant
          </div>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '10px', marginBottom: '10px' }}>
            {entry.primary && <NodeRow node={entry.primary} kind="primary" />}
            {entry.standbys.map((n) => (
              <NodeRow key={n.device.id} node={n} kind="standby" />
            ))}
            {!entry.primary && entry.standbys.length === 0 && (
              <div style={{ textAlign: 'center', padding: '16px', color: 'var(--text-muted)', fontSize: '13px' }}>Rôles non encore assignés</div>
            )}
          </div>
        )}

        {message && (
          <div style={{ display: 'flex', alignItems: 'center', gap: '6px', fontSize: '12px', color: '#991B1B', background: '#FEF2F2', borderRadius: '7px', padding: '6px 10px', marginTop: '10px' }}>
            <AlertTriangle size={13} />
            {message}
          </div>
        )}
      </div>

      <div style={{ padding: '10px 18px 14px', display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: '8px', flexWrap: 'wrap' }}>
        <div style={{ fontSize: '12px', color: 'var(--text-muted)' }}>
          {entry.license.plan ? (
            <>
              <strong style={{ color: 'var(--text-primary)' }}>{entry.license.plan}</strong> · {entry.license.used}/{entry.license.total} postes
              <span style={{ width: '80px', height: '5px', background: '#E5E7EB', borderRadius: '3px', display: 'inline-block', verticalAlign: 'middle', margin: '0 6px', overflow: 'hidden' }}>
                <span style={{ display: 'block', height: '100%', width: `${pct}%`, borderRadius: '3px', background: entry.license.ok ? '#10B981' : '#EF4444' }} />
              </span>
            </>
          ) : (
            <span style={{ color: 'var(--danger)' }}>Aucune licence active</span>
          )}
        </div>
        {entry.primary && (
          <button
            onClick={() => onToggle(entry.primary!.device.id, entry.primary!.device.is_active)}
            disabled={togglingId === entry.primary.device.id}
            style={{
              background: '#FEE8E8',
              color: 'var(--danger)',
              border: '1px solid #EF9A9A',
              borderRadius: '4px',
              padding: '5px 10px',
              fontSize: '11px',
              fontWeight: 600,
              cursor: togglingId === entry.primary.device.id ? 'default' : 'pointer',
              display: 'inline-flex',
              alignItems: 'center',
              gap: '6px',
            }}
          >
            {togglingId === entry.primary.device.id && <Loader2 size={12} className="cl-spin" />}
            Désactiver le primaire
          </button>
        )}
      </div>
    </div>
  )
}

export default function ClustersPage() {
  const [data, setData] = useState<ClusterOverview | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [togglingId, setTogglingId] = useState<string | null>(null)
  const [actionError, setActionError] = useState<string | null>(null)

  const load = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const overview = await getClusterOverview()
      setData(overview)
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

  async function handleToggle(deviceId: string, currentlyActive: boolean) {
    if (currentlyActive) {
      const ok = window.confirm('Désactiver ce nœud ?')
      if (!ok) return
    }
    setActionError(null)
    setTogglingId(deviceId)
    try {
      const result = await toggleDeviceActive(deviceId)
      setData((prev) => {
        if (!prev) return prev
        const clusters = prev.clusters.map((entry) => {
          const updateNode = (node: ClusterNodeStatus | null): ClusterNodeStatus | null =>
            node && node.device.id === result.id ? { ...node, device: { ...node.device, is_active: result.is_active } } : node
          return {
            ...entry,
            primary: updateNode(entry.primary),
            standbys: entry.standbys.map((n) => updateNode(n) as ClusterNodeStatus),
          }
        })
        return { ...prev, clusters }
      })
    } catch (err) {
      setActionError(normalizeApiError(err).message)
    } finally {
      setTogglingId(null)
    }
  }

  const okCount = data ? data.total_tenants - data.alerts_total : 0

  return (
    <div>
      <div style={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', marginBottom: '24px', gap: '16px', flexWrap: 'wrap' }}>
        <div>
          <h1 style={{ fontSize: '22px', fontWeight: 700, color: 'var(--text-primary)' }}>Supervision des clusters</h1>
          <p style={{ color: 'var(--text-secondary)', marginTop: '4px' }}>
            État des clusters PostgreSQL des tenants — nœud primaire, standbys et conformité de licence
          </p>
        </div>
        <Link
          to="/devices"
          style={{
            background: '#FFFFFF',
            border: '1px solid var(--border)',
            borderRadius: '6px',
            padding: '8px 14px',
            fontWeight: 600,
            fontSize: '13px',
            color: 'var(--text-primary)',
            textDecoration: 'none',
            display: 'flex',
            alignItems: 'center',
            gap: '6px',
          }}
        >
          <Network size={14} /> Vue liste des machines
        </Link>
      </div>

      {actionError && (
        <div
          role="alert"
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
            background: '#FDECEC',
            border: '1px solid #F3B7B7',
            color: 'var(--danger)',
            borderRadius: '8px',
            padding: '10px 14px',
            fontSize: '13px',
            marginBottom: '18px',
          }}
        >
          <AlertCircle size={16} />
          <span>{actionError}</span>
        </div>
      )}

      {loading && <LoadingState />}
      {!loading && error && <ErrorState message={error} onRetry={() => void load()} />}

      {!loading && !error && data && (
        <>
          <div style={{ display: 'flex', gap: '12px', marginBottom: '24px', flexWrap: 'wrap' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', background: '#FFFFFF', border: '1px solid var(--border)', borderRadius: '10px', padding: '10px 16px' }}>
              <div style={{ fontSize: '1.4rem', fontWeight: 800, color: 'var(--text-primary)' }}>{data.total_tenants}</div>
              <div style={{ fontSize: '.82rem', fontWeight: 600, color: 'var(--text-secondary)' }}>entreprises</div>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', background: '#FFFFFF', border: '1px solid var(--border)', borderRadius: '10px', padding: '10px 16px' }}>
              <div style={{ fontSize: '1.4rem', fontWeight: 800, color: data.alerts_total > 0 ? '#DC2626' : '#059669' }}>{data.alerts_total}</div>
              <div style={{ fontSize: '.82rem', fontWeight: 600, color: 'var(--text-secondary)' }}>alerte{data.alerts_total !== 1 ? 's' : ''}</div>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', background: '#FFFFFF', border: '1px solid var(--border)', borderRadius: '10px', padding: '10px 16px' }}>
              <div style={{ fontSize: '1.4rem', fontWeight: 800, color: '#059669' }}>{okCount}</div>
              <div style={{ fontSize: '.82rem', fontWeight: 600, color: 'var(--text-secondary)' }}>clusters OK</div>
            </div>
          </div>

          {data.clusters.length === 0 ? (
            <div style={{ textAlign: 'center', padding: '60px 20px', color: 'var(--text-muted)' }}>
              <ShieldAlert size={28} style={{ marginBottom: '8px', opacity: 0.5 }} />
              <p style={{ fontSize: '15px', fontWeight: 600, marginBottom: '6px' }}>Aucun cluster enregistré</p>
              <p style={{ fontSize: '13px' }}>Les nœuds PME s'enregistrent automatiquement au démarrage.</p>
            </div>
          ) : (
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(380px, 1fr))', gap: '20px' }}>
              {data.clusters.map((entry) => (
                <ClusterCard key={entry.tenant.id} entry={entry} onToggle={(id, active) => void handleToggle(id, active)} togglingId={togglingId} />
              ))}
            </div>
          )}
        </>
      )}
    </div>
  )
}
