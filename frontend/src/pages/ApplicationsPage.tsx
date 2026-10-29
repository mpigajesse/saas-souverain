// Catalogue des applications hébergées (refonte V3, ADR-009) : SaaS externes
// qu'AMANE exécute dans les clusters des PME. Validation contre le registre,
// retrait, rattachement aux licences (page Licences).

import type React from 'react'
import { useCallback, useEffect, useState } from 'react'
import { AlertCircle, AppWindow, Archive, BadgeCheck, Loader2, Pencil, Plus, RotateCw, Trash2, X } from 'lucide-react'
import {
  createApplication,
  deleteApplication,
  getApplications,
  normalizeApiError,
  retirerApplication,
  updateApplication,
  validerApplication,
  type ApplicationWritePayload,
} from '../api/client'
import { STATUT_APPLICATION_LABELS } from '../types/index'
import type { Application, FieldErrors, StatutApplication } from '../types/index'

const EXEMPLE_MANIFESTE = `{
  "type_artefact": "docker",
  "image": "editeurs/facturation@sha256:<64 caractères hexadécimaux>",
  "commande": ["/app/serveur"],
  "port": 8080,
  "sante": "/sante",
  "variables": {"APP_ENV": "production"},
  "base_de_donnees": {
    "variables": {"DATABASE_URL": "postgres://{utilisateur}:{mot_de_passe}@{hote}:{port}/{base}"}
  },
  "volumes": [{"nom": "fichiers", "chemin": "/data"}]
}`

const STATUT_COULEURS: Record<StatutApplication, { bg: string; fg: string }> = {
  brouillon: { bg: '#FFF3CD', fg: '#856404' },
  validee: { bg: '#D1FAE5', fg: '#065F46' },
  retiree: { bg: '#F3F4F6', fg: '#6B7280' },
}

function formatDate(iso: string | null): string {
  if (!iso) return '—'
  return new Date(iso).toLocaleDateString('fr-FR')
}

/** `dépôt@sha256:abcdef…` → `dépôt@sha256:abcdef123456…` (lisible dans un tableau). */
function imageCourte(image: string): string {
  const [depot, digest] = image.split('@')
  if (!digest) return image
  return `${depot}@${digest.slice(0, 19)}…`
}

function StatutBadge({ statut }: { statut: StatutApplication }) {
  const c = STATUT_COULEURS[statut]
  return (
    <span style={{ background: c.bg, color: c.fg, padding: '2px 8px', borderRadius: '10px', fontSize: '12px', fontWeight: 600 }}>
      {STATUT_APPLICATION_LABELS[statut]}
    </span>
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

const thStyle: React.CSSProperties = {
  padding: '12px 16px',
  textAlign: 'left',
  color: 'var(--text-secondary)',
  fontSize: '11px',
  textTransform: 'uppercase',
}

function boutonLigne(fond: string, couleur: string, bordure: string, occupe: boolean): React.CSSProperties {
  return {
    background: fond,
    color: couleur,
    border: `1px solid ${bordure}`,
    borderRadius: '4px',
    padding: '5px 8px',
    fontSize: '11px',
    fontWeight: 600,
    cursor: occupe ? 'default' : 'pointer',
    display: 'flex',
    alignItems: 'center',
    gap: '4px',
  }
}

interface FormState {
  nom: string
  editeur: string
  version: string
  manifeste: string
}

const FORM_VIDE: FormState = { nom: '', editeur: '', version: '1.0', manifeste: EXEMPLE_MANIFESTE }

export default function ApplicationsPage() {
  const [applications, setApplications] = useState<Application[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const [showModal, setShowModal] = useState(false)
  const [editing, setEditing] = useState<Application | null>(null)
  const [form, setForm] = useState<FormState>(FORM_VIDE)
  const [submitting, setSubmitting] = useState(false)
  const [formError, setFormError] = useState<string | null>(null)
  const [fieldErrors, setFieldErrors] = useState<FieldErrors | null>(null)

  const [rowActionId, setRowActionId] = useState<string | null>(null)
  const [rowError, setRowError] = useState<string | null>(null)

  const load = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      setApplications(await getApplications())
    } catch (err) {
      setError(normalizeApiError(err).message)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    // Différé d'une micro-tâche (React 19, règle react-hooks/set-state-in-effect).
    queueMicrotask(() => {
      void load()
    })
  }, [load])

  function remplacer(app: Application) {
    setApplications((prev) => prev.map((a) => (a.id === app.id ? app : a)))
  }

  function openCreate() {
    setEditing(null)
    setForm(FORM_VIDE)
    setFormError(null)
    setFieldErrors(null)
    setShowModal(true)
  }

  function openEdit(app: Application) {
    setEditing(app)
    setForm({ nom: app.nom, editeur: app.editeur, version: app.version, manifeste: JSON.stringify(app.manifeste, null, 2) })
    setFormError(null)
    setFieldErrors(null)
    setShowModal(true)
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setFormError(null)
    setFieldErrors(null)
    let manifeste: unknown
    try {
      manifeste = JSON.parse(form.manifeste)
    } catch {
      setFieldErrors({ manifeste: ['JSON invalide : vérifiez les guillemets et les virgules.'] })
      return
    }
    const payload: ApplicationWritePayload = { nom: form.nom, editeur: form.editeur, version: form.version }
    // Une application validée a un manifeste figé : on ne le renvoie pas.
    if (!editing || editing.statut === 'brouillon') payload.manifeste = manifeste
    setSubmitting(true)
    try {
      if (editing) {
        remplacer(await updateApplication(editing.id, payload))
      } else {
        const created = await createApplication(payload)
        setApplications((prev) => [created, ...prev])
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

  async function action(app: Application, faire: () => Promise<void>) {
    setRowError(null)
    setRowActionId(app.id)
    try {
      await faire()
    } catch (err) {
      setRowError(normalizeApiError(err).message)
    } finally {
      setRowActionId(null)
    }
  }

  function handleValider(app: Application) {
    void action(app, async () => remplacer(await validerApplication(app.id)))
  }

  function handleRetirer(app: Application) {
    const suite = app.licences_actives > 0
      ? ` Elle est rattachée à ${app.licences_actives} licence(s) active(s) : les piles générées ensuite ne la contiendront plus.`
      : ''
    if (!window.confirm(`Retirer ${app.nom} ${app.version} ?${suite}`)) return
    void action(app, async () => remplacer(await retirerApplication(app.id)))
  }

  function handleDelete(app: Application) {
    if (!window.confirm(`Supprimer définitivement ${app.nom} ${app.version} ?`)) return
    void action(app, async () => {
      await deleteApplication(app.id)
      setApplications((prev) => prev.filter((a) => a.id !== app.id))
    })
  }

  const manifesteFige = editing !== null && editing.statut !== 'brouillon'

  return (
    <div>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '28px' }}>
        <div>
          <h1 style={{ fontSize: '22px', fontWeight: 700, color: 'var(--text-primary)' }}>Applications hébergées</h1>
          <p style={{ color: 'var(--text-secondary)', marginTop: '4px' }}>
            SaaS externes exécutés dans les clusters des PME — image épinglée par digest, réseau sans sortie
          </p>
        </div>
        <button
          onClick={openCreate}
          disabled={loading}
          style={{
            background: 'var(--crimson)',
            color: '#FFFFFF',
            border: 'none',
            borderRadius: '6px',
            padding: '10px 16px',
            fontWeight: 600,
            cursor: loading ? 'default' : 'pointer',
            opacity: loading ? 0.6 : 1,
            boxShadow: 'var(--shadow)',
            display: 'flex',
            alignItems: 'center',
            gap: '6px',
          }}
        >
          <Plus size={15} /> Nouvelle application
        </button>
      </div>

      <div
        style={{
          background: '#FFFFFF',
          border: '1px solid var(--border)',
          borderRadius: 'var(--radius)',
          padding: '14px 18px',
          fontSize: '13px',
          color: 'var(--text-secondary)',
          lineHeight: 1.6,
          marginBottom: '20px',
        }}
      >
        <strong style={{ color: 'var(--text-primary)' }}>Cycle de vie.</strong> Un brouillon se modifie librement. La
        validation vérifie que l'image est publiée dans le registre ; le manifeste est alors figé (changer d'image, c'est
        créer une nouvelle version). Seule une application validée se rattache à une licence, depuis la page Licences.
        Une application retirée disparaît des piles générées ensuite.
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

      {loading && (
        <div style={{ display: 'flex', alignItems: 'center', gap: '10px', color: 'var(--text-secondary)', padding: '48px 0' }}>
          <Loader2 size={18} className="ap-spin" />
          <span>Chargement des applications...</span>
        </div>
      )}
      <style>{`.ap-spin { animation: ap-spin 0.8s linear infinite; } @keyframes ap-spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }`}</style>

      {!loading && error && (
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '12px',
            background: '#FDECEC',
            border: '1px solid #F3B7B7',
            color: 'var(--danger)',
            borderRadius: 'var(--radius)',
            padding: '18px 20px',
          }}
        >
          <AlertCircle size={18} />
          <span style={{ flex: 1 }}>{error}</span>
          <button onClick={() => void load()} style={boutonLigne('#FFFFFF', 'var(--danger)', 'var(--danger)', false)}>
            <RotateCw size={14} /> Réessayer
          </button>
        </div>
      )}

      {!loading && !error && (
        <div style={{ background: '#FFFFFF', borderRadius: 'var(--radius)', boxShadow: 'var(--shadow)', overflow: 'hidden' }}>
          {applications.length === 0 ? (
            <div style={{ textAlign: 'center', padding: '48px 20px', color: 'var(--text-muted)' }}>
              <AppWindow size={28} style={{ marginBottom: '8px', opacity: 0.5 }} />
              <p>Aucune application au catalogue</p>
            </div>
          ) : (
            <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '14px' }}>
              <thead style={{ background: '#F5EFE8', borderBottom: '2px solid var(--gold)' }}>
                <tr>
                  <th style={thStyle}>Application</th>
                  <th style={thStyle}>Éditeur</th>
                  <th style={thStyle}>Image</th>
                  <th style={thStyle}>Statut</th>
                  <th style={thStyle}>PME</th>
                  <th style={{ ...thStyle, textAlign: 'right' }}>Actions</th>
                </tr>
              </thead>
              <tbody>
                {applications.map((app) => {
                  const occupe = rowActionId === app.id
                  return (
                    <tr key={app.id} style={{ borderBottom: '1px solid var(--border)' }}>
                      <td style={{ padding: '14px 16px' }}>
                        <div style={{ fontWeight: 600 }}>{app.nom} <span style={{ color: 'var(--text-muted)', fontWeight: 500 }}>{app.version}</span></div>
                        <div style={{ fontSize: '11px', color: 'var(--text-muted)', marginTop: '2px' }}>
                          {app.validee_le ? `Validée le ${formatDate(app.validee_le)}` : `Créée le ${formatDate(app.created_at)}`}
                        </div>
                      </td>
                      <td style={{ padding: '14px 16px' }}>{app.editeur}</td>
                      <td style={{ padding: '14px 16px', fontFamily: 'monospace', fontSize: '11px', color: 'var(--text-secondary)' }} title={app.image}>
                        {imageCourte(app.image)}
                      </td>
                      <td style={{ padding: '14px 16px' }}><StatutBadge statut={app.statut} /></td>
                      <td style={{ padding: '14px 16px', fontWeight: 600 }}>{app.licences_actives}</td>
                      <td style={{ padding: '14px 16px', textAlign: 'right' }}>
                        <div style={{ display: 'flex', gap: '8px', justifyContent: 'flex-end' }}>
                          {app.statut === 'brouillon' && (
                            <button onClick={() => handleValider(app)} disabled={occupe} title="Vérifier l'image dans le registre et valider"
                              style={boutonLigne('#E8F5E9', '#2E7D32', '#A5D6A7', occupe)}>
                              {occupe ? <Loader2 size={12} className="ap-spin" /> : <BadgeCheck size={12} />} Valider
                            </button>
                          )}
                          {app.statut !== 'retiree' && (
                            <button onClick={() => openEdit(app)} title="Modifier la fiche"
                              style={boutonLigne('#F5EFE8', 'var(--text-primary)', 'var(--border)', false)}>
                              <Pencil size={12} /> Modifier
                            </button>
                          )}
                          {app.statut === 'validee' && (
                            <button onClick={() => handleRetirer(app)} disabled={occupe} title="Retirer des piles générées"
                              style={boutonLigne('#FEE8E8', 'var(--danger)', '#EF9A9A', occupe)}>
                              {occupe ? <Loader2 size={12} className="ap-spin" /> : <Archive size={12} />} Retirer
                            </button>
                          )}
                          <button onClick={() => handleDelete(app)} disabled={occupe} title="Supprimer (impossible si rattachée)"
                            style={{ ...boutonLigne('transparent', 'var(--danger)', 'transparent', occupe) }}>
                            <Trash2 size={12} /> Supprimer
                          </button>
                        </div>
                      </td>
                    </tr>
                  )
                })}
              </tbody>
            </table>
          )}
        </div>
      )}

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
          <div style={{ background: '#FFFFFF', borderRadius: '12px', padding: '28px', maxWidth: '640px', width: '100%', maxHeight: '90vh', overflowY: 'auto', boxShadow: 'var(--shadow)' }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '8px' }}>
              <h3 style={{ fontSize: '18px', fontWeight: 700 }}>{editing ? `Modifier ${editing.nom} ${editing.version}` : 'Nouvelle application'}</h3>
              <button onClick={() => setShowModal(false)} style={{ background: 'transparent', border: 'none', cursor: 'pointer', color: 'var(--text-muted)' }}>
                <X size={18} />
              </button>
            </div>
            <p style={{ fontSize: '13px', color: 'var(--text-secondary)', marginBottom: '20px' }}>
              L'image doit être publiée dans le registre de l'éditeur AMANE et désignée par son digest. Les identifiants de
              la base ne figurent jamais ici : ils sont générés dans le cluster de la PME et injectés au démarrage.
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
              <div style={{ display: 'flex', gap: '12px' }}>
                <div style={{ flex: 2 }}>
                  <label style={labelStyle}>Nom</label>
                  <input required value={form.nom} onChange={(e) => setForm({ ...form, nom: e.target.value })} style={inputStyle} />
                  <FieldError messages={fieldErrors?.nom} />
                </div>
                <div style={{ flex: 1 }}>
                  <label style={labelStyle}>Version</label>
                  <input required value={form.version} onChange={(e) => setForm({ ...form, version: e.target.value })} style={inputStyle} />
                  <FieldError messages={fieldErrors?.version} />
                </div>
              </div>
              <div>
                <label style={labelStyle}>Éditeur du SaaS</label>
                <input required value={form.editeur} onChange={(e) => setForm({ ...form, editeur: e.target.value })} style={inputStyle} />
                <FieldError messages={fieldErrors?.editeur} />
              </div>
              <div>
                <label style={labelStyle}>
                  Manifeste (JSON){manifesteFige ? ' — figé depuis la validation' : ''}
                </label>
                <textarea
                  value={form.manifeste}
                  onChange={(e) => setForm({ ...form, manifeste: e.target.value })}
                  readOnly={manifesteFige}
                  rows={14}
                  spellCheck={false}
                  style={{ ...inputStyle, fontFamily: 'monospace', fontSize: '12px', background: manifesteFige ? '#F9FAFB' : '#FFFFFF' }}
                />
                <FieldError messages={fieldErrors?.manifeste} />
              </div>

              <div style={{ display: 'flex', gap: '10px', marginTop: '10px' }}>
                <button
                  type="button"
                  onClick={() => setShowModal(false)}
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
                  {submitting && <Loader2 size={14} className="ap-spin" />}
                  {editing ? 'Enregistrer' : 'Créer le brouillon'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  )
}
