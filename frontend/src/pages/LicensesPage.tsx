// Gestion des licences — CRUD réel via l'API. Plus aucune mutation en mémoire.

import type React from 'react'
import { useCallback, useEffect, useState } from 'react'
import { AlertCircle, KeyRound, Loader2, Pencil, Plus, PowerOff, RotateCw, Trash2, X } from 'lucide-react'
import {
  createLicense,
  deleteLicense,
  getApplications,
  getLicenses,
  getTenants,
  normalizeApiError,
  updateLicense,
  type LicenseWritePayload,
} from '../api/client'
import { PLAN_LABELS } from '../types/index'
import type { Application, FieldErrors, License, LicensePlan, Tenant } from '../types/index'
import StatusBadge from '../components/StatusBadge'

function formatDate(iso: string | null): string {
  if (!iso) return 'sans échéance'
  return new Date(iso).toLocaleDateString('fr-FR')
}

function toDateInput(iso: string | null): string {
  if (!iso) return ''
  return iso.slice(0, 10)
}

function LoadingState() {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: '10px', color: 'var(--text-secondary)', padding: '48px 0' }}>
      <Loader2 size={18} className="lp-spin" />
      <span>Chargement des licences...</span>
      <style>{`.lp-spin { animation: lp-spin 0.8s linear infinite; } @keyframes lp-spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }`}</style>
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

function FieldError({ messages }: { messages?: string[] }) {
  if (!messages || messages.length === 0) return null
  return <div style={{ color: 'var(--danger)', fontSize: '12px', marginTop: '4px' }}>{messages[0]}</div>
}

const inputStyle: React.CSSProperties = {
  width: '100%',
  padding: '8px 10px',
  borderRadius: '6px',
  border: '1px solid var(--border)',
  fontSize: '13px',
}

const labelStyle: React.CSSProperties = {
  fontSize: '12px',
  color: 'var(--text-secondary)',
  display: 'block',
  marginBottom: '4px',
  fontWeight: 600,
}

interface FormState {
  tenant: string
  plan: LicensePlan
  seats: number
  starts_at: string
  expires_at: string
  is_active: boolean
  /** UUID de l'application hébergée, '' pour aucune. */
  application: string
}

function emptyForm(defaultTenant: string): FormState {
  return {
    tenant: defaultTenant,
    plan: 'starter',
    seats: 1,
    starts_at: new Date().toISOString().slice(0, 10),
    expires_at: '',
    is_active: true,
    application: '',
  }
}

export default function LicensesPage() {
  const [licenses, setLicenses] = useState<License[]>([])
  const [tenants, setTenants] = useState<Tenant[]>([])
  const [applications, setApplications] = useState<Application[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [searchTerm, setSearchTerm] = useState('')
  const [filterStatus, setFilterStatus] = useState<'ALL' | 'ACTIVE' | 'INACTIVE'>('ALL')

  const [showModal, setShowModal] = useState(false)
  const [editingLicense, setEditingLicense] = useState<License | null>(null)
  const [form, setForm] = useState<FormState>(emptyForm(''))
  const [submitting, setSubmitting] = useState(false)
  const [formError, setFormError] = useState<string | null>(null)
  const [fieldErrors, setFieldErrors] = useState<FieldErrors | null>(null)

  const [rowActionId, setRowActionId] = useState<string | null>(null)
  const [rowError, setRowError] = useState<string | null>(null)

  const load = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const [licensesData, tenantsData, applicationsData] = await Promise.all([
        getLicenses(),
        getTenants(),
        getApplications(),
      ])
      setLicenses(licensesData)
      setTenants(tenantsData)
      setApplications(applicationsData)
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

  function openCreateModal() {
    setEditingLicense(null)
    setForm(emptyForm(tenants[0]?.id ?? ''))
    setFormError(null)
    setFieldErrors(null)
    setShowModal(true)
  }

  function openEditModal(lic: License) {
    setEditingLicense(lic)
    setForm({
      tenant: lic.tenant,
      plan: lic.plan,
      seats: lic.seats,
      starts_at: toDateInput(lic.starts_at),
      expires_at: toDateInput(lic.expires_at),
      is_active: lic.is_active,
      application: lic.application ?? '',
    })
    setFormError(null)
    setFieldErrors(null)
    setShowModal(true)
  }

  function closeModal() {
    setShowModal(false)
  }

  function updateField<K extends keyof FormState>(key: K, value: FormState[K]) {
    setForm((prev) => ({ ...prev, [key]: value }))
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setFormError(null)
    setFieldErrors(null)
    setSubmitting(true)
    const payload: LicenseWritePayload = {
      tenant: form.tenant,
      plan: form.plan,
      seats: form.seats,
      starts_at: form.starts_at,
      expires_at: form.expires_at || null,
      is_active: form.is_active,
      application: form.application || null,
    }
    try {
      if (editingLicense) {
        const updated = await updateLicense(editingLicense.id, payload)
        setLicenses((prev) => prev.map((l) => (l.id === updated.id ? updated : l)))
      } else {
        const created = await createLicense(payload)
        setLicenses((prev) => [created, ...prev])
      }
      setShowModal(false)
    } catch (err) {
      const apiError = normalizeApiError(err)
      setFormError(apiError.message)
      setFieldErrors(apiError.fieldErrors)
    } finally {
      setSubmitting(false)
    }
  }

  async function handleDelete(lic: License) {
    const ok = window.confirm(`Supprimer définitivement cette licence (${lic.tenant_name ?? 'tenant'} — ${PLAN_LABELS[lic.plan] || lic.plan}) ?`)
    if (!ok) return
    setRowError(null)
    setRowActionId(lic.id)
    try {
      await deleteLicense(lic.id)
      setLicenses((prev) => prev.filter((l) => l.id !== lic.id))
    } catch (err) {
      setRowError(normalizeApiError(err).message)
    } finally {
      setRowActionId(null)
    }
  }

  async function handleToggleActive(lic: License) {
    setRowError(null)
    setRowActionId(lic.id)
    try {
      const updated = await updateLicense(lic.id, { is_active: !lic.is_active })
      setLicenses((prev) => prev.map((l) => (l.id === updated.id ? updated : l)))
    } catch (err) {
      setRowError(normalizeApiError(err).message)
    } finally {
      setRowActionId(null)
    }
  }

  const filteredLicenses = licenses.filter((lic) => {
    const matchesSearch =
      (lic.tenant_name || '').toLowerCase().includes(searchTerm.toLowerCase()) ||
      (PLAN_LABELS[lic.plan] || lic.plan || '').toLowerCase().includes(searchTerm.toLowerCase())

    if (filterStatus === 'ACTIVE') return matchesSearch && lic.is_active
    if (filterStatus === 'INACTIVE') return matchesSearch && !lic.is_active
    return matchesSearch
  })

  const selectedTenant = tenants.find((t) => t.id === form.tenant) ?? null

  return (
    <div>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '28px' }}>
        <div>
          <h1 style={{ fontSize: '22px', fontWeight: 700, color: 'var(--text-primary)' }}>Licences</h1>
          <p style={{ color: 'var(--text-secondary)', marginTop: '4px' }}>
            Émission et gestion des licences des tenants
          </p>
        </div>
        <button
          onClick={openCreateModal}
          disabled={loading || tenants.length === 0}
          style={{
            background: 'var(--crimson)',
            color: '#FFFFFF',
            border: 'none',
            borderRadius: '6px',
            padding: '10px 16px',
            fontWeight: 600,
            cursor: loading || tenants.length === 0 ? 'default' : 'pointer',
            opacity: loading || tenants.length === 0 ? 0.6 : 1,
            boxShadow: 'var(--shadow)',
            display: 'flex',
            alignItems: 'center',
            gap: '6px',
          }}
        >
          <Plus size={15} /> Nouvelle licence
        </button>
      </div>

      {rowError && (
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
          <span>{rowError}</span>
        </div>
      )}

      {loading && <LoadingState />}
      {!loading && error && <ErrorState message={error} onRetry={() => void load()} />}

      {!loading && !error && (
        <>
          <div
            style={{
              background: '#FFFFFF',
              padding: '14px 20px',
              borderRadius: 'var(--radius)',
              border: '1px solid var(--border)',
              marginBottom: '20px',
              display: 'flex',
              gap: '16px',
              alignItems: 'center',
              justifyContent: 'space-between',
              flexWrap: 'wrap',
            }}
          >
            <input
              type="text"
              placeholder="Rechercher par tenant ou offre..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              style={{
                flex: 1,
                minWidth: '220px',
                maxWidth: '400px',
                padding: '8px 14px',
                borderRadius: '6px',
                border: '1px solid var(--border)',
                fontSize: '13px',
              }}
            />
            <div style={{ display: 'flex', gap: '8px' }}>
              {(['ALL', 'ACTIVE', 'INACTIVE'] as const).map((st) => (
                <button
                  key={st}
                  onClick={() => setFilterStatus(st)}
                  style={{
                    padding: '6px 12px',
                    borderRadius: '6px',
                    border: '1px solid var(--border)',
                    background: filterStatus === st ? 'var(--sidebar-bg)' : '#FFFFFF',
                    color: filterStatus === st ? '#FFFFFF' : 'var(--text-primary)',
                    fontWeight: 600,
                    fontSize: '12px',
                    cursor: 'pointer',
                  }}
                >
                  {st === 'ALL' ? 'Toutes' : st === 'ACTIVE' ? 'Actives' : 'Inactives'}
                </button>
              ))}
            </div>
          </div>

          <div style={{ background: '#FFFFFF', borderRadius: 'var(--radius)', boxShadow: 'var(--shadow)', overflow: 'hidden' }}>
            {licenses.length === 0 ? (
              <div style={{ textAlign: 'center', padding: '48px 20px', color: 'var(--text-muted)' }}>
                <KeyRound size={28} style={{ marginBottom: '8px', opacity: 0.5 }} />
                <p>Aucune licence enregistrée</p>
              </div>
            ) : filteredLicenses.length === 0 ? (
              <div style={{ textAlign: 'center', padding: '48px 20px', color: 'var(--text-muted)' }}>
                <p>Aucune licence ne correspond à ces critères</p>
              </div>
            ) : (
              <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '14px' }}>
                <thead style={{ background: '#F5EFE8', borderBottom: '2px solid var(--gold)' }}>
                  <tr>
                    <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Tenant</th>
                    <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Offre</th>
                    <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Sièges</th>
                    <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Application</th>
                    <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Début</th>
                    <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Expiration</th>
                    <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Statut</th>
                    <th style={{ padding: '12px 16px', textAlign: 'right', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {filteredLicenses.map((lic) => (
                    <tr key={lic.id} style={{ borderBottom: '1px solid var(--border)' }}>
                      <td style={{ padding: '14px 16px', fontWeight: 600 }}>{lic.tenant_name || '—'}</td>
                      <td style={{ padding: '14px 16px' }}>
                        <span style={{ background: '#FFF3CD', color: '#856404', padding: '2px 8px', borderRadius: '10px', fontSize: '12px', fontWeight: 600 }}>
                          {PLAN_LABELS[lic.plan] || lic.plan}
                        </span>
                      </td>
                      <td style={{ padding: '14px 16px', fontWeight: 600 }}>{lic.seats} poste{lic.seats > 1 ? 's' : ''}</td>
                      <td style={{ padding: '14px 16px', fontSize: '13px', color: lic.application_nom ? 'var(--text-primary)' : 'var(--text-muted)' }}>
                        {lic.application_nom || 'Aucune'}
                      </td>
                      <td style={{ padding: '14px 16px', fontSize: '12px', color: 'var(--text-secondary)' }}>{formatDate(lic.starts_at)}</td>
                      <td style={{ padding: '14px 16px' }}>{formatDate(lic.expires_at)}</td>
                      <td style={{ padding: '14px 16px' }}><StatusBadge active={lic.is_active} /></td>
                      <td style={{ padding: '14px 16px', textAlign: 'right' }}>
                        <div style={{ display: 'flex', gap: '8px', justifyContent: 'flex-end' }}>
                          <button
                            onClick={() => openEditModal(lic)}
                            title="Modifier la licence"
                            style={{
                              background: '#F5EFE8',
                              color: 'var(--text-primary)',
                              border: '1px solid var(--border)',
                              borderRadius: '4px',
                              padding: '5px 8px',
                              fontSize: '11px',
                              fontWeight: 600,
                              cursor: 'pointer',
                              display: 'flex',
                              alignItems: 'center',
                              gap: '4px',
                            }}
                          >
                            <Pencil size={12} /> Modifier
                          </button>
                          <button
                            onClick={() => void handleToggleActive(lic)}
                            disabled={rowActionId === lic.id}
                            title={lic.is_active ? 'Désactiver la licence' : 'Réactiver la licence'}
                            style={{
                              background: lic.is_active ? '#FEE8E8' : '#E8F5E9',
                              color: lic.is_active ? 'var(--danger)' : '#2E7D32',
                              border: `1px solid ${lic.is_active ? '#EF9A9A' : '#A5D6A7'}`,
                              borderRadius: '4px',
                              padding: '5px 8px',
                              fontSize: '11px',
                              fontWeight: 600,
                              cursor: rowActionId === lic.id ? 'default' : 'pointer',
                              display: 'flex',
                              alignItems: 'center',
                              gap: '4px',
                            }}
                          >
                            {rowActionId === lic.id ? <Loader2 size={12} className="lp-spin" /> : <PowerOff size={12} />}
                            {lic.is_active ? 'Désactiver' : 'Réactiver'}
                          </button>
                          <button
                            onClick={() => void handleDelete(lic)}
                            disabled={rowActionId === lic.id}
                            title="Supprimer la licence"
                            style={{
                              background: 'transparent',
                              color: 'var(--danger)',
                              border: 'none',
                              borderRadius: '4px',
                              padding: '5px 8px',
                              fontSize: '11px',
                              fontWeight: 600,
                              cursor: rowActionId === lic.id ? 'default' : 'pointer',
                              display: 'flex',
                              alignItems: 'center',
                              gap: '4px',
                            }}
                          >
                            <Trash2 size={12} /> Supprimer
                          </button>
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>
        </>
      )}

      {/* Modal de création / édition */}
      {showModal && (
        <div
          style={{
            position: 'fixed',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            background: 'rgba(0,0,0,0.6)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 1000,
          }}
        >
          <div style={{ background: '#FFFFFF', borderRadius: '12px', padding: '28px', maxWidth: '480px', width: '100%', boxShadow: 'var(--shadow)' }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '8px' }}>
              <h3 style={{ fontSize: '18px', fontWeight: 700 }}>{editingLicense ? 'Modifier la licence' : 'Nouvelle licence'}</h3>
              <button onClick={closeModal} style={{ background: 'transparent', border: 'none', cursor: 'pointer', color: 'var(--text-muted)' }}>
                <X size={18} />
              </button>
            </div>
            <p style={{ fontSize: '13px', color: 'var(--text-secondary)', marginBottom: '20px' }}>
              Le nombre de sièges ne peut pas dépasser l'effectif déclaré du tenant
              {selectedTenant ? ` (${selectedTenant.employee_count} employé${selectedTenant.employee_count > 1 ? 's' : ''})` : ''} : l'API rejette
              toute licence dont seats &gt; employee_count.
            </p>

            {formError && (
              <div
                role="alert"
                style={{
                  display: 'flex',
                  alignItems: 'flex-start',
                  gap: '8px',
                  background: '#FDECEC',
                  border: '1px solid #F3B7B7',
                  color: 'var(--danger)',
                  borderRadius: '8px',
                  padding: '10px 12px',
                  fontSize: '13px',
                  marginBottom: '16px',
                }}
              >
                <AlertCircle size={16} style={{ flexShrink: 0, marginTop: '1px' }} />
                <span>{formError}</span>
              </div>
            )}

            <form onSubmit={handleSubmit} style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>
              <div>
                <label style={labelStyle}>Tenant</label>
                <select
                  value={form.tenant}
                  onChange={(e) => updateField('tenant', e.target.value)}
                  required
                  style={inputStyle}
                >
                  <option value="" disabled>Sélectionner un tenant</option>
                  {tenants.map((t) => (
                    <option key={t.id} value={t.id}>{t.name} ({t.employee_count} employés)</option>
                  ))}
                </select>
                <FieldError messages={fieldErrors?.tenant} />
              </div>

              <div>
                <label style={labelStyle}>Offre</label>
                <select
                  value={form.plan}
                  onChange={(e) => updateField('plan', e.target.value as LicensePlan)}
                  style={inputStyle}
                >
                  <option value="starter">Starter</option>
                  <option value="pro">Pro</option>
                  <option value="enterprise">Enterprise</option>
                </select>
                <FieldError messages={fieldErrors?.plan} />
              </div>

              <div>
                <label style={labelStyle}>Sièges (postes autorisés)</label>
                <input
                  type="number"
                  min={1}
                  required
                  value={form.seats}
                  onChange={(e) => updateField('seats', parseInt(e.target.value, 10) || 1)}
                  style={inputStyle}
                />
                <FieldError messages={fieldErrors?.seats} />
              </div>

              <div style={{ display: 'flex', gap: '12px' }}>
                <div style={{ flex: 1 }}>
                  <label style={labelStyle}>Date de début</label>
                  <input
                    type="date"
                    required
                    value={form.starts_at}
                    onChange={(e) => updateField('starts_at', e.target.value)}
                    style={inputStyle}
                  />
                  <FieldError messages={fieldErrors?.starts_at} />
                </div>
                <div style={{ flex: 1 }}>
                  <label style={labelStyle}>Date d'expiration (optionnelle)</label>
                  <input
                    type="date"
                    value={form.expires_at}
                    onChange={(e) => updateField('expires_at', e.target.value)}
                    style={inputStyle}
                  />
                  <FieldError messages={fieldErrors?.expires_at} />
                </div>
              </div>

              <div>
                <label style={labelStyle}>Application hébergée</label>
                <select
                  value={form.application}
                  onChange={(e) => updateField('application', e.target.value)}
                  style={inputStyle}
                >
                  <option value="">Aucune</option>
                  {applications
                    .filter((a) => a.statut === 'validee' || a.id === form.application)
                    .map((a) => (
                      <option key={a.id} value={a.id}>
                        {a.nom} {a.version} — {a.editeur}{a.statut !== 'validee' ? ' (retirée)' : ''}
                      </option>
                    ))}
                </select>
                <div style={{ fontSize: '11px', color: 'var(--text-muted)', marginTop: '4px' }}>
                  Seules les applications validées se rattachent. La PME la reçoit en régénérant sa pile.
                </div>
                <FieldError messages={fieldErrors?.application} />
              </div>

              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <input
                  id="license-active"
                  type="checkbox"
                  checked={form.is_active}
                  onChange={(e) => updateField('is_active', e.target.checked)}
                />
                <label htmlFor="license-active" style={{ fontSize: '13px', color: 'var(--text-primary)' }}>
                  Licence active
                </label>
              </div>

              <div style={{ display: 'flex', gap: '10px', marginTop: '10px' }}>
                <button
                  type="button"
                  onClick={closeModal}
                  style={{ flex: 1, padding: '10px', background: '#FFF', border: '1px solid var(--border)', borderRadius: '6px', cursor: 'pointer' }}
                >
                  Annuler
                </button>
                <button
                  type="submit"
                  disabled={submitting}
                  style={{
                    flex: 1,
                    padding: '10px',
                    background: submitting ? '#C46A72' : 'var(--crimson)',
                    color: '#FFF',
                    border: 'none',
                    borderRadius: '6px',
                    fontWeight: 600,
                    cursor: submitting ? 'default' : 'pointer',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    gap: '8px',
                  }}
                >
                  {submitting && <Loader2 size={14} className="lp-spin" />}
                  {editingLicense ? 'Enregistrer' : 'Émettre la licence'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  )
}
