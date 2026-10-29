// Parc machines — remplace templates/devices/list.html.
// Liste réelle via GET /api/devices/ ; bascule actif/inactif via toggleDeviceActive().

import { useCallback, useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { AlertCircle, HardDrive, Loader2, Power, PowerOff, RotateCw } from 'lucide-react'
import { getDevices, normalizeApiError, toggleDeviceActive } from '../api/client'
import StatusBadge from '../components/StatusBadge'
import { NODE_ROLE_LABELS, isDeviceOnline } from '../types/index'
import EtatsNoeud from '../components/EtatsNoeud'
import type { Device } from '../types/index'

function formatDateTime(iso: string): string {
  if (!iso) return '—'
  return new Date(iso).toLocaleString('fr-FR')
}

function LoadingState() {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: '10px', color: 'var(--text-secondary)', padding: '48px 0' }}>
      <Loader2 size={18} className="dv-spin" />
      <span>Chargement du parc machines...</span>
      <style>{`.dv-spin { animation: dv-spin 0.8s linear infinite; } @keyframes dv-spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }`}</style>
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

export default function DevicesPage() {
  const [devices, setDevices] = useState<Device[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [togglingId, setTogglingId] = useState<string | null>(null)
  const [actionError, setActionError] = useState<string | null>(null)

  const load = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const data = await getDevices()
      setDevices(data)
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

  async function handleToggle(device: Device) {
    if (device.is_active) {
      const ok = window.confirm('Désactiver cette machine ? Pensez à la rotation DEK côté cluster.')
      if (!ok) return
    }
    setActionError(null)
    setTogglingId(device.id)
    try {
      const result = await toggleDeviceActive(device.id)
      setDevices((prev) => prev.map((d) => (d.id === result.id ? { ...d, is_active: result.is_active } : d)))
    } catch (err) {
      setActionError(normalizeApiError(err).message)
    } finally {
      setTogglingId(null)
    }
  }

  return (
    <div>
      <div style={{ marginBottom: '24px' }}>
        <h1 style={{ fontSize: '22px', fontWeight: 700, color: 'var(--text-primary)' }}>Parc machines</h1>
        <p style={{ color: 'var(--text-secondary)', marginTop: '4px' }}>
          Suivi des installations — identifiant UUID, pas l'adresse MAC
        </p>
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

      {!loading && !error && (
        <div style={{ background: '#FFFFFF', borderRadius: 'var(--radius)', boxShadow: 'var(--shadow)', overflow: 'hidden' }}>
          {devices.length === 0 ? (
            <div style={{ textAlign: 'center', padding: '48px 20px', color: 'var(--text-muted)' }}>
              <HardDrive size={28} style={{ marginBottom: '8px', opacity: 0.5 }} />
              <p>Aucune machine enregistrée</p>
            </div>
          ) : (
            <div style={{ overflowX: 'auto' }}>
              <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '13px', minWidth: '960px' }}>
                <thead style={{ background: '#F5EFE8', borderBottom: '2px solid var(--gold)' }}>
                  <tr>
                    <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Tenant</th>
                    <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Hostname</th>
                    <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Rôle</th>
                    <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Adresse annoncée</th>
                    <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>OS</th>
                    <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Dernière vue</th>
                    <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>En ligne</th>
                    <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Statut</th>
                    <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Licence</th>
                    <th style={{ padding: '12px 16px', textAlign: 'right', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {devices.map((dev) => {
                    const online = isDeviceOnline(dev)
                    return (
                      <tr key={dev.id} style={{ borderBottom: '1px solid var(--border)' }}>
                        <td style={{ padding: '12px 16px' }}>
                          <Link to={`/tenants/${dev.tenant}`} style={{ color: 'var(--crimson)', fontWeight: 600, textDecoration: 'none' }}>
                            {dev.tenant_name || '—'}
                          </Link>
                        </td>
                        <td style={{ padding: '12px 16px', fontWeight: 600 }}>
                          {dev.hostname || '—'}
                          <EtatsNoeud device={dev} />
                        </td>
                        <td style={{ padding: '12px 16px' }}>
                          <span
                            style={{
                              fontSize: '11px',
                              fontWeight: 700,
                              padding: '2px 8px',
                              borderRadius: '10px',
                              background: dev.node_role === 'primary' ? '#D1FAE5' : dev.node_role === 'standby' ? '#FEF3C7' : 'transparent',
                              color: dev.node_role === 'primary' ? '#065F46' : dev.node_role === 'standby' ? '#92400E' : 'var(--text-muted)',
                            }}
                          >
                            {NODE_ROLE_LABELS[dev.node_role] || 'Rôle inconnu'}
                          </span>
                        </td>
                        <td style={{ padding: '12px 16px', fontFamily: 'monospace' }}>{dev.node_addr || '—'}</td>
                        <td style={{ padding: '12px 16px' }}>{dev.os || '—'}</td>
                        <td style={{ padding: '12px 16px', color: 'var(--text-muted)' }}>{formatDateTime(dev.last_seen)}</td>
                        <td style={{ padding: '12px 16px' }}>
                          <StatusBadge active={online} label={online ? 'En ligne' : 'Hors ligne'} />
                        </td>
                        <td style={{ padding: '12px 16px' }}><StatusBadge active={dev.is_active} /></td>
                        <td style={{ padding: '12px 16px' }}>
                          {dev.is_within_license ? (
                            <span style={{ color: '#2E7D32', fontWeight: 600 }}>OK</span>
                          ) : (
                            <span style={{ color: 'var(--danger)', fontWeight: 600 }}>Hors licence</span>
                          )}
                        </td>
                        <td style={{ padding: '12px 16px', textAlign: 'right' }}>
                          <button
                            onClick={() => void handleToggle(dev)}
                            disabled={togglingId === dev.id}
                            style={{
                              display: 'inline-flex',
                              alignItems: 'center',
                              gap: '6px',
                              background: dev.is_active ? '#FEE8E8' : '#E8F5E9',
                              color: dev.is_active ? 'var(--danger)' : '#2E7D32',
                              border: `1px solid ${dev.is_active ? '#EF9A9A' : '#A5D6A7'}`,
                              borderRadius: '4px',
                              padding: '5px 10px',
                              fontSize: '11px',
                              fontWeight: 600,
                              cursor: togglingId === dev.id ? 'default' : 'pointer',
                            }}
                          >
                            {togglingId === dev.id ? (
                              <Loader2 size={12} className="dv-spin" />
                            ) : dev.is_active ? (
                              <PowerOff size={12} />
                            ) : (
                              <Power size={12} />
                            )}
                            {dev.is_active ? 'Désactiver' : 'Réactiver'}
                          </button>
                        </td>
                      </tr>
                    )
                  })}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}
    </div>
  )
}
