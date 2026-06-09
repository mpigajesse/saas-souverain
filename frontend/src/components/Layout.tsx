import type React from 'react'
import { useState } from 'react'
import { NavLink } from 'react-router-dom'
import type { ReactNode } from 'react'

interface LayoutProps {
  children: ReactNode
}

const IconDashboard = () => (
  <svg
    viewBox="0 0 16 16"
    width={16}
    height={16}
    fill="currentColor"
    aria-hidden="true"
  >
    <rect x="1" y="1" width="6" height="6" rx="1" />
    <rect x="9" y="1" width="6" height="6" rx="1" />
    <rect x="1" y="9" width="6" height="6" rx="1" />
    <rect x="9" y="9" width="6" height="6" rx="1" />
  </svg>
)

const IconTenants = () => (
  <svg
    viewBox="0 0 16 16"
    width={16}
    height={16}
    fill="none"
    stroke="currentColor"
    strokeWidth="1.2"
    aria-hidden="true"
  >
    <path d="M2 14V6l6-4 6 4v8H2z" />
    <rect x="5" y="9" width="2" height="3" />
    <rect x="9" y="9" width="2" height="3" />
  </svg>
)

const NAV_LINKS = [
  { to: '/', label: 'Dashboard', exact: true, Icon: IconDashboard },
  { to: '/tenants', label: 'Tenants', exact: false, Icon: IconTenants },
]

export default function Layout({ children }: LayoutProps) {
  const [hoveredLink, setHoveredLink] = useState<string | null>(null)

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
  }

  const headerStyle: React.CSSProperties = {
    padding: '20px',
    borderBottom: '1px solid rgba(196,151,42,0.3)',
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
    padding: '16px 12px',
    flex: 1,
  }

  const sectionLabelStyle: React.CSSProperties = {
    color: 'rgba(232,184,75,0.5)',
    fontSize: '10px',
    letterSpacing: '1.5px',
    textTransform: 'uppercase',
    padding: '8px',
    display: 'block',
    marginBottom: '4px',
  }

  const footerStyle: React.CSSProperties = {
    position: 'absolute',
    bottom: '20px',
    left: '20px',
    color: 'rgba(255,255,255,0.3)',
    fontSize: '11px',
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
        <div style={headerStyle}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '6px' }}>
            <img src="/logoentreprise.png" alt="EL BARAA CONSULT logo" style={{ height: 40 }} />
          </div>
          <span style={brandNameStyle}>EL BARAA CONSULT</span>
          <span style={brandSubStyle}>Plateforme SaaS</span>
        </div>

        <nav style={navStyle}>
          <span style={sectionLabelStyle}>NAVIGATION</span>
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

        <div style={footerStyle}>v1.0.0</div>
      </aside>

      <main style={contentStyle}>{children}</main>
    </>
  )
}
