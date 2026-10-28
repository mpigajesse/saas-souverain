// Onboarding PME — remplace templates/tenants/bienvenue.html.
// Données réelles uniquement : GET /api/tenants/me/onboarding/ (voir
// CONTRAT-API.md §3). Le jeton d'enregistrement vient de la session
// (user.tenant.registration_token, exposé par /api/auth/me/) car il n'est pas
// renvoyé par TenantSerializer.
//
// Les téléchargements sont des réponses fichier Django sur une autre origine :
// on utilise de vrais liens <a href> (navigation de premier niveau, les
// cookies de session suivent) — surtout pas un fetch, qui perdrait
// l'authentification côté navigateur pour un téléchargement de fichier.

import type React from 'react'
import { useCallback, useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import {
  AlertCircle,
  CheckCircle2,
  Clock,
  Download,
  Laptop,
  Loader2,
  Monitor,
  RotateCw,
  ShieldCheck,
  Terminal,
} from 'lucide-react'
import { downloadComposeUrl, downloadInstallerUrl, getOnboarding, normalizeApiError } from '../api/client'
import { useAuth } from '../context/useAuth'
import { PLAN_LABELS } from '../types/index'
import type { Onboarding } from '../types/index'

function LoadingState() {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: '10px', color: 'var(--text-secondary)', padding: '64px 0', justifyContent: 'center' }}>
      <Loader2 size={20} className="bvp-spin" />
      <span>Chargement de votre parcours d'installation…</span>
      <style>{`.bvp-spin { animation: bvp-spin 0.8s linear infinite; } @keyframes bvp-spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }`}</style>
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

function StepCard({ number, title, children }: { number: number; title: string; children: React.ReactNode }) {
  return (
    <div
      style={{
        background: '#FFFFFF',
        border: '1px solid var(--border)',
        borderRadius: '12px',
        padding: '28px 32px',
        marginBottom: '16px',
        display: 'flex',
        gap: '20px',
        alignItems: 'flex-start',
      }}
    >
      <div
        style={{
          width: '40px',
          height: '40px',
          minWidth: '40px',
          background: 'var(--gold)',
          color: '#fff',
          borderRadius: '50%',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontWeight: 800,
          fontSize: '1.1rem',
        }}
      >
        {number}
      </div>
      <div style={{ flex: 1 }}>
        <div style={{ fontWeight: 700, fontSize: '1.05rem', marginBottom: '8px' }}>{title}</div>
        {children}
      </div>
    </div>
  )
}

export default function BienvenuePage() {
  const { user } = useAuth()
  const [data, setData] = useState<Onboarding | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const onboarding = await getOnboarding()
      setData(onboarding)
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

  if (loading) return <LoadingState />
  if (error) return <ErrorState message={error} onRetry={() => void load()} />
  if (!data) return null

  const { tenant, active_license, active_devices } = data
  const registrationToken = user?.tenant?.registration_token ?? null

  const parcComplete = Boolean(active_license) && active_devices >= (active_license?.seats ?? Infinity)

  return (
    <div style={{ maxWidth: '760px', margin: '0 auto' }}>
      <div style={{ marginBottom: '28px' }}>
        <h1 style={{ fontSize: '1.6rem', fontWeight: 800, marginBottom: '6px', color: 'var(--text-primary)' }}>
          Bienvenue, <span style={{ color: 'var(--crimson)' }}>{tenant.name}</span> !
        </h1>
        <p style={{ color: 'var(--text-muted)', fontSize: '0.95rem' }}>
          Votre espace est prêt. Installez le logiciel en 3 étapes.
        </p>
      </div>

      {/* Statut licence */}
      {active_license ? (
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '10px',
            fontSize: '0.9rem',
            marginBottom: '24px',
            background: '#D1E7DD',
            color: '#0A5C36',
            border: '1px solid #A3D9B8',
            borderRadius: '8px',
            padding: '10px 14px',
          }}
        >
          <CheckCircle2 size={16} style={{ flexShrink: 0 }} />
          <span>
            Licence <strong>{PLAN_LABELS[active_license.plan] ?? active_license.plan}</strong> — {active_license.seats} poste{active_license.seats > 1 ? 's' : ''}
            {active_license.expires_at && (
              <> — expire le <strong>{new Date(active_license.expires_at).toLocaleDateString('fr-FR')}</strong></>
            )}
          </span>
        </div>
      ) : (
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '10px',
            fontSize: '0.9rem',
            marginBottom: '24px',
            background: '#FEE8E8',
            color: 'var(--crimson)',
            border: '1px solid var(--crimson)',
            borderRadius: '8px',
            padding: '10px 14px',
          }}
        >
          <AlertCircle size={16} style={{ flexShrink: 0 }} />
          <span>Aucune licence active. Contactez votre éditeur.</span>
        </div>
      )}

      {/* Étape 1 — Téléchargement */}
      <StepCard number={1} title="Télécharger l'installateur">
        <p style={{ color: 'var(--text-secondary)', fontSize: '0.88rem', marginBottom: '18px' }}>
          Choisissez votre système d'exploitation, ou récupérez directement le fichier{' '}
          <code style={{ background: 'rgba(0,0,0,.06)', padding: '1px 5px', borderRadius: '3px' }}>docker-compose.yml</code> de votre cluster.
        </p>

        <div style={{ display: 'flex', gap: '12px', flexWrap: 'wrap', marginBottom: '18px' }}>
          <a
            href={downloadInstallerUrl('windows')}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '10px',
              padding: '14px 22px',
              border: '2px solid var(--crimson)',
              borderRadius: '10px',
              background: 'var(--crimson)',
              color: '#fff',
              fontSize: '0.92rem',
              fontWeight: 600,
              textDecoration: 'none',
              minWidth: '190px',
            }}
          >
            <Monitor size={20} />
            <span>
              <span style={{ display: 'block', fontSize: '0.78rem', fontWeight: 400, opacity: 0.85, marginBottom: '1px' }}>
                Télécharger pour
              </span>
              Windows
            </span>
            <Download size={14} style={{ marginLeft: 'auto' }} />
          </a>

          <a
            href={downloadInstallerUrl('linux')}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '10px',
              padding: '14px 22px',
              border: '2px solid var(--border)',
              borderRadius: '10px',
              background: '#fff',
              color: 'var(--text-primary)',
              fontSize: '0.92rem',
              fontWeight: 600,
              textDecoration: 'none',
              minWidth: '190px',
            }}
          >
            <Terminal size={20} />
            <span>
              <span style={{ display: 'block', fontSize: '0.78rem', fontWeight: 400, opacity: 0.7, marginBottom: '1px' }}>
                Télécharger pour
              </span>
              Linux
            </span>
            <Download size={14} style={{ marginLeft: 'auto' }} />
          </a>

          <a
            href={downloadComposeUrl()}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '10px',
              padding: '14px 22px',
              border: '2px solid var(--border)',
              borderRadius: '10px',
              background: '#fff',
              color: 'var(--text-primary)',
              fontSize: '0.92rem',
              fontWeight: 600,
              textDecoration: 'none',
              minWidth: '190px',
            }}
          >
            <Laptop size={20} />
            <span>
              <span style={{ display: 'block', fontSize: '0.78rem', fontWeight: 400, opacity: 0.7, marginBottom: '1px' }}>
                Télécharger
              </span>
              docker-compose.yml
            </span>
            <Download size={14} style={{ marginLeft: 'auto' }} />
          </a>
        </div>

        <div style={{ background: '#F5EFE8', border: '1px solid var(--border)', borderRadius: '8px', padding: '14px 18px' }}>
          <p style={{ fontSize: '0.78rem', color: 'var(--text-muted)', marginBottom: '10px' }}>
            Ces identifiants sont intégrés dans l'installateur. Conservez-les comme référence.
          </p>
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px' }}>
            <div>
              <div style={{ fontSize: '0.72rem', fontWeight: 600, textTransform: 'uppercase', letterSpacing: '.06em', color: 'var(--text-muted)', marginBottom: '4px' }}>
                Token d'enregistrement
              </div>
              <div style={{ background: '#F5EFE8', padding: '10px 14px', borderRadius: '6px', fontFamily: 'monospace', fontSize: '0.78rem', wordBreak: 'break-all' }}>
                {registrationToken ?? 'Non disponible'}
              </div>
            </div>
            <div>
              <div style={{ fontSize: '0.72rem', fontWeight: 600, textTransform: 'uppercase', letterSpacing: '.06em', color: 'var(--text-muted)', marginBottom: '4px' }}>
                ID tenant
              </div>
              <div style={{ background: '#F5EFE8', padding: '10px 14px', borderRadius: '6px', fontFamily: 'monospace', fontSize: '0.78rem', wordBreak: 'break-all' }}>
                {tenant.id}
              </div>
            </div>
          </div>
        </div>
      </StepCard>

      {/* Étape 2 — Exécution */}
      <StepCard number={2} title="Exécuter l'installateur sur chaque PC">
        <p style={{ color: 'var(--text-secondary)', fontSize: '0.88rem', marginBottom: '14px' }}>
          L'installateur configure Docker si besoin, démarre le logiciel avec vos identifiants et enregistre la
          machine auprès du relais éditeur.
        </p>
        <div
          style={{
            background: '#EFF6FF',
            border: '1px solid #BFDBFE',
            borderRadius: '8px',
            padding: '14px 18px',
            fontSize: '0.875rem',
            color: '#1E40AF',
          }}
        >
          <strong>Windows :</strong> clic droit sur l'installateur téléchargé → <em>Exécuter en tant qu'administrateur</em>.
          <br />
          <strong>Linux :</strong>{' '}
          <code style={{ background: 'rgba(0,0,0,.08)', padding: '1px 5px', borderRadius: '3px' }}>
            chmod +x installer-pme.sh &amp;&amp; sudo ./installer-pme.sh
          </code>
        </div>
      </StepCard>

      {/* Étape 3 — Vérification */}
      <StepCard number={3} title="Vérification de votre parc">
        <p style={{ color: 'var(--text-secondary)', fontSize: '0.88rem', marginBottom: '16px' }}>
          Dès que chaque PC termine l'installation, il apparaît automatiquement dans votre espace.
        </p>

        <span
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            gap: '8px',
            padding: '8px 16px',
            borderRadius: '20px',
            fontWeight: 600,
            fontSize: '0.9rem',
            background: active_devices === 0 ? '#FFF3CD' : parcComplete ? 'var(--crimson)' : '#D1E7DD',
            color: active_devices === 0 ? '#856404' : parcComplete ? '#fff' : '#0A5C36',
          }}
        >
          {active_devices === 0 ? <Clock size={14} /> : <CheckCircle2 size={14} />}
          {active_devices === 0
            ? 'En attente — aucune machine connectée'
            : parcComplete
              ? `Parc complet — ${active_devices} machine${active_devices > 1 ? 's' : ''} connectée${active_devices > 1 ? 's' : ''}`
              : `${active_devices} machine${active_devices > 1 ? 's' : ''} connectée${active_devices > 1 ? 's' : ''}`}
        </span>

        {active_license && active_devices > 0 && (
          <p style={{ color: 'var(--text-muted)', fontSize: '0.8rem', marginTop: '10px' }}>
            {active_devices} / {active_license.seats} poste(s) utilisé(s)
          </p>
        )}

        <div style={{ marginTop: '16px', display: 'flex', gap: '10px', flexWrap: 'wrap' }}>
          <button
            onClick={() => void load()}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              padding: '9px 18px',
              borderRadius: 'var(--radius)',
              fontSize: '13px',
              fontWeight: 600,
              border: '1px solid var(--border)',
              background: '#fff',
              color: 'var(--text-primary)',
              cursor: 'pointer',
            }}
          >
            <RotateCw size={14} /> Rafraîchir
          </button>
          {active_devices >= 1 && (
            <Link
              to="/mon-espace"
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
                padding: '9px 18px',
                borderRadius: 'var(--radius)',
                fontSize: '13px',
                fontWeight: 600,
                border: 'none',
                background: 'var(--crimson)',
                color: '#fff',
                textDecoration: 'none',
              }}
            >
              <ShieldCheck size={14} /> Accéder à mon espace
            </Link>
          )}
        </div>
      </StepCard>
    </div>
  )
}
