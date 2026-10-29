import type { ReactNode } from 'react'
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { Loader2, ShieldAlert } from 'lucide-react'
import { AuthProvider } from './context/AuthContext'
import { useAuth } from './context/useAuth'
import Layout from './components/Layout'
import type { UserRole } from './types/index'

import DashboardPage from './pages/DashboardPage'
import TenantsPage from './pages/TenantsPage'
import TenantDetail from './pages/TenantDetail'
import TenantFormPage from './pages/TenantFormPage'
import LicensesPage from './pages/LicensesPage'
import DevicesPage from './pages/DevicesPage'
import ClustersPage from './pages/ClustersPage'
import RelayPage from './pages/RelayPage'
import DeploiementPage from './pages/DeploiementPage'
import ArchitecturePage from './pages/ArchitecturePage'
import ApplicationsPage from './pages/ApplicationsPage'
import MonEspacePage from './pages/MonEspacePage'
import BienvenuePage from './pages/BienvenuePage'
import LoginPage from './pages/LoginPage'
import RegisterPage from './pages/RegisterPage'

/** Écran d'attente pendant la vérification de la session (GET /api/auth/me/). */
function LoadingScreen() {
  return (
    <div
      style={{
        minHeight: '100vh',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        gap: '12px',
        background: 'var(--content-bg)',
        color: 'var(--text-secondary)',
      }}
    >
      <Loader2 size={28} className="app-spin" />
      <span style={{ fontSize: '13px' }}>Vérification de la session...</span>
      <style>{`
        .app-spin { animation: app-spin 0.8s linear infinite; }
        @keyframes app-spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
      `}</style>
    </div>
  )
}

function NotFoundPage() {
  return (
    <div
      style={{
        minHeight: '100vh',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        gap: '12px',
        background: 'var(--content-bg)',
        color: 'var(--text-secondary)',
        textAlign: 'center',
        padding: '20px',
      }}
    >
      <ShieldAlert size={36} color="var(--gold)" />
      <h1 style={{ fontSize: '20px', fontWeight: 700, color: 'var(--text-primary)' }}>Page introuvable</h1>
      <p style={{ fontSize: '13px', maxWidth: '360px' }}>
        Cette page n'existe pas ou vous n'y avez pas accès.
      </p>
    </div>
  )
}

/** Redirection par défaut selon le rôle : editor -> "/", tenant -> "/mon-espace". */
function homeRouteFor(role: UserRole): string {
  return role === 'editor' ? '/' : '/mon-espace'
}

/** N'affiche les enfants qu'aux utilisateurs anonymes (login, register). */
function PublicOnly({ children }: { children: ReactNode }) {
  const { user, loading } = useAuth()
  if (loading) return <LoadingScreen />
  if (user) return <Navigate to={homeRouteFor(user.role)} replace />
  return <>{children}</>
}

/** Exige une session active du rôle donné ; redirige sinon (login, ou l'espace du bon rôle). */
function RequireRole({ role, children }: { role: UserRole; children: ReactNode }) {
  const { user, loading } = useAuth()
  if (loading) return <LoadingScreen />
  if (!user) return <Navigate to="/login" replace />
  if (user.role !== role) return <Navigate to={homeRouteFor(user.role)} replace />
  return <Layout>{children}</Layout>
}

function EditorRoute({ children }: { children: ReactNode }) {
  return <RequireRole role="editor">{children}</RequireRole>
}

function TenantRoute({ children }: { children: ReactNode }) {
  return <RequireRole role="tenant">{children}</RequireRole>
}

export default function App() {
  return (
    <AuthProvider>
      <BrowserRouter>
        <Routes>
          <Route path="/login" element={<PublicOnly><LoginPage /></PublicOnly>} />
          <Route path="/register" element={<PublicOnly><RegisterPage /></PublicOnly>} />

          <Route path="/" element={<EditorRoute><DashboardPage /></EditorRoute>} />
          <Route path="/tenants" element={<EditorRoute><TenantsPage /></EditorRoute>} />
          <Route path="/tenants/nouveau" element={<EditorRoute><TenantFormPage /></EditorRoute>} />
          <Route path="/tenants/:id" element={<EditorRoute><TenantDetail /></EditorRoute>} />
          <Route path="/tenants/:id/modifier" element={<EditorRoute><TenantFormPage /></EditorRoute>} />
          <Route path="/licenses" element={<EditorRoute><LicensesPage /></EditorRoute>} />
          <Route path="/applications" element={<EditorRoute><ApplicationsPage /></EditorRoute>} />
          <Route path="/devices" element={<EditorRoute><DevicesPage /></EditorRoute>} />
          <Route path="/clusters" element={<EditorRoute><ClustersPage /></EditorRoute>} />
          <Route path="/relay" element={<EditorRoute><RelayPage /></EditorRoute>} />
          <Route path="/deploiement" element={<EditorRoute><DeploiementPage /></EditorRoute>} />
          <Route path="/architecture" element={<EditorRoute><ArchitecturePage /></EditorRoute>} />

          <Route path="/mon-espace" element={<TenantRoute><MonEspacePage /></TenantRoute>} />
          <Route path="/bienvenue" element={<TenantRoute><BienvenuePage /></TenantRoute>} />

          <Route path="*" element={<NotFoundPage />} />
        </Routes>
      </BrowserRouter>
    </AuthProvider>
  )
}
