// Formulaire de création / édition d'un tenant — remplace templates/tenants/form.html.
// Présence d'un :id dans l'URL = édition ; sinon création. Tout passe par l'API.

import type React from 'react'
import { useCallback, useEffect, useState } from 'react'
import { Link, useNavigate, useParams } from 'react-router-dom'
import { AlertCircle, Loader2, RotateCw, Save } from 'lucide-react'
import { createTenant, getTenant, normalizeApiError, updateTenant, type TenantWritePayload } from '../api/client'
import type { FieldErrors } from '../types/index'

const EMPTY_FORM: TenantWritePayload = {
  name: '',
  email: '',
  phone: '',
  address: '',
  employee_count: 1,
  is_active: true,
}

const inputStyle: React.CSSProperties = {
  width: '100%',
  padding: '10px 12px',
  borderRadius: '6px',
  border: '1px solid var(--border)',
  fontSize: '14px',
}

const labelStyle: React.CSSProperties = {
  display: 'block',
  fontSize: '12px',
  fontWeight: 600,
  color: 'var(--text-secondary)',
  marginBottom: '5px',
}

function FieldError({ messages }: { messages?: string[] }) {
  if (!messages || messages.length === 0) return null
  return <div style={{ color: 'var(--danger)', fontSize: '12px', marginTop: '4px' }}>{messages[0]}</div>
}

function LoadingState() {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: '10px', color: 'var(--text-secondary)', padding: '48px 0' }}>
      <Loader2 size={18} className="tf-spin" />
      <span>Chargement du tenant...</span>
      <style>{`.tf-spin { animation: tf-spin 0.8s linear infinite; } @keyframes tf-spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }`}</style>
    </div>
  )
}

export default function TenantFormPage() {
  const { id } = useParams<{ id: string }>()
  const isEdit = Boolean(id)
  const navigate = useNavigate()

  const [form, setForm] = useState<TenantWritePayload>(EMPTY_FORM)
  const [tenantName, setTenantName] = useState<string | null>(null)
  const [loading, setLoading] = useState(isEdit)
  const [loadError, setLoadError] = useState<string | null>(null)
  const [submitting, setSubmitting] = useState(false)
  const [formError, setFormError] = useState<string | null>(null)
  const [fieldErrors, setFieldErrors] = useState<FieldErrors | null>(null)

  const load = useCallback(async () => {
    if (!id) return
    setLoading(true)
    setLoadError(null)
    try {
      const tenant = await getTenant(id)
      setForm({
        name: tenant.name,
        email: tenant.email,
        phone: tenant.phone,
        address: tenant.address,
        employee_count: tenant.employee_count,
        is_active: tenant.is_active,
      })
      setTenantName(tenant.name)
    } catch (err) {
      setLoadError(normalizeApiError(err).message)
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

  function updateField<K extends keyof TenantWritePayload>(key: K, value: TenantWritePayload[K]) {
    setForm((prev) => ({ ...prev, [key]: value }))
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setFormError(null)
    setFieldErrors(null)
    setSubmitting(true)
    try {
      const result = isEdit && id ? await updateTenant(id, form) : await createTenant(form)
      navigate(`/tenants/${result.id}`, { replace: true })
    } catch (err) {
      const apiError = normalizeApiError(err)
      setFormError(apiError.message)
      setFieldErrors(apiError.fieldErrors)
    } finally {
      setSubmitting(false)
    }
  }

  const cancelHref = isEdit && id ? `/tenants/${id}` : '/tenants'
  const action = isEdit ? 'Modifier le tenant' : 'Nouveau tenant'

  return (
    <div>
      <div style={{ fontSize: '14px', color: 'var(--text-secondary)', marginBottom: '18px' }}>
        <Link to="/tenants" style={{ color: 'var(--crimson)', fontWeight: 500, textDecoration: 'none' }}>← Tenants</Link>
        {tenantName && <span> / {tenantName}</span>}
        <span> / {action}</span>
      </div>

      <div style={{ marginBottom: '24px' }}>
        <h1 style={{ fontSize: '22px', fontWeight: 700, color: 'var(--text-primary)' }}>{action}</h1>
      </div>

      {loading && <LoadingState />}

      {!loading && loadError && (
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
            maxWidth: '640px',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <AlertCircle size={18} />
            <span>{loadError}</span>
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
      )}

      {!loading && !loadError && (
        <div
          style={{
            background: '#FFFFFF',
            border: '1px solid var(--border)',
            borderRadius: 'var(--radius)',
            boxShadow: 'var(--shadow)',
            padding: '24px',
            maxWidth: '640px',
          }}
        >
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
                marginBottom: '18px',
              }}
            >
              <AlertCircle size={16} style={{ flexShrink: 0, marginTop: '1px' }} />
              <span>{formError}</span>
            </div>
          )}

          <form onSubmit={handleSubmit} style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
            <div>
              <label style={labelStyle}>Nom de l'entreprise</label>
              <input
                type="text"
                required
                value={form.name ?? ''}
                onChange={(e) => updateField('name', e.target.value)}
                style={inputStyle}
              />
              <FieldError messages={fieldErrors?.name} />
            </div>

            <div>
              <label style={labelStyle}>Email de contact</label>
              <input
                type="email"
                required
                value={form.email ?? ''}
                onChange={(e) => updateField('email', e.target.value)}
                style={inputStyle}
              />
              <FieldError messages={fieldErrors?.email} />
            </div>

            <div style={{ display: 'flex', gap: '12px' }}>
              <div style={{ flex: 1 }}>
                <label style={labelStyle}>Téléphone</label>
                <input
                  type="tel"
                  value={form.phone ?? ''}
                  onChange={(e) => updateField('phone', e.target.value)}
                  style={inputStyle}
                />
                <FieldError messages={fieldErrors?.phone} />
              </div>
              <div style={{ width: '160px' }}>
                <label style={labelStyle}>Employés</label>
                <input
                  type="number"
                  required
                  min={1}
                  value={form.employee_count ?? 1}
                  onChange={(e) => updateField('employee_count', Number(e.target.value))}
                  style={inputStyle}
                />
                <FieldError messages={fieldErrors?.employee_count} />
              </div>
            </div>

            <div>
              <label style={labelStyle}>Adresse</label>
              <input
                type="text"
                value={form.address ?? ''}
                onChange={(e) => updateField('address', e.target.value)}
                style={inputStyle}
              />
              <FieldError messages={fieldErrors?.address} />
            </div>

            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <input
                id="is_active"
                type="checkbox"
                checked={form.is_active ?? true}
                onChange={(e) => updateField('is_active', e.target.checked)}
              />
              <label htmlFor="is_active" style={{ fontSize: '13px', color: 'var(--text-primary)' }}>
                Tenant actif
              </label>
              <FieldError messages={fieldErrors?.is_active} />
            </div>

            <div style={{ display: 'flex', gap: '10px', marginTop: '10px' }}>
              <button
                type="submit"
                disabled={submitting}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  gap: '8px',
                  padding: '10px 20px',
                  background: submitting ? '#C46A72' : 'var(--crimson)',
                  color: '#FFF',
                  border: 'none',
                  borderRadius: '6px',
                  fontWeight: 600,
                  cursor: submitting ? 'default' : 'pointer',
                }}
              >
                {submitting ? <Loader2 size={16} className="tf-spin" /> : <Save size={16} />}
                {action}
              </button>
              <Link
                to={cancelHref}
                style={{
                  padding: '10px 20px',
                  background: '#FFF',
                  border: '1px solid var(--border)',
                  borderRadius: '6px',
                  color: 'var(--text-primary)',
                  textDecoration: 'none',
                  fontWeight: 600,
                  fontSize: '14px',
                  display: 'flex',
                  alignItems: 'center',
                }}
              >
                Annuler
              </Link>
            </div>
          </form>
        </div>
      )}
    </div>
  )
}
