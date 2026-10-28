// Fiche détaillée d'un tenant — remplace templates/tenants/detail.html.
// Tenant, ses licences et ses appareils : données réelles uniquement.

import { useCallback, useEffect, useState } from 'react'
import { useNavigate, useParams, Link } from 'react-router-dom'
import { AlertCircle, KeyRound, Loader2, Pencil, Power, PowerOff, Printer, RotateCw, Trash2 } from 'lucide-react'
import { deleteTenant, getDevices, getLicenses, getTenant, normalizeApiError, toggleDeviceActive } from '../api/client'
import StatusBadge from '../components/StatusBadge'
import { NODE_ROLE_LABELS, PLAN_LABELS, isDeviceOnline } from '../types/index'
import type { Device, License, Tenant } from '../types/index'

function formatDate(iso: string | null): string {
  if (!iso) return 'sans échéance'
  return new Date(iso).toLocaleDateString('fr-FR')
}

function formatDateTime(iso: string): string {
  return new Date(iso).toLocaleString('fr-FR')
}

function getInitials(name: string): string {
  return name.slice(0, 2).toUpperCase()
}

function LoadingState({ label }: { label: string }) {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: '10px', color: 'var(--text-secondary)', padding: '48px 0' }}>
      <Loader2 size={18} className="td-spin" />
      <span>{label}</span>
      <style>{`.td-spin { animation: td-spin 0.8s linear infinite; } @keyframes td-spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }`}</style>
    </div>
  )
}

export default function TenantDetail() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()

  const [tenant, setTenant] = useState<Tenant | null>(null)
  const [licenses, setLicenses] = useState<License[]>([])
  const [devices, setDevices] = useState<Device[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const [togglingId, setTogglingId] = useState<string | null>(null)
  const [actionError, setActionError] = useState<string | null>(null)
  const [deleting, setDeleting] = useState(false)

  const load = useCallback(async () => {
    if (!id) return
    setLoading(true)
    setError(null)
    try {
      const [tenantData, licensesData, devicesData] = await Promise.all([
        getTenant(id),
        getLicenses(id),
        getDevices(id),
      ])
      setTenant(tenantData)
      setLicenses(licensesData)
      setDevices(devicesData)
    } catch (err) {
      setError(normalizeApiError(err).message)
    } finally {
      setLoading(false)
    }
  }, [id])

  useEffect(() => {
    // Chargement initial differe d'une micro-tache : appeler load() dans la
    // phase synchrone de l'effet declenche des rendus en cascade (React 19,
    // regle react-hooks/set-state-in-effect).
    queueMicrotask(() => {
      void load()
    })
  }, [load])

  async function handleToggleDevice(device: Device) {
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

  async function handleDeleteTenant() {
    if (!tenant) return
    const ok = window.confirm(`Supprimer définitivement le tenant « ${tenant.name} » ? Cette action est irréversible.`)
    if (!ok) return
    setActionError(null)
    setDeleting(true)
    try {
      await deleteTenant(tenant.id)
      navigate('/tenants', { replace: true })
    } catch (err) {
      setActionError(normalizeApiError(err).message)
      setDeleting(false)
    }
  }

  if (loading) return <LoadingState label="Chargement du dossier client..." />

  if (error) {
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
          <span>{error}</span>
        </div>
        <button
          onClick={() => void load()}
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

  if (!tenant) return <div style={{ color: 'var(--danger)' }}>Tenant introuvable.</div>

  return (
    <div>
      {/* Bandeau supérieur */}
      <div className="no-print" style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '24px' }}>
        <div style={{ fontSize: '14px', color: 'var(--text-secondary)' }}>
          <Link to="/tenants" style={{ color: 'var(--crimson)', fontWeight: 500, textDecoration: 'none' }}>← Tenants</Link>
          <span> / {tenant.name}</span>
        </div>
        <div style={{ display: 'flex', gap: '10px' }}>
          <button
            onClick={() => window.print()}
            className="no-print"
            style={{
              background: '#FFFFFF',
              border: '1px solid var(--border)',
              padding: '8px 14px',
              borderRadius: '6px',
              fontWeight: 600,
              fontSize: '13px',
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
            }}
          >
            <Printer size={14} /> Imprimer
          </button>
          <Link
            to={`/tenants/${tenant.id}/modifier`}
            style={{
              background: '#FFFFFF',
              border: '1px solid var(--border)',
              padding: '8px 14px',
              borderRadius: '6px',
              fontWeight: 600,
              fontSize: '13px',
              color: 'var(--text-primary)',
              textDecoration: 'none',
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
            }}
          >
            <Pencil size={14} /> Modifier
          </Link>
          <button
            onClick={() => void handleDeleteTenant()}
            disabled={deleting}
            style={{
              background: 'var(--danger)',
              border: 'none',
              padding: '8px 14px',
              borderRadius: '6px',
              fontWeight: 600,
              fontSize: '13px',
              color: '#FFFFFF',
              cursor: deleting ? 'default' : 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
            }}
          >
            {deleting ? <Loader2 size={14} className="td-spin" /> : <Trash2 size={14} />} Supprimer
          </button>
        </div>
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

      {/* Fiche entreprise */}
      <div
        style={{
          background: '#FFFFFF',
          border: '1px solid var(--border)',
          borderRadius: 'var(--radius)',
          padding: '24px',
          boxShadow: 'var(--shadow)',
          marginBottom: '28px',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '16px', marginBottom: '8px' }}>
          <div
            style={{
              width: '48px',
              height: '48px',
              borderRadius: '50%',
              background: 'var(--crimson)',
              color: '#FFFFFF',
              fontSize: '20px',
              fontWeight: 700,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            {getInitials(tenant.name)}
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
            <span style={{ fontSize: '20px', fontWeight: 700, color: 'var(--text-primary)' }}>{tenant.name}</span>
            <StatusBadge active={tenant.is_active} />
          </div>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: '16px', marginTop: '20px' }}>
          <div>
            <div style={{ color: 'var(--text-secondary)', fontSize: '12px' }}>Email de contact</div>
            <div style={{ color: 'var(--text-primary)', fontWeight: 600 }}>{tenant.email}</div>
          </div>
          <div>
            <div style={{ color: 'var(--text-secondary)', fontSize: '12px' }}>Téléphone</div>
            <div style={{ color: 'var(--text-primary)', fontWeight: 600 }}>{tenant.phone || '—'}</div>
          </div>
          <div>
            <div style={{ color: 'var(--text-secondary)', fontSize: '12px' }}>Créé le</div>
            <div style={{ color: 'var(--text-primary)', fontWeight: 600 }}>{formatDate(tenant.created_at)}</div>
          </div>
          <div style={{ gridColumn: '1 / -1' }}>
            <div style={{ color: 'var(--text-secondary)', fontSize: '12px' }}>Adresse</div>
            <div style={{ color: 'var(--text-primary)', fontWeight: 600 }}>{tenant.address || '—'}</div>
          </div>
          <div>
            <div style={{ color: 'var(--text-secondary)', fontSize: '12px' }}>Employés (postes autorisés)</div>
            <div style={{ fontSize: '22px', fontWeight: 700, color: 'var(--crimson)' }}>{tenant.employee_count}</div>
          </div>
          <div style={{ gridColumn: '1 / -1' }}>
            <div style={{ color: 'var(--text-secondary)', fontSize: '12px' }}>Jeton d'enregistrement client</div>
            <div
              style={{
                fontFamily: 'monospace',
                fontSize: '13px',
                background: '#F5EFE8',
                padding: '8px 12px',
                borderRadius: '6px',
                userSelect: 'all',
                marginTop: '4px',
              }}
            >
              {tenant.registration_token}
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '6px', color: 'var(--text-muted)', fontSize: '12px', marginTop: '6px' }}>
              <KeyRound size={13} />
              À communiquer à la PME lors de l'installation du logiciel. Ne pas partager publiquement.
            </div>
          </div>
        </div>
      </div>

      {/* Licences */}
      <div style={{ marginBottom: '28px' }}>
        <h2 style={{ borderLeft: '3px solid var(--crimson)', paddingLeft: '12px', fontSize: '16px', fontWeight: 600, marginBottom: '16px' }}>
          Licences
        </h2>
        <div style={{ background: '#FFFFFF', borderRadius: 'var(--radius)', boxShadow: 'var(--shadow)', overflow: 'hidden' }}>
          {licenses.length === 0 ? (
            <div style={{ textAlign: 'center', color: 'var(--text-muted)', padding: '24px' }}>Aucune licence</div>
          ) : (
            <table style={{ width: '100%', borderCollapse: 'collapse' }}>
              <thead style={{ background: '#F5EFE8' }}>
                <tr>
                  <th style={{ padding: '12px 16px', textAlign: 'left', fontSize: '11px', color: 'var(--text-secondary)' }}>Offre</th>
                  <th style={{ padding: '12px 16px', textAlign: 'left', fontSize: '11px', color: 'var(--text-secondary)' }}>Sièges</th>
                  <th style={{ padding: '12px 16px', textAlign: 'left', fontSize: '11px', color: 'var(--text-secondary)' }}>Début</th>
                  <th style={{ padding: '12px 16px', textAlign: 'left', fontSize: '11px', color: 'var(--text-secondary)' }}>Fin</th>
                  <th style={{ padding: '12px 16px', textAlign: 'left', fontSize: '11px', color: 'var(--text-secondary)' }}>Statut</th>
                </tr>
              </thead>
              <tbody>
                {licenses.map((lic) => (
                  <tr key={lic.id}>
                    <td style={{ padding: '14px 16px', fontWeight: 600 }}>{PLAN_LABELS[lic.plan] || lic.plan}</td>
                    <td style={{ padding: '14px 16px' }}>{lic.seats} poste{lic.seats > 1 ? 's' : ''}</td>
                    <td style={{ padding: '14px 16px' }}>{formatDate(lic.starts_at)}</td>
                    <td style={{ padding: '14px 16px' }}>{formatDate(lic.expires_at)}</td>
                    <td style={{ padding: '14px 16px' }}><StatusBadge active={lic.is_active} /></td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </div>

      {/* Machines / nœuds du cluster */}
      <div style={{ marginBottom: '28px' }}>
        <h2 style={{ borderLeft: '3px solid var(--crimson)', paddingLeft: '12px', fontSize: '16px', fontWeight: 600, marginBottom: '16px' }}>
          Parc machines ({devices.length} / {tenant.employee_count})
        </h2>

        <div style={{ background: '#FFFFFF', borderRadius: 'var(--radius)', boxShadow: 'var(--shadow)', overflow: 'hidden' }}>
          {devices.length === 0 ? (
            <div style={{ textAlign: 'center', color: 'var(--text-muted)', padding: '24px' }}>Aucune machine</div>
          ) : (
            <table style={{ width: '100%', borderCollapse: 'collapse' }}>
              <thead style={{ background: '#F5EFE8' }}>
                <tr>
                  <th style={{ padding: '12px 16px', textAlign: 'left', fontSize: '11px', color: 'var(--text-secondary)' }}>Hostname</th>
                  <th style={{ padding: '12px 16px', textAlign: 'left', fontSize: '11px', color: 'var(--text-secondary)' }}>Rôle</th>
                  <th style={{ padding: '12px 16px', textAlign: 'left', fontSize: '11px', color: 'var(--text-secondary)' }}>Adresse annoncée</th>
                  <th style={{ padding: '12px 16px', textAlign: 'left', fontSize: '11px', color: 'var(--text-secondary)' }}>Dernière vue</th>
                  <th style={{ padding: '12px 16px', textAlign: 'left', fontSize: '11px', color: 'var(--text-secondary)' }}>En ligne</th>
                  <th style={{ padding: '12px 16px', textAlign: 'left', fontSize: '11px', color: 'var(--text-secondary)' }}>Licence</th>
                  <th style={{ padding: '12px 16px', textAlign: 'right', fontSize: '11px', color: 'var(--text-secondary)' }}>Actions</th>
                </tr>
              </thead>
              <tbody>
                {devices.map((dev) => {
                  const online = isDeviceOnline(dev)
                  return (
                    <tr key={dev.id} className="no-print">
                      <td style={{ padding: '14px 16px', fontWeight: 600 }}>{dev.hostname || '—'}</td>
                      <td style={{ padding: '14px 16px' }}>
                        <span
                          style={{
                            fontSize: '11px',
                            fontWeight: 700,
                            padding: '2px 8px',
                            borderRadius: '10px',
                            background: dev.node_role === 'primary' ? '#E8F5E9' : dev.node_role === 'standby' ? '#FFF3CD' : '#EEEEEE',
                            color: dev.node_role === 'primary' ? '#2E7D32' : dev.node_role === 'standby' ? '#856404' : '#616161',
                          }}
                        >
                          {NODE_ROLE_LABELS[dev.node_role] || 'Rôle inconnu'}
                        </span>
                      </td>
                      <td style={{ padding: '14px 16px', fontFamily: 'monospace' }}>{dev.node_addr || '—'}</td>
                      <td style={{ padding: '14px 16px', fontSize: '12px', color: 'var(--text-muted)' }}>{formatDateTime(dev.last_seen)}</td>
                      <td style={{ padding: '14px 16px' }}>
                        <StatusBadge active={online} label={online ? 'En ligne' : 'Hors ligne'} />
                      </td>
                      <td style={{ padding: '14px 16px' }}>
                        {dev.is_within_license ? (
                          <span style={{ color: '#2E7D32', fontWeight: 600, fontSize: '12px' }}>OK</span>
                        ) : (
                          <span style={{ color: 'var(--danger)', fontWeight: 600, fontSize: '12px' }}>Hors licence</span>
                        )}
                      </td>
                      <td style={{ padding: '14px 16px', textAlign: 'right' }}>
                        <button
                          onClick={() => void handleToggleDevice(dev)}
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
                            <Loader2 size={12} className="td-spin" />
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
          )}
        </div>
      </div>
    </div>
  )
}
