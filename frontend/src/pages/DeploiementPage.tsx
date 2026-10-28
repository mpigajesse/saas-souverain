// Déploiement — image Docker du nœud PME, publiée sur le registre local de
// l'éditeur (port 5000). Données réelles via GET /api/node-image/ (voir
// CONTRAT-API.md §2). Aucune image publiée ou registre injoignable sont des
// états normaux, affichés calmement — pas des pannes de la console.

import { useCallback, useEffect, useState } from 'react'
import {
  AlertCircle,
  CheckCircle2,
  Clock,
  Copy,
  Download,
  Hash,
  Inbox,
  PackageCheck,
  RotateCw,
  Loader2,
  Terminal,
  Wifi,
  WifiOff,
} from 'lucide-react'
import { downloadNodeImageUrl, getNodeImage, normalizeApiError } from '../api/client'
import type { NodeImage } from '../types/index'

function LoadingState() {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: '10px', color: 'var(--text-secondary)', padding: '48px 0' }}>
      <Loader2 size={18} className="dp-spin" />
      <span>Chargement de l'état de l'image du nœud PME...</span>
      <style>{`.dp-spin { animation: dp-spin 0.8s linear infinite; } @keyframes dp-spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }`}</style>
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

function EmptyState({ onRetry }: { onRetry: () => void }) {
  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        gap: '14px',
        textAlign: 'center',
        background: '#FFFFFF',
        border: '1px dashed var(--border)',
        borderRadius: 'var(--radius)',
        padding: '56px 32px',
      }}
    >
      <Inbox size={32} color="var(--text-muted)" />
      <div>
        <div style={{ fontWeight: 700, color: 'var(--text-primary)', marginBottom: '6px' }}>
          Aucune image publiée
        </div>
        <p style={{ color: 'var(--text-secondary)', fontSize: '13px', maxWidth: '440px' }}>
          Construisez l'image du nœud PME puis republiez-la sur le registre local avec :
        </p>
        <code
          style={{
            display: 'inline-block',
            marginTop: '10px',
            background: '#F5EFE8',
            border: '1px solid var(--border)',
            borderRadius: '6px',
            padding: '8px 14px',
            fontSize: '12.5px',
            color: 'var(--text-primary)',
          }}
        >
          spike/build-node.ps1
        </code>
      </div>
      <button
        onClick={onRetry}
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '6px',
          background: '#FFFFFF',
          border: '1px solid var(--border)',
          color: 'var(--text-primary)',
          borderRadius: '6px',
          padding: '7px 14px',
          fontSize: '12px',
          fontWeight: 600,
          cursor: 'pointer',
        }}
      >
        <RotateCw size={14} /> Vérifier à nouveau
      </button>
    </div>
  )
}

function formatTaille(octets: number | null): string {
  if (octets === null) return '—'
  const go = octets / (1024 * 1024 * 1024)
  if (go >= 1) return `${go.toFixed(2)} Go`
  const mo = octets / (1024 * 1024)
  return `${mo.toFixed(1)} Mo`
}

function formatDate(iso: string | null): string {
  if (!iso) return '—'
  return new Date(iso).toLocaleString('fr-FR')
}

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false)

  const handleCopy = useCallback(() => {
    navigator.clipboard
      .writeText(text)
      .then(() => {
        setCopied(true)
        setTimeout(() => setCopied(false), 1800)
      })
      .catch(() => {
        // Presse-papiers indisponible (contexte non sécurisé, permission
        // refusée) : la commande reste sélectionnable manuellement.
      })
  }, [text])

  return (
    <button
      onClick={handleCopy}
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: '5px',
        background: copied ? 'rgba(16,185,129,.15)' : 'rgba(255,255,255,.08)',
        border: `1px solid ${copied ? 'rgba(16,185,129,.4)' : 'rgba(255,255,255,.15)'}`,
        color: copied ? '#6EE7B7' : 'rgba(255,255,255,.8)',
        borderRadius: '6px',
        padding: '5px 10px',
        fontSize: '11px',
        fontWeight: 600,
        cursor: 'pointer',
        flexShrink: 0,
      }}
    >
      {copied ? <CheckCircle2 size={12} /> : <Copy size={12} />}
      {copied ? 'Copié' : 'Copier'}
    </button>
  )
}

function CommandLine({ command }: { command: string }) {
  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: '10px',
        background: '#0f172a',
        border: '1px solid rgba(255,255,255,.1)',
        borderRadius: '8px',
        padding: '10px 12px',
      }}
    >
      <Terminal size={14} color="rgba(255,255,255,.4)" style={{ flexShrink: 0 }} />
      <code style={{ flex: 1, color: '#E2E8F0', fontSize: '12.5px', fontFamily: 'monospace', overflowX: 'auto', whiteSpace: 'nowrap' }}>
        {command}
      </code>
      <CopyButton text={command} />
    </div>
  )
}

export default function DeploiementPage() {
  const [data, setData] = useState<NodeImage | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const image = await getNodeImage()
      setData(image)
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

  return (
    <div>
      <div style={{ marginBottom: '24px' }}>
        <h1 style={{ fontSize: '22px', fontWeight: 700, color: 'var(--text-primary)' }}>Déploiement</h1>
        <p style={{ color: 'var(--text-secondary)', marginTop: '4px' }}>
          Image Docker du nœud PME, distribuée par le registre local de l'éditeur
        </p>
      </div>

      {loading && <LoadingState />}
      {!loading && error && <ErrorState message={error} onRetry={() => void load()} />}

      {!loading && !error && data && !data.disponible && <EmptyState onRetry={() => void load()} />}

      {!loading && !error && data && data.disponible && (
        <>
          {/* Bandeau d'état de l'image */}
          <div
            style={{
              background: 'linear-gradient(135deg, #0f172a 0%, #1e1b4b 60%, #312e81 100%)',
              borderRadius: '16px',
              padding: '26px 30px',
              marginBottom: '28px',
              color: '#fff',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', flexWrap: 'wrap', gap: '16px', marginBottom: '20px' }}>
              <div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '1.3rem', fontWeight: 800, marginBottom: '4px' }}>
                  <PackageCheck size={22} /> {data.image ?? 'Image du nœud PME'}
                </div>
                <div style={{ fontFamily: 'monospace', fontSize: '0.78rem', color: 'rgba(255,255,255,.4)' }}>
                  Publiée le {formatDate(data.construit_le)}
                </div>
              </div>
              <span
                style={{
                  display: 'inline-flex',
                  alignItems: 'center',
                  gap: '7px',
                  borderRadius: '20px',
                  padding: '6px 16px',
                  fontSize: '0.8rem',
                  fontWeight: 700,
                  background: 'rgba(16,185,129,.15)',
                  color: '#6EE7B7',
                  border: '1px solid rgba(16,185,129,.3)',
                }}
              >
                <CheckCircle2 size={14} /> Image disponible
              </span>
            </div>

            <div style={{ display: 'flex', flexWrap: 'wrap', gap: '10px', borderTop: '1px solid rgba(255,255,255,.08)', paddingTop: '20px', marginBottom: '20px' }}>
              <div style={{ background: 'rgba(255,255,255,.05)', border: '1px solid rgba(255,255,255,.1)', borderRadius: '10px', padding: '10px 16px', minWidth: '120px' }}>
                <div style={{ fontSize: '0.67rem', textTransform: 'uppercase', letterSpacing: '.08em', color: 'rgba(255,255,255,.38)', marginBottom: '4px' }}>Version</div>
                <div style={{ fontSize: '0.88rem', fontWeight: 700, color: '#fff', fontFamily: 'monospace' }}>{data.version ?? '—'}</div>
              </div>
              <div style={{ background: 'rgba(255,255,255,.05)', border: '1px solid rgba(255,255,255,.1)', borderRadius: '10px', padding: '10px 16px', minWidth: '120px' }}>
                <div style={{ fontSize: '0.67rem', textTransform: 'uppercase', letterSpacing: '.08em', color: 'rgba(255,255,255,.38)', marginBottom: '4px' }}>Taille</div>
                <div style={{ fontSize: '0.88rem', fontWeight: 700, color: '#fff', fontFamily: 'monospace' }}>{formatTaille(data.taille_octets)}</div>
              </div>
              <div style={{ background: 'rgba(255,255,255,.05)', border: '1px solid rgba(255,255,255,.1)', borderRadius: '10px', padding: '10px 16px', minWidth: '120px' }}>
                <div style={{ fontSize: '0.67rem', textTransform: 'uppercase', letterSpacing: '.08em', color: 'rgba(255,255,255,.38)', marginBottom: '4px' }}>Fichier</div>
                <div style={{ fontSize: '0.88rem', fontWeight: 700, color: '#fff', fontFamily: 'monospace' }}>{data.fichier ?? '—'}</div>
              </div>
              <div style={{ background: 'rgba(255,255,255,.05)', border: '1px solid rgba(255,255,255,.1)', borderRadius: '10px', padding: '10px 16px', flex: 1, minWidth: '260px' }}>
                <div style={{ fontSize: '0.67rem', textTransform: 'uppercase', letterSpacing: '.08em', color: 'rgba(255,255,255,.38)', marginBottom: '4px', display: 'flex', alignItems: 'center', gap: '5px' }}>
                  <Hash size={11} /> SHA-256
                </div>
                <div style={{ fontSize: '0.78rem', fontWeight: 700, color: '#fff', fontFamily: 'monospace', wordBreak: 'break-all' }}>{data.sha256 ?? '—'}</div>
              </div>
            </div>

            <a
              href={downloadNodeImageUrl()}
              style={{
                display: 'inline-flex',
                alignItems: 'center',
                gap: '8px',
                background: 'var(--gold)',
                color: '#2D1419',
                borderRadius: '8px',
                padding: '10px 20px',
                fontSize: '0.85rem',
                fontWeight: 700,
                textDecoration: 'none',
              }}
            >
              <Download size={16} /> Télécharger l'archive ({formatTaille(data.taille_octets)})
            </a>
          </div>

          {/* Registre Docker local */}
          <div style={{ background: '#FFFFFF', border: '1px solid var(--border)', borderRadius: 'var(--radius)', padding: '22px 24px', boxShadow: 'var(--shadow)', marginBottom: '20px' }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '10px', marginBottom: '14px' }}>
              <h2 style={{ fontSize: '1rem', fontWeight: 700, color: 'var(--text-primary)' }}>Registre Docker local</h2>
              {data.registre.joignable ? (
                <span style={{ display: 'inline-flex', alignItems: 'center', gap: '6px', fontSize: '12px', fontWeight: 700, color: '#065F46', background: '#D1FAE5', borderRadius: '20px', padding: '4px 12px' }}>
                  <Wifi size={13} /> Joignable — {data.registre.hote}
                </span>
              ) : (
                <span style={{ display: 'inline-flex', alignItems: 'center', gap: '6px', fontSize: '12px', fontWeight: 700, color: '#6B7280', background: '#F3F4F6', borderRadius: '20px', padding: '4px 12px' }}>
                  <WifiOff size={13} /> Injoignable — {data.registre.hote}
                </span>
              )}
            </div>

            <p style={{ color: 'var(--text-secondary)', fontSize: '13px', marginBottom: '14px' }}>
              Les machines PME du réseau local récupèrent l'image directement depuis ce registre :
            </p>

            <div style={{ borderRadius: '10px', overflow: 'hidden', marginBottom: '14px' }}>
              <CommandLine command={data.registre.commande_pull ?? `docker pull ${data.registre.hote}/${data.image ?? ''}`} />
            </div>

            {data.registre.tags.length > 0 && (
              <div style={{ display: 'flex', flexWrap: 'wrap', gap: '8px', alignItems: 'center' }}>
                <span style={{ fontSize: '12px', color: 'var(--text-muted)' }}>Tags publiés :</span>
                {data.registre.tags.map((tag) => (
                  <span key={tag} style={{ fontFamily: 'monospace', fontSize: '12px', background: '#F5EFE8', border: '1px solid var(--border)', borderRadius: '6px', padding: '3px 9px' }}>
                    {tag}
                  </span>
                ))}
              </div>
            )}

            {!data.registre.joignable && (
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '12px', color: 'var(--text-muted)', marginTop: '10px' }}>
                <Clock size={13} /> Registre non interrogeable pour l'instant — la commande ci-dessus reste valable dès qu'il répond.
              </div>
            )}
          </div>

          {/* Procédure de déploiement PME */}
          <div style={{ background: '#FFFFFF', border: '1px solid var(--border)', borderRadius: 'var(--radius)', padding: '22px 24px', boxShadow: 'var(--shadow)' }}>
            <h2 style={{ fontSize: '1rem', fontWeight: 700, color: 'var(--text-primary)', marginBottom: '16px' }}>Procédure de déploiement côté PME</h2>

            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(320px, 1fr))', gap: '18px' }}>
              <div>
                <div style={{ fontSize: '13px', fontWeight: 700, color: 'var(--text-primary)', marginBottom: '8px' }}>
                  Voie 1 — Tirer l'image depuis le registre
                </div>
                <p style={{ fontSize: '12.5px', color: 'var(--text-secondary)', marginBottom: '10px' }}>
                  La machine PME doit être sur le même réseau local que le PC éditeur.
                </p>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                  <CommandLine command={data.registre.commande_pull ?? `docker pull ${data.registre.hote}/${data.image ?? ''}`} />
                  <CommandLine command={`docker run -d --name ss-node ${data.image ?? 'ss-node:v2'}`} />
                </div>
              </div>

              <div>
                <div style={{ fontSize: '13px', fontWeight: 700, color: 'var(--text-primary)', marginBottom: '8px' }}>
                  Voie 2 — Charger l'archive téléchargée
                </div>
                <p style={{ fontSize: '12.5px', color: 'var(--text-secondary)', marginBottom: '10px' }}>
                  Sans réseau commun : transférez l'archive téléchargée ci-dessus sur la machine PME (clé USB, partage).
                </p>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                  <CommandLine command={`docker load -i ${data.fichier ?? 'ss-node-v2.tar.gz'}`} />
                  <CommandLine command={`docker run -d --name ss-node ${data.image ?? 'ss-node:v2'}`} />
                </div>
              </div>
            </div>
          </div>
        </>
      )}
    </div>
  )
}
