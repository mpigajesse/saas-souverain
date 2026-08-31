import type React from 'react'
import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { getTenants } from '../api/client'
import StatusBadge from '../components/StatusBadge'
import type { Tenant } from '../types/index'

export default function TenantsPage() {
  const [tenants, setTenants] = useState<Tenant[]>([])
  const [loading, setLoading] = useState(true)
  const [searchTerm, setSearchTerm] = useState('')
  
  // Modal de création de Tenant
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [newTenantName, setNewTenantName] = useState('')
  const [newSubdomain, setNewSubdomain] = useState('')

  useEffect(() => {
    let cancelled = false
    async function load() {
      try {
        const data = await getTenants()
        if (!cancelled) setTenants(data)
      } catch (err: unknown) {
        console.error(err)
      } finally {
        if (!cancelled) setLoading(false)
      }
    }
    void load()
    return () => {
      cancelled = true
    }
  }, [])

  const handleCreateTenant = (e: React.FormEvent) => {
    e.preventDefault()
    if (!newTenantName || !newSubdomain) return

    const newTenant: Tenant = {
      id: `tenant-${Date.now()}`,
      name: newTenantName,
      subdomain: newSubdomain.toLowerCase().replace(/\s+/g, '-'),
      created_at: new Date().toISOString(),
      is_active: true,
    }

    setTenants([newTenant, ...tenants])
    setNewTenantName('')
    setNewSubdomain('')
    setShowCreateModal(false)
  }

  const filteredTenants = tenants.filter((t) =>
    t.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    t.subdomain.toLowerCase().includes(searchTerm.toLowerCase())
  )

  if (loading) return <p style={{ color: 'var(--text-secondary)' }}>Chargement des entreprises...</p>

  return (
    <div>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '24px' }}>
        <div>
          <h1 style={{ fontSize: '22px', fontWeight: 700, color: 'var(--text-primary)' }}>
            Entreprises Abonnées (Tenants)
          </h1>
          <p style={{ color: 'var(--text-secondary)', marginTop: '4px' }}>
            Gestion des comptes PME clients et déploiement des clusters locaux
          </p>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
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
          }}
        >
          ＋ Inscrire un Nouveau Tenant PME
        </button>
      </div>

      {/* Barre de recherche */}
      <div style={{ marginBottom: '20px' }}>
        <input
          type="text"
          placeholder="🔍 Rechercher une PME par nom ou sous-domaine..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          style={{
            width: '100%',
            maxWidth: '420px',
            padding: '10px 14px',
            borderRadius: '6px',
            border: '1px solid var(--border)',
            fontSize: '13px',
            background: '#FFFFFF',
          }}
        />
      </div>

      <div style={{ background: '#FFFFFF', borderRadius: 'var(--radius)', boxShadow: 'var(--shadow)', overflow: 'hidden' }}>
        <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '14px' }}>
          <thead style={{ background: '#F5EFE8', borderBottom: '2px solid var(--gold)' }}>
            <tr>
              <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Entreprise / PME</th>
              <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Sous-Domaine Métier</th>
              <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Date d'Abonnement</th>
              <th style={{ padding: '12px 16px', textAlign: 'left', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Statut</th>
              <th style={{ padding: '12px 16px', textAlign: 'right', color: 'var(--text-secondary)', fontSize: '11px', textTransform: 'uppercase' }}>Action</th>
            </tr>
          </thead>
          <tbody>
            {filteredTenants.map((t) => (
              <tr key={t.id} style={{ borderBottom: '1px solid var(--border)' }}>
                <td style={{ padding: '14px 16px', fontWeight: 600 }}>{t.name}</td>
                <td style={{ padding: '14px 16px', fontFamily: 'monospace', color: 'var(--text-secondary)' }}>{t.subdomain}.amane.local</td>
                <td style={{ padding: '14px 16px', color: 'var(--text-muted)' }}>{new Date(t.created_at).toLocaleDateString('fr-FR')}</td>
                <td style={{ padding: '14px 16px' }}><StatusBadge active={t.is_active} /></td>
                <td style={{ padding: '14px 16px', textAlign: 'right' }}>
                  <Link to={`/tenants/${t.id}`} style={{ color: 'var(--crimson)', fontWeight: 600, textDecoration: 'none' }}>
                    Consulter Dossier →
                  </Link>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Modal Inscription Tenant */}
      {showCreateModal && (
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
          <div style={{ background: '#FFFFFF', borderRadius: '12px', padding: '28px', maxWidth: '440px', width: '100%', boxShadow: 'var(--shadow)' }}>
            <h3 style={{ fontSize: '18px', fontWeight: 700, marginBottom: '12px' }}>Inscrire une nouvelle PME (Tenant)</h3>

            <form onSubmit={handleCreateTenant} style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>
              <div>
                <label style={{ fontSize: '12px', color: 'var(--text-secondary)', display: 'block', marginBottom: '4px' }}>Nom de l'Entreprise PME</label>
                <input
                  type="text"
                  required
                  placeholder="ex: SOUPLEX PHARMA"
                  value={newTenantName}
                  onChange={(e) => {
                    setNewTenantName(e.target.value)
                    setNewSubdomain(e.target.value.toLowerCase().replace(/\s+/g, '-'))
                  }}
                  style={{ width: '100%', padding: '8px', borderRadius: '6px', border: '1px solid var(--border)' }}
                />
              </div>

              <div>
                <label style={{ fontSize: '12px', color: 'var(--text-secondary)', display: 'block', marginBottom: '4px' }}>Sous-domaine réseau</label>
                <input
                  type="text"
                  required
                  placeholder="ex: souplex-pharma"
                  value={newSubdomain}
                  onChange={(e) => setNewSubdomain(e.target.value)}
                  style={{ width: '100%', padding: '8px', borderRadius: '6px', border: '1px solid var(--border)' }}
                />
              </div>

              <div style={{ display: 'flex', gap: '10px', marginTop: '10px' }}>
                <button
                  type="button"
                  onClick={() => setShowCreateModal(false)}
                  style={{ flex: 1, padding: '10px', background: '#FFF', border: '1px solid var(--border)', borderRadius: '6px', cursor: 'pointer' }}
                >
                  Annuler
                </button>
                <button
                  type="submit"
                  style={{ flex: 1, padding: '10px', background: 'var(--crimson)', color: '#FFF', border: 'none', borderRadius: '6px', fontWeight: 600, cursor: 'pointer' }}
                >
                  Créer le Tenant
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  )
}
