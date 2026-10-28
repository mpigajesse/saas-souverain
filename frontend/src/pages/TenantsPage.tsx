// Liste des tenants (entreprises clientes) — remplace templates/tenants/list.html.
// CRUD réel via l'API : plus aucune création ou suppression en mémoire locale.

import { useCallback, useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { AlertCircle, Building2, Loader2, Plus, RotateCw, Search, Trash2 } from 'lucide-react'
import { deleteTenant, getTenants, normalizeApiError } from '../api/client'
import StatusBadge from '../components/StatusBadge'
import type { Tenant } from '../types/index'

function LoadingState() {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: '10px', color: 'var(--text-secondary)', padding: '48px 0' }}>
      <Loader2 size={18} className="tp-spin" />
      <span>Chargement des tenants...</span>
      <style>{`.tp-spin { animation: tp-spin 0.8s linear infinite; } @keyframes tp-spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }`}</style>
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

export default function TenantsPage() {
  const [tenants, setTenants] = useState<Tenant[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [searchTerm, setSearchTerm] = useState('')
  const [deletingId, setDeletingId] = useState<string | null>(null)
  const [deleteError, setDeleteError] = useState<string | null>(null)

  const load = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const data = await getTenants()
      setTenants(data)
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

  async function handleDelete(tenant: Tenant) {
    if (!window.confirm(`Supprimer définitivement le tenant « ${tenant.name} » ? Cette action est irréversible.`)) {
      return
    }
    setDeleteError(null)
    setDeletingId(tenant.id)
    try {
      await deleteTenant(tenant.id)
      setTenants((prev) => prev.filter((t) => t.id !== tenant.id))
    } catch (err) {
      setDeleteError(normalizeApiError(err).message)
    } finally {
      setDeletingId(null)
    }
  }

  const filteredTenants = tenants.filter(
    (t) =>
      t.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
      t.email.toLowerCase().includes(searchTerm.toLowerCase())
  )

  return (
    <div>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '24px' }}>
        <div>
          <h1 style={{ fontSize: '22px', fontWeight: 700, color: 'var(--text-primary)' }}>Tenants</h1>
          <p style={{ color: 'var(--text-secondary)', marginTop: '4px' }}>Gestion des entreprises clientes</p>
        </div>
        <Link
          to="/tenants/nouveau"
          style={{
            background: 'var(--crimson)',
            color: '#FFFFFF',
            border: 'none',
            padding: '10px 18px',
            borderRadius: 'var(--radius)',
            fontSize: '13px',
            fontWeight: 600,
            cursor: 'pointer',
            boxShadow: 'var(--shadow)',
            textDecoration: 'none',
            display: 'inline-flex',
            alignItems: 'center',
            gap: '6px',
          }}
        >
          <Plus size={15} /> Nouveau tenant
        </Link>
      </div>

      {deleteError && (
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
          <span>{deleteError}</span>
        </div>
      )}

      {loading && <LoadingState />}
      {!loading && error && <ErrorState message={error} onRetry={() => void load()} />}

      {!loading && !error && (
        <>
          <div style={{ marginBottom: '20px', position: 'relative', maxWidth: '420px' }}>
            <Search size={15} style={{ position: 'absolute', left: '12px', top: '50%', transform: 'translateY(-50%)', color: 'var(--text-muted)' }} />
            <input
              type="text"
              placeholder="Rechercher un tenant par nom ou email..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              style={{
                width: '100%',
                padding: '10px 14px 10px 34px',
                borderRadius: '6px',
                border: '1px solid var(--border)',
                fontSize: '13px',
                background: '#FFFFFF',
              }}
            />
          </div>

          <div style={{ background: '#FFFFFF', borderRadius: 'var(--radius)', boxShadow: 'var(--shadow)', overflow: 'hidden' }}>
            {tenants.length === 0 ? (
              <div style={{ textAlign: 'center', padding: '48px 20px', color: 'var(--text-muted)' }}>
                <Building2 size={28} style={{ marginBottom: '8px', opacity: 0.5 }} />
                <p>Aucun tenant enregistré</p>
              </div>
            ) : filteredTenants.length === 0 ? (
              <div style={{ textAlign: 'center', padding: '48px 20px', color: 'var(--text-muted)' }}>
                <p>Aucun tenant ne correspond à « {searchTerm} »</p>
              </div>
            ) : (
              <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '14px' }}>
                <thead style={{ background: '#F5EFE8', borderBottom: '2px solid var(--gold)' }}>
                  <tr>
                    <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Entreprise</th>
                    <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Email</th>
                    <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Employés</th>
                    <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Statut</th>
                    <th style={{ padding: '12px 16px', textAlign: 'right', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {filteredTenants.map((t) => (
                    <tr key={t.id} style={{ borderBottom: '1px solid var(--border)' }}>
                      <td style={{ padding: '14px 16px' }}>
                        <Link to={`/tenants/${t.id}`} style={{ color: 'var(--crimson)', fontWeight: 600, textDecoration: 'none' }}>
                          {t.name}
                        </Link>
                      </td>
                      <td style={{ padding: '14px 16px', color: 'var(--text-secondary)' }}>{t.email}</td>
                      <td style={{ padding: '14px 16px' }}>{t.employee_count}</td>
                      <td style={{ padding: '14px 16px' }}><StatusBadge active={t.is_active} /></td>
                      <td style={{ padding: '14px 16px', textAlign: 'right' }}>
                        <div style={{ display: 'flex', gap: '10px', justifyContent: 'flex-end', alignItems: 'center' }}>
                          <Link to={`/tenants/${t.id}/modifier`} style={{ color: 'var(--text-secondary)', fontWeight: 600, textDecoration: 'none', fontSize: '13px' }}>
                            Modifier
                          </Link>
                          <button
                            onClick={() => void handleDelete(t)}
                            disabled={deletingId === t.id}
                            title="Supprimer ce tenant"
                            style={{
                              display: 'inline-flex',
                              alignItems: 'center',
                              gap: '4px',
                              background: 'transparent',
                              border: 'none',
                              color: 'var(--danger)',
                              fontSize: '13px',
                              fontWeight: 600,
                              cursor: deletingId === t.id ? 'default' : 'pointer',
                              padding: '4px 6px',
                            }}
                          >
                            {deletingId === t.id ? <Loader2 size={14} className="tp-spin" /> : <Trash2 size={14} />}
                            Supprimer
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
    </div>
  )
}
