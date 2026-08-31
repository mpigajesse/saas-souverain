import type React from 'react'
import { useState } from 'react'
import { NavLink, useNavigate } from 'react-router-dom'
import type { ReactNode } from 'react'
import { useAuth } from '../context/AuthContext'

interface LayoutProps {
  children: ReactNode
}

const IconDashboard = () => (
  <svg viewBox="0 0 16 16" width={16} height={16} fill="currentColor" aria-hidden="true">
    <rect x="1" y="1" width="6" height="6" rx="1" />
    <rect x="9" y="1" width="6" height="6" rx="1" />
    <rect x="1" y="9" width="6" height="6" rx="1" />
    <rect x="9" y="9" width="6" height="6" rx="1" />
  </svg>
)

const IconTenants = () => (
  <svg viewBox="0 0 16 16" width={16} height={16} fill="none" stroke="currentColor" strokeWidth="1.2" aria-hidden="true">
    <path d="M2 14V6l6-4 6 4v8H2z" />
    <rect x="5" y="9" width="2" height="3" />
    <rect x="9" y="9" width="2" height="3" />
  </svg>
)

const IconLicenses = () => (
  <svg viewBox="0 0 16 16" width={16} height={16} fill="none" stroke="currentColor" strokeWidth="1.2" aria-hidden="true">
    <path d="M12.5 7.5A3.5 3.5 0 1 0 9 11h1v2h2v-2h.5a.5.5 0 0 0 .5-.5v-3a.5.5 0 0 0-.5-.5z" />
    <circle cx="5.5" cy="6.5" r="2.5" />
  </svg>
)

const NAV_LINKS = [
  { to: '/', label: 'Console Éditeur', exact: true, Icon: IconDashboard },
  { to: '/tenants', label: 'Tenants (Clients PME)', exact: false, Icon: IconTenants },
  { to: '/licenses', label: 'Licences Ed25519', exact: false, Icon: IconLicenses },
]

export default function Layout({ children }: LayoutProps) {
  const [hoveredLink, setHoveredLink] = useState<string | null>(null)
  const { user, logout } = useAuth()
  const navigate = useNavigate()

  const handleLogout = () => {
    logout()
    navigate('/login')
  }

  const sidebarStyle: React.CSSProperties = {
    width: '240px',
    height: '100vh',
    position: 'fixed',
    top: 0,
    left: 0,
    backgroundColor: 'var(--sidebar-bg)',
    display: 'flex',
    flexDirection: 'column',
    zIndex: 100,
    overflowY: 'auto',
  }

  const headerStyle: React.CSSProperties = {
    padding: '20px 16px',
    borderBottom: '1px solid rgba(196,151,42,0.25)',
    display: 'flex',
    flexDirection: 'column',
    gap: '4px',
  }

  const brandNameStyle: React.CSSProperties = {
    color: '#FFFFFF',
    fontWeight: 700,
    fontSize: '13px',
    letterSpacing: '0.02em',
  }

  const brandSubStyle: React.CSSProperties = {
    color: 'var(--gold)',
    fontSize: '11px',
  }

  const navStyle: React.CSSProperties = {
    padding: '16px 12px 8px 12px',
  }

  const sectionLabelStyle: React.CSSProperties = {
    color: 'rgba(232,184,75,0.6)',
    fontSize: '10px',
    letterSpacing: '1.5px',
    textTransform: 'uppercase',
    padding: '4px 8px',
    display: 'block',
    marginBottom: '6px',
  }

  const footerStyle: React.CSSProperties = {
    marginTop: 'auto',
    padding: '16px',
    borderTop: '1px solid rgba(255,255,255,0.1)',
    display: 'flex',
    flexDirection: 'column',
    gap: '10px',
    background: 'rgba(0,0,0,0.2)',
  }

  const contentStyle: React.CSSProperties = {
    marginLeft: '240px',
    minHeight: '100vh',
    background: 'var(--content-bg)',
    padding: '32px',
  }

  const baseLinkStyle: React.CSSProperties = {
    display: 'flex',
    alignItems: 'center',
    gap: '10px',
    padding: '10px 12px',
    borderRadius: '6px',
    color: 'rgba(255,255,255,0.7)',
    marginBottom: '2px',
    fontSize: '14px',
    fontWeight: 500,
    transition: 'background 0.15s, color 0.15s',
    borderLeft: '3px solid transparent',
  }

  const activeLinkStyle: React.CSSProperties = {
    ...baseLinkStyle,
    background: 'var(--sidebar-active)',
    color: '#FFFFFF',
    borderLeft: '3px solid var(--gold)',
  }

  const hoverLinkStyle: React.CSSProperties = {
    ...baseLinkStyle,
    background: 'var(--sidebar-hover)',
    color: '#FFFFFF',
  }

  return (
    <>
      <aside style={sidebarStyle}>
        {/* En-tête avec Grand Logo Éditeur */}
        <div style={headerStyle}>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', marginBottom: '10px' }}>
            <img src="/logoentreprise.png" alt="EL BARAA CONSULT logo" style={{ height: '68px', maxWidth: '100%', objectFit: 'contain' }} />
          </div>
          <span style={brandNameStyle}>{user?.company || 'EL BARAA CONSULT'}</span>
          <span style={brandSubStyle}>Console d'Administration Éditeur</span>
        </div>

        {/* Menu Principal */}
        <nav style={navStyle}>
          <span style={sectionLabelStyle}>GESTION SAAS</span>
          {NAV_LINKS.map(({ to, label, exact, Icon }) => (
            <NavLink
              key={to}
              to={to}
              end={exact}
              style={({ isActive }) => {
                if (isActive) return activeLinkStyle
                if (hoveredLink === to) return hoverLinkStyle
                return baseLinkStyle
              }}
              onMouseEnter={() => setHoveredLink(to)}
              onMouseLeave={() => setHoveredLink(null)}
            >
              <Icon />
              {label}
            </NavLink>
          ))}
        </nav>

        {/* SECTION 1 ADDITIONNELLE : STATUT INFRASTRUCTURE & RELAIS AVEUGLE */}
        <div style={{ padding: '8px 12px 12px 12px', margin: '0 8px', background: 'rgba(255,255,255,0.03)', borderRadius: '6px', border: '1px solid rgba(196,151,42,0.15)' }}>
          <span style={sectionLabelStyle}>INFRASTRUCTURE & INFOS</span>
          
          <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', fontSize: '11px', color: 'rgba(255,255,255,0.8)' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <span style={{ color: 'rgba(255,255,255,0.6)' }}>Relais Aveugle :</span>
              <span style={{ color: '#4CAF50', fontWeight: 700 }}>🟢 Actif</span>
            </div>

            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <span style={{ color: 'rgba(255,255,255,0.6)' }}>VPN WireGuard :</span>
              <span style={{ color: 'var(--gold)', fontWeight: 600, fontFamily: 'monospace' }}>10.10.0.0/24</span>
            </div>

            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <span style={{ color: 'rgba(255,255,255,0.6)' }}>Crypto Clés :</span>
              <span style={{ color: '#FFF', fontWeight: 600 }}>AK / DEK (100%)</span>
            </div>
          </div>
        </div>

        {/* SECTION 2 ADDITIONNELLE : RACCOURCIS OUTILS & ACCÈS DIRECTS */}
        <div style={{ padding: '12px', margin: '8px 8px 0 8px' }}>
          <span style={sectionLabelStyle}>ACCÈS RAPIDES</span>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '6px', fontSize: '12px' }}>
            <a
              href="http://localhost:3000"
              target="_blank"
              rel="noopener noreferrer"
              style={{
                color: 'rgba(255,255,255,0.7)',
                padding: '6px 8px',
                borderRadius: '4px',
                background: 'rgba(255,255,255,0.05)',
                display: 'flex',
                alignItems: 'center',
                gap: '8px',
                textDecoration: 'none',
              }}
            >
              <span>📊</span> Live Grafana (Port 3000)
            </a>

            <a
              href="http://localhost:8000/admin/"
              target="_blank"
              rel="noopener noreferrer"
              style={{
                color: 'rgba(255,255,255,0.7)',
                padding: '6px 8px',
                borderRadius: '4px',
                background: 'rgba(255,255,255,0.05)',
                display: 'flex',
                alignItems: 'center',
                gap: '8px',
                textDecoration: 'none',
              }}
            >
              <span>⚙️</span> Admin Django Backend
            </a>
          </div>
        </div>

        {/* Pied de Sidebar & Profil Éditeur avec Bouton Déconnexion */}
        <div style={footerStyle}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
            <div
              style={{
                width: '32px',
                height: '32px',
                borderRadius: '50%',
                background: 'var(--gold)',
                color: '#2D1419',
                fontWeight: 700,
                fontSize: '12px',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
            >
              {user?.name.substring(0, 2).toUpperCase() || 'AD'}
            </div>
            <div style={{ overflow: 'hidden' }}>
              <div style={{ color: '#FFF', fontSize: '12px', fontWeight: 600, textOverflow: 'ellipsis', overflow: 'hidden', whiteSpace: 'nowrap' }}>
                {user?.name || 'Administrateur'}
              </div>
              <div style={{ color: 'rgba(255,255,255,0.5)', fontSize: '10px' }}>{user?.role || 'SaaS Admin'}</div>
            </div>
          </div>

          <button
            onClick={handleLogout}
            style={{
              width: '100%',
              padding: '6px 10px',
              background: 'rgba(255,255,255,0.08)',
              color: 'rgba(255,255,255,0.7)',
              border: '1px solid rgba(255,255,255,0.15)',
              borderRadius: '4px',
              fontSize: '11px',
              cursor: 'pointer',
              fontWeight: 500,
            }}
          >
            🚪 Déconnexion
          </button>
        </div>
      </aside>

      <main style={contentStyle}>{children}</main>
    </>
  )
}
