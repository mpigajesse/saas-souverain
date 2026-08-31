import type React from 'react'
import { useState } from 'react'
import { useNavigate, Link } from 'react-router-dom'
import { useAuth } from '../context/AuthContext'

export default function RegisterPage() {
  const [company, setCompany] = useState('')
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const { register } = useAuth()
  const navigate = useNavigate()

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (company && email) {
      register(company, email, password)
      navigate('/')
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
          <h1 style={{ fontSize: '22px', fontWeight: 700, color: '#17090C' }}>Inscription Éditeur SaaS</h1>
          <p style={{ fontSize: '13px', color: '#666666', marginTop: '4px' }}>
            Déployez la plateforme AMANE V2 pour gérer vos clients PME
          </p>
        </div>

        <form onSubmit={handleSubmit} style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          <div>
            <label style={{ display: 'block', fontSize: '12px', fontWeight: 600, color: '#444', marginBottom: '4px' }}>
              Nom de votre Société Éditeur SaaS
            </label>
            <input
              type="text"
              required
              placeholder="ex: EL BARAA CONSULT / SYSTEMES S.A."
              value={company}
              onChange={(e) => setCompany(e.target.value)}
              style={{
                width: '100%',
                padding: '10px 12px',
                borderRadius: '8px',
                border: '1px solid #DDD',
                fontSize: '14px',
              }}
            />
          </div>

          <div>
            <label style={{ display: 'block', fontSize: '12px', fontWeight: 600, color: '#444', marginBottom: '4px' }}>
              Email administrateur
            </label>
            <input
              type="email"
              required
              placeholder="contact@votre-societe.com"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              style={{
                width: '100%',
                padding: '10px 12px',
                borderRadius: '8px',
                border: '1px solid #DDD',
                fontSize: '14px',
              }}
            />
          </div>

          <div>
            <label style={{ display: 'block', fontSize: '12px', fontWeight: 600, color: '#444', marginBottom: '4px' }}>
              Mot de passe
            </label>
            <input
              type="password"
              required
              placeholder="Minimum 8 caractères"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              style={{
                width: '100%',
                padding: '10px 12px',
                borderRadius: '8px',
                border: '1px solid #DDD',
                fontSize: '14px',
              }}
            />
          </div>

          <button
            type="submit"
            style={{
              width: '100%',
              padding: '14px',
              background: '#800020',
              color: '#FFFFFF',
              border: 'none',
              borderRadius: '8px',
              fontWeight: 700,
              fontSize: '15px',
              cursor: 'pointer',
              marginTop: '10px',
            }}
          >
            Créer mon Compte Éditeur
          </button>
        </form>

        <div style={{ marginTop: '24px', textAlign: 'center', fontSize: '13px', color: '#666' }}>
          Vous avez déjà un compte ?{' '}
          <Link to="/login" style={{ color: '#800020', fontWeight: 600, textDecoration: 'none' }}>
            Se connecter
          </Link>
        </div>
      </div>
    </div>
  )
}
