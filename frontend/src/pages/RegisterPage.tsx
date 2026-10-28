import type React from 'react'
import { useState } from 'react'
import { useNavigate, Link } from 'react-router-dom'
import { AlertCircle, Loader2, UserPlus } from 'lucide-react'
import { useAuth } from '../context/useAuth'
import type { ApiError, FieldErrors, RegisterPayload } from '../types/index'

const EMPTY_FORM: RegisterPayload = {
  name: '',
  email: '',
  phone: '',
  address: '',
  employee_count: 1,
  password: '',
  password_confirm: '',
}

const inputStyle: React.CSSProperties = {
  width: '100%',
  padding: '10px 12px',
  borderRadius: '8px',
  border: '1px solid #DDD',
  fontSize: '14px',
}

const labelStyle: React.CSSProperties = {
  display: 'block',
  fontSize: '12px',
  fontWeight: 600,
  color: '#444',
  marginBottom: '4px',
}

function FieldError({ messages }: { messages?: string[] }) {
  if (!messages || messages.length === 0) return null
  return (
    <div style={{ color: 'var(--danger)', fontSize: '12px', marginTop: '4px' }}>{messages[0]}</div>
  )
}

export default function RegisterPage() {
  const [form, setForm] = useState<RegisterPayload>(EMPTY_FORM)
  const [submitting, setSubmitting] = useState(false)
  const [formError, setFormError] = useState<string | null>(null)
  const [fieldErrors, setFieldErrors] = useState<FieldErrors | null>(null)
  const { register } = useAuth()
  const navigate = useNavigate()

  function updateField<K extends keyof RegisterPayload>(key: K, value: RegisterPayload[K]) {
    setForm((prev) => ({ ...prev, [key]: value }))
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setFormError(null)
    setFieldErrors(null)
    setSubmitting(true)
    try {
      const user = await register(form)
      navigate(user.role === 'editor' ? '/' : '/mon-espace', { replace: true })
    } catch (err) {
      const apiError = err as ApiError
      setFormError(apiError.message)
      setFieldErrors(apiError.fieldErrors)
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <div
      style={{
        minHeight: '100vh',
        background: 'linear-gradient(135deg, #2D1419 0%, #17090C 100%)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '20px',
      }}
    >
      <div
        style={{
          background: '#FFFFFF',
          borderRadius: '16px',
          padding: '40px',
          maxWidth: '460px',
          width: '100%',
          boxShadow: '0 25px 50px -12px rgba(0, 0, 0, 0.5)',
        }}
      >
        <div style={{ textAlign: 'center', marginBottom: '28px' }}>
          <img src="/logoentreprise.png" alt="EL BARAA CONSULT" style={{ height: '48px', marginBottom: '12px' }} />
          <h1 style={{ fontSize: '22px', fontWeight: 700, color: '#17090C' }}>Inscription PME</h1>
          <p style={{ fontSize: '13px', color: '#666666', marginTop: '4px' }}>
            Créez le compte de votre entreprise sur SaaS Souverain
          </p>
        </div>

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
              placeholder="ex : EL BARAA CONSULT"
              value={form.name}
              onChange={(e) => updateField('name', e.target.value)}
              style={inputStyle}
            />
            <FieldError messages={fieldErrors?.name} />
          </div>

          <div>
            <label style={labelStyle}>Email</label>
            <input
              type="email"
              required
              placeholder="contact@votre-societe.com"
              value={form.email}
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
                required
                placeholder="+212 5XX XXX XXX"
                value={form.phone}
                onChange={(e) => updateField('phone', e.target.value)}
                style={inputStyle}
              />
              <FieldError messages={fieldErrors?.phone} />
            </div>
            <div style={{ width: '140px' }}>
              <label style={labelStyle}>Employés</label>
              <input
                type="number"
                required
                min={1}
                value={form.employee_count}
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
              required
              placeholder="Adresse du siège"
              value={form.address}
              onChange={(e) => updateField('address', e.target.value)}
              style={inputStyle}
            />
            <FieldError messages={fieldErrors?.address} />
          </div>

          <div>
            <label style={labelStyle}>Mot de passe</label>
            <input
              type="password"
              required
              placeholder="Minimum 8 caractères"
              autoComplete="new-password"
              value={form.password}
              onChange={(e) => updateField('password', e.target.value)}
              style={inputStyle}
            />
            <FieldError messages={fieldErrors?.password} />
          </div>

          <div>
            <label style={labelStyle}>Confirmation du mot de passe</label>
            <input
              type="password"
              required
              placeholder="Retapez le mot de passe"
              autoComplete="new-password"
              value={form.password_confirm}
              onChange={(e) => updateField('password_confirm', e.target.value)}
              style={inputStyle}
            />
            <FieldError messages={fieldErrors?.password_confirm} />
          </div>

          <button
            type="submit"
            disabled={submitting}
            style={{
              width: '100%',
              padding: '14px',
              background: submitting ? '#A64B5B' : '#800020',
              color: '#FFFFFF',
              border: 'none',
              borderRadius: '8px',
              fontWeight: 700,
              fontSize: '15px',
              cursor: submitting ? 'default' : 'pointer',
              marginTop: '10px',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              gap: '8px',
            }}
          >
            {submitting ? <Loader2 size={18} className="spin" /> : <UserPlus size={18} />}
            {submitting ? 'Création en cours...' : 'Créer mon compte'}
          </button>
        </form>

        <div style={{ marginTop: '24px', textAlign: 'center', fontSize: '13px', color: '#666' }}>
          Vous avez déjà un compte ?{' '}
          <Link to="/login" style={{ color: '#800020', fontWeight: 600, textDecoration: 'none' }}>
            Se connecter
          </Link>
        </div>
      </div>

      <style>{`
        .spin { animation: spin 0.8s linear infinite; }
        @keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
      `}</style>
    </div>
  )
}
