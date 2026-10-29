import type React from 'react'
import { useState } from 'react'
import { NavLink, useNavigate } from 'react-router-dom'
import type { ComponentType, ReactNode } from 'react'
import {
  AppWindow,
  BarChart3,
  Boxes,
  Building2,
  Container,
  Download,
  HardDrive,
  KeyRound,
  LayoutDashboard,
  LogOut,
  Network,
  Radio,
  Settings,
} from 'lucide-react'
import { useAuth } from '../context/useAuth'
import type { UserRole } from '../types/index'

interface LayoutProps {
  children: ReactNode
}

interface NavItem {
  to: string
  label: string
  exact: boolean
  Icon: ComponentType<{ size?: number }>
}

const EDITOR_LINKS: NavItem[] = [
  { to: '/', label: 'Console', exact: true, Icon: LayoutDashboard },
  { to: '/tenants', label: 'Tenants', exact: false, Icon: Building2 },
  { to: '/licenses', label: 'Licences', exact: false, Icon: KeyRound },
  { to: '/applications', label: 'Applications', exact: false, Icon: AppWindow },
  { to: '/devices', label: 'Appareils', exact: false, Icon: HardDrive },
  { to: '/clusters', label: 'Clusters', exact: false, Icon: Network },
  { to: '/relay', label: 'Relais', exact: false, Icon: Radio },
  { to: '/deploiement', label: 'Déploiement', exact: false, Icon: Container },
  { to: '/architecture', label: 'Architecture', exact: false, Icon: Boxes },
]

const TENANT_LINKS: NavItem[] = [
  { to: '/mon-espace', label: 'Mon espace', exact: true, Icon: LayoutDashboard },
  { to: '/bienvenue', label: 'Installation', exact: false, Icon: Download },
]

const ROLE_SECTION_LABEL: Record<UserRole, string> = {
  editor: 'GESTION SAAS',
  tenant: 'MON ENTREPRISE',
}

const ROLE_SUBTITLE: Record<UserRole, string> = {
  editor: "Console d'administration éditeur",
  tenant: 'Espace client PME',
}

function navLinksFor(role: UserRole | null): NavItem[] {
  if (role === 'tenant') return TENANT_LINKS
  return EDITOR_LINKS
}

export default function Layout({ children }: LayoutProps) {
  const [hoveredLink, setHoveredLink] = useState<string | null>(null)
  const { user, role, logout } = useAuth()
  const navigate = useNavigate()

  const handleLogout = () => {
    logout()
      .catch(() => {
        // logout() désauthentifie déjà localement dans son bloc finally ;
        // une erreur réseau ici ne doit pas empêcher le retour à l'écran de connexion.
      })
      .finally(() => navigate('/login', { replace: true }))
  }

  const displayName = user?.username ?? 'Utilisateur'
  const displayInitials = displayName.slice(0, 2).toUpperCase()
  const displaySubtitle = role ? ROLE_SUBTITLE[role] : ''
  const brandName = role === 'tenant' ? user?.tenant?.name ?? displayName : 'SaaS Souverain'

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

  const quickLinkStyle: React.CSSProperties = {
    color: 'rgba(255,255,255,0.7)',
    padding: '6px 8px',
    borderRadius: '4px',
    background: 'rgba(255,255,255,0.05)',
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    textDecoration: 'none',
  }

  return (
    <>
      <aside style={sidebarStyle}>
        {/* En-tête */}
        <div style={headerStyle}>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', marginBottom: '10px' }}>
            <img src="/logoentreprise.png" alt="SaaS Souverain" style={{ height: '68px', maxWidth: '100%', objectFit: 'contain' }} />
          </div>
          <span style={brandNameStyle}>{brandName}</span>
          <span style={brandSubStyle}>{displaySubtitle}</span>
        </div>

        {/* Menu principal, dépendant du rôle */}
        <nav style={navStyle}>
          <span style={sectionLabelStyle}>{role ? ROLE_SECTION_LABEL[role] : ''}</span>
          {navLinksFor(role).map(({ to, label, exact, Icon }) => (
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
              <Icon size={16} />
              {label}
            </NavLink>
          ))}
        </nav>

        {/* Accès rapides — uniquement pour l'éditeur */}
        {role === 'editor' && (
          <div style={{ padding: '12px', margin: '8px 8px 0 8px' }}>
            <span style={sectionLabelStyle}>ACCÈS RAPIDES</span>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '6px', fontSize: '12px' }}>
              <a href="http://localhost:3000" target="_blank" rel="noopener noreferrer" style={quickLinkStyle}>
                <BarChart3 size={14} /> Grafana
              </a>
              <a href="http://localhost:8000/admin/" target="_blank" rel="noopener noreferrer" style={quickLinkStyle}>
                <Settings size={14} /> Admin Django
              </a>
            </div>
          </div>
        )}

        {/* Pied de sidebar : profil connecté + déconnexion */}
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
                flexShrink: 0,
              }}
            >
              {displayInitials}
            </div>
            <div style={{ overflow: 'hidden' }}>
              <div style={{ color: '#FFF', fontSize: '12px', fontWeight: 600, textOverflow: 'ellipsis', overflow: 'hidden', whiteSpace: 'nowrap' }}>
                {displayName}
              </div>
              <div style={{ color: 'rgba(255,255,255,0.5)', fontSize: '10px' }}>
                {role === 'editor' ? 'Administrateur SaaS' : 'Compte PME'}
              </div>
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
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              gap: '6px',
            }}
          >
            <LogOut size={13} /> Déconnexion
          </button>
        </div>
      </aside>

      <main style={contentStyle}>{children}</main>
    </>
  )
}
