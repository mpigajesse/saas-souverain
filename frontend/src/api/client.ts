// Client HTTP unique de la console — voir CONTRAT-API.md (source de vérité).
//
// Aucune donnée simulée ici : chaque fonction relaie fidèlement la réponse (ou
// l'erreur) du backend Django/DRF. Une panne réseau ou une erreur serveur doit
// remonter à l'appelant sous forme d'exception — jamais d'un repli silencieux
// vers des données inventées.

import axios, { AxiosError } from 'axios'
import type {
  ApiError,
  Application,
  ClusterOverview,
  DashboardStats,
  Device,
  DeviceToggleResult,
  EspacePme,
  FieldErrors,
  License,
  LoginPayload,
  NodeImage,
  Onboarding,
  RegisterPayload,
  RelayStatus,
  Tenant,
  User,
} from '../types/index'

// ---------------------------------------------------------------------------
// Instance axios
// ---------------------------------------------------------------------------

// Chemin RELATIF par défaut, et c'est essentiel : le bundle Vite est figé au
// build, donc une URL absolue « http://localhost:8000/api » compilée en dur
// désignerait le localhost *du visiteur*. Depuis une autre machine du réseau,
// la console se connecterait à elle-même et tomberait en erreur réseau.
// En relatif, l'appel repart vers l'hôte qui a servi la page, quelle que soit
// l'IP tapée dans le navigateur ; nginx relaie ensuite /api vers Django
// (voir frontend/nginx.conf). VITE_API_URL reste disponible pour pointer un
// backend distinct au besoin.
const API_BASE_URL: string =
  (import.meta.env.VITE_API_URL as string | undefined) || '/api'

export const http = axios.create({
  baseURL: API_BASE_URL,
  withCredentials: true,
  xsrfCookieName: 'csrftoken',
  xsrfHeaderName: 'X-CSRFToken',
  headers: {
    'Content-Type': 'application/json',
  },
})

/**
 * Origine du backend (sans le suffixe /api), pour construire les URL de
 * téléchargement ouvertes directement par le navigateur.
 *
 * Avec la base relative « /api », retourne la chaîne vide : les liens restent
 * relatifs et pointent donc vers l'hôte affiché dans la barre d'adresse — ce
 * qui est exactement le comportement voulu depuis une machine du réseau.
 */
function getApiOrigin(): string {
  return API_BASE_URL.replace(/\/api\/?$/, '')
}

// ---------------------------------------------------------------------------
// CSRF — Django exige le cookie csrftoken posé avant toute écriture.
// ---------------------------------------------------------------------------

function hasCsrfCookie(): boolean {
  return /(?:^|;\s*)csrftoken=/.test(document.cookie)
}

let csrfPromise: Promise<void> | null = null

/**
 * Garantit la présence du cookie csrftoken en appelant GET /api/auth/csrf/
 * si besoin. Idempotent et sûr à appeler plusieurs fois en parallèle.
 */
export async function ensureCsrf(): Promise<void> {
  if (hasCsrfCookie()) return
  if (!csrfPromise) {
    csrfPromise = http
      .get<{ detail: string }>('/auth/csrf/')
      .then(() => undefined)
      .catch((err) => {
        csrfPromise = null
        throw err
      })
  }
  return csrfPromise
}

// Filet de sécurité : toute écriture passe par ensureCsrf(), même si l'appelant
// a oublié de le faire explicitement.
http.interceptors.request.use(async (config) => {
  const method = (config.method || 'get').toLowerCase()
  if (method !== 'get' && method !== 'head' && method !== 'options') {
    await ensureCsrf()
  }
  return config
})

// ---------------------------------------------------------------------------
// Normalisation des erreurs — toute l'UI passe par là pour afficher un message.
// ---------------------------------------------------------------------------

function isStringArray(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((item) => typeof item === 'string')
}

/**
 * Transforme une erreur axios (ou autre) en message français exploitable par
 * l'UI, en conservant les erreurs de validation champ par champ renvoyées par
 * DRF (ex : {"email": ["Une entreprise avec cet email existe déjà."]}).
 */
export function normalizeApiError(err: unknown): ApiError {
  if (axios.isAxiosError(err)) {
    const axiosErr = err as AxiosError<unknown>
    const status = axiosErr.response?.status ?? null

    if (!axiosErr.response) {
      return {
        message: "Impossible de contacter le serveur. Vérifiez votre connexion réseau et réessayez.",
        fieldErrors: null,
        status: null,
      }
    }

    const data = axiosErr.response.data

    if (data && typeof data === 'object') {
      const obj = data as Record<string, unknown>

      if (typeof obj.detail === 'string') {
        return { message: obj.detail, fieldErrors: null, status }
      }

      const fieldErrors: FieldErrors = {}
      for (const [key, value] of Object.entries(obj)) {
        if (isStringArray(value)) {
          fieldErrors[key] = value
        }
      }

      if (Object.keys(fieldErrors).length > 0) {
        const preferred = fieldErrors.non_field_errors?.[0]
        const firstAny = Object.values(fieldErrors)[0]?.[0]
        return {
          message: preferred ?? firstAny ?? 'Le formulaire contient des erreurs.',
          fieldErrors,
          status,
        }
      }
    }

    return {
      message: `Erreur inattendue du serveur (${status ?? '—'}).`,
      fieldErrors: null,
      status,
    }
  }

  if (err instanceof Error) {
    return { message: err.message, fieldErrors: null, status: null }
  }

  return { message: 'Erreur inconnue.', fieldErrors: null, status: null }
}

// ---------------------------------------------------------------------------
// 1. Authentification — /api/auth/
// ---------------------------------------------------------------------------

export async function login(payload: LoginPayload): Promise<User> {
  const { data } = await http.post<User>('/auth/login/', payload)
  return data
}

export async function logout(): Promise<void> {
  await http.post('/auth/logout/')
}

/** GET /api/auth/me/ — lève une erreur (401) si personne n'est connecté. */
export async function getCurrentUser(): Promise<User> {
  const { data } = await http.get<User>('/auth/me/')
  return data
}

export async function register(payload: RegisterPayload): Promise<User> {
  const { data } = await http.post<User>('/auth/register/', payload)
  return data
}

// ---------------------------------------------------------------------------
// 2. Espace éditeur — dashboard, clusters, relais
// ---------------------------------------------------------------------------

export async function getDashboardStats(): Promise<DashboardStats> {
  const { data } = await http.get<DashboardStats>('/dashboard/stats/')
  return data
}

export async function getClusterOverview(): Promise<ClusterOverview> {
  const { data } = await http.get<ClusterOverview>('/clusters/')
  return data
}

export async function getRelayStatus(): Promise<RelayStatus> {
  const { data } = await http.get<RelayStatus>('/relay/status/')
  return data
}

export async function toggleDeviceActive(id: string): Promise<DeviceToggleResult> {
  const { data } = await http.post<DeviceToggleResult>(`/devices/${id}/toggle/`)
  return data
}

/** GET /api/node-image/ — état de l'image du nœud PME publiée par l'éditeur. */
export async function getNodeImage(): Promise<NodeImage> {
  const { data } = await http.get<NodeImage>('/node-image/')
  return data
}

/** URL absolue de téléchargement de l'archive `.tar.gz` de l'image du nœud PME. */
export function downloadNodeImageUrl(): string {
  return `${getApiOrigin()}/api/node-image/download/`
}

// ---------------------------------------------------------------------------
// CRUD — /api/tenants/, /api/licenses/, /api/devices/
// ---------------------------------------------------------------------------

export type TenantWritePayload = Partial<
  Pick<Tenant, 'name' | 'email' | 'phone' | 'address' | 'employee_count' | 'is_active'>
>

export async function getTenants(): Promise<Tenant[]> {
  const { data } = await http.get<Tenant[]>('/tenants/')
  return data
}

export async function getTenant(id: string): Promise<Tenant> {
  const { data } = await http.get<Tenant>(`/tenants/${id}/`)
  return data
}

export async function createTenant(payload: TenantWritePayload): Promise<Tenant> {
  const { data } = await http.post<Tenant>('/tenants/', payload)
  return data
}

export async function updateTenant(id: string, payload: TenantWritePayload): Promise<Tenant> {
  const { data } = await http.patch<Tenant>(`/tenants/${id}/`, payload)
  return data
}

export async function deleteTenant(id: string): Promise<void> {
  await http.delete(`/tenants/${id}/`)
}

export type LicenseWritePayload = Partial<
  Pick<License, 'tenant' | 'plan' | 'seats' | 'starts_at' | 'expires_at' | 'is_active' | 'application'>
>

export async function getLicenses(tenantId?: string): Promise<License[]> {
  const url = tenantId ? `/licenses/?tenant=${encodeURIComponent(tenantId)}` : '/licenses/'
  const { data } = await http.get<License[]>(url)
  return data
}

export async function getLicense(id: string): Promise<License> {
  const { data } = await http.get<License>(`/licenses/${id}/`)
  return data
}

export async function createLicense(payload: LicenseWritePayload): Promise<License> {
  const { data } = await http.post<License>('/licenses/', payload)
  return data
}

export async function updateLicense(id: string, payload: LicenseWritePayload): Promise<License> {
  const { data } = await http.patch<License>(`/licenses/${id}/`, payload)
  return data
}

export async function deleteLicense(id: string): Promise<void> {
  await http.delete(`/licenses/${id}/`)
}

export type DeviceWritePayload = Partial<
  Pick<Device, 'hostname' | 'os' | 'node_role' | 'node_addr' | 'web_addr' | 'is_active'>
>

export async function getDevices(tenantId?: string): Promise<Device[]> {
  const url = tenantId ? `/devices/?tenant=${encodeURIComponent(tenantId)}` : '/devices/'
  const { data } = await http.get<Device[]>(url)
  return data
}

export async function getDevice(id: string): Promise<Device> {
  const { data } = await http.get<Device>(`/devices/${id}/`)
  return data
}

export async function updateDevice(id: string, payload: DeviceWritePayload): Promise<Device> {
  const { data } = await http.patch<Device>(`/devices/${id}/`, payload)
  return data
}

export async function deleteDevice(id: string): Promise<void> {
  await http.delete(`/devices/${id}/`)
}

// ---------------------------------------------------------------------------
// 3. Espace PME — /api/tenants/me/
// ---------------------------------------------------------------------------

export async function getEspacePme(): Promise<EspacePme> {
  const { data } = await http.get<EspacePme>('/tenants/me/')
  return data
}

export async function getOnboarding(): Promise<Onboarding> {
  const { data } = await http.get<Onboarding>('/tenants/me/onboarding/')
  return data
}

/** URL absolue de téléchargement du docker-compose.yml généré pour le tenant courant. */
export function downloadComposeUrl(): string {
  return `${getApiOrigin()}/api/tenants/me/download/compose/`
}

/** URL absolue de téléchargement du script d'installation (linux|windows),
 *  pour un serveur (défaut) ou un témoin etcd. */
export function downloadInstallerUrl(os: 'linux' | 'windows', type: 'serveur' | 'temoin' = 'serveur'): string {
  const suffixe = type === 'temoin' ? '?type=temoin' : ''
  return `${getApiOrigin()}/api/tenants/me/download/installer/${os}/${suffixe}`
}

// ---------------------------------------------------------------------------
// Applications hébergées — /api/applications/ (staff, ADR-009)
// ---------------------------------------------------------------------------

export type ApplicationWritePayload = Partial<Pick<Application, 'nom' | 'editeur' | 'version'>> & {
  /** Objet JSON libre : le serveur le valide et renvoie sa forme normalisée. */
  manifeste?: unknown
}

export async function getApplications(): Promise<Application[]> {
  const { data } = await http.get<Application[]>('/applications/')
  return data
}

export async function createApplication(payload: ApplicationWritePayload): Promise<Application> {
  const { data } = await http.post<Application>('/applications/', payload)
  return data
}

export async function updateApplication(id: string, payload: ApplicationWritePayload): Promise<Application> {
  const { data } = await http.patch<Application>(`/applications/${id}/`, payload)
  return data
}

export async function deleteApplication(id: string): Promise<void> {
  await http.delete(`/applications/${id}/`)
}

/** Brouillon → validée, si l'image est publiée dans le registre (409 / 503 sinon). */
export async function validerApplication(id: string): Promise<Application> {
  const { data } = await http.post<Application>(`/applications/${id}/valider/`)
  return data
}

export async function retirerApplication(id: string): Promise<Application> {
  const { data } = await http.post<Application>(`/applications/${id}/retirer/`)
  return data
}
