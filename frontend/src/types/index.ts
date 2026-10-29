// Types miroir des serializers DRF (tenants/, licenses/, devices/, auth/, dashboard/,
// clusters/, relay/) — voir CONTRAT-API.md, section 4 pour les objets partagés.
// Toute modification ici doit correspondre à un champ réellement servi par
// l'API — sinon la console affiche des cases vides en production.

export interface Tenant {
  id: string
  name: string
  email: string
  phone: string
  address: string
  employee_count: number
  is_active: boolean
  created_at: string
  /** Jeton a communiquer a la PME pour l'enrolement de ses noeuds (lecture seule). */
  registration_token: string
}

export type LicensePlan = 'starter' | 'pro' | 'enterprise'

export interface License {
  id: string
  tenant: string
  /** Aplati par LicenseSerializer (source='tenant.name'), lecture seule. */
  tenant_name?: string
  plan: LicensePlan
  /** Nombre de postes autorisés par la licence. */
  seats: number
  starts_at: string
  expires_at: string | null
  is_active: boolean
  created_at: string
}

/** Rôle PostgreSQL du nœud dans le cluster PME (vide tant qu'il n'a pas annoncé). */
export type NodeRole = 'primary' | 'standby' | ''

export interface Device {
  id: string
  tenant: string
  /** Aplati par DeviceSerializer (source='tenant.name'), lecture seule. */
  tenant_name?: string
  /** Identité de licence : UUID d'installation authentifié, jamais la MAC. */
  installation_id: string
  mac_address: string
  hostname: string
  os: string
  node_role: NodeRole
  /** Serveur (PostgreSQL + nœud) ou témoin etcd seul (ADR-005). */
  node_kind: 'serveur' | 'temoin'
  /** IP:port annoncée par le nœud (ex : 192.168.200.130:9001). */
  node_addr: string
  /** IP:port de l'interface web PME (ex : 192.168.200.130:3000). */
  web_addr: string
  registered_at: string
  last_seen: string
  is_active: boolean
  is_within_license: boolean
}

/** Libellés d'affichage des plans de licence. */
export const PLAN_LABELS: Record<LicensePlan, string> = {
  starter: 'Starter',
  pro: 'Pro',
  enterprise: 'Enterprise',
}

/** Libellés d'affichage des rôles de nœud. */
export const NODE_ROLE_LABELS: Record<string, string> = {
  primary: 'Primaire (actif)',
  standby: 'Standby (passif)',
  '': 'Rôle inconnu',
}

/**
 * Un nœud est considéré en ligne s'il s'est annoncé récemment au SaaS.
 * L'API ne sert pas de booléen : la fraîcheur de last_seen fait foi.
 */
export const ONLINE_THRESHOLD_MS = 2 * 60 * 1000

export function isDeviceOnline(device: Pick<Device, 'last_seen' | 'is_active'>): boolean {
  if (!device.is_active || !device.last_seen) return false
  return Date.now() - new Date(device.last_seen).getTime() < ONLINE_THRESHOLD_MS
}

// ---------------------------------------------------------------------------
// Authentification — /api/auth/
// ---------------------------------------------------------------------------

/** Rôle applicatif dérivé de is_staff côté serveur : 'editor' si staff, sinon 'tenant'. */
export type UserRole = 'editor' | 'tenant'

/** Sous-ensemble du tenant rattaché à un compte non-staff, servi dans <User>. */
export interface UserTenant {
  id: string
  name: string
  registration_token: string
}

export interface User {
  id: number
  username: string
  email: string
  is_staff: boolean
  role: UserRole
  tenant: UserTenant | null
}

/** Corps de POST /api/auth/login/ */
export interface LoginPayload {
  username: string
  password: string
}

/** Corps de POST /api/auth/register/ — validé par tenants.forms.InscriptionForm. */
export interface RegisterPayload {
  name: string
  email: string
  phone: string
  address: string
  employee_count: number
  password: string
  password_confirm: string
}

// ---------------------------------------------------------------------------
// Espace éditeur — dashboard, clusters, relais
// ---------------------------------------------------------------------------

export interface DashboardCounters {
  total_tenants: number
  active_licenses: number
  total_devices: number
  tenants_over_limit: number
}

/** Réponse de GET /api/dashboard/stats/. */
export interface DashboardStats {
  stats: DashboardCounters
  recent_tenants: Tenant[]
}

/** Motif de santé d'un cluster PME — voir devices/services.build_cluster_overview. */
export type ClusterHealth =
  | 'ok'
  | 'empty'
  | 'no_primary'
  | 'primary_offline'
  | 'no_standby'
  | 'over_license'

export interface ClusterNodeStatus {
  device: Device
  online: boolean
}

export interface ClusterLicenseStatus {
  ok: boolean
  reason: string | null
  used: number
  total: number
  plan: string
}

export interface ClusterEntry {
  tenant: Pick<Tenant, 'id' | 'name' | 'email'>
  primary: ClusterNodeStatus | null
  standbys: ClusterNodeStatus[]
  total: number
  online_count: number
  license: ClusterLicenseStatus
  health: ClusterHealth
  is_alert: boolean
}

export interface ClusterOverview {
  alerts_total: number
  total_tenants: number
  clusters: ClusterEntry[]
}

export interface RelayHealth {
  reachable: boolean
  status: string
  version: string
  hostname: string
  uptime_secs: number
  uptime_fmt: string
  port: string
  blobs_dir: string
  blob_tenants: number
  networks: string[]
  zero_knowledge: boolean
  error?: string
}

export interface RelayNode {
  node_id: string
  short_id: string
  last_seen: string
  time_ago: string
  [key: string]: unknown
}

export interface RelayTenantNodes {
  tenant: Pick<Tenant, 'id' | 'name'>
  nodes: RelayNode[]
}

export interface RelayStatus {
  relay_url: string
  health: RelayHealth
  total_nodes: number
  tenants: RelayTenantNodes[]
  editeur_subnet: string
  relay_subnet: string
  pme_subnet: string
}

export interface DeviceToggleResult {
  id: string
  is_active: boolean
}

// ---------------------------------------------------------------------------
// Espace PME — /api/tenants/me/
// ---------------------------------------------------------------------------

/** Accès pgAdmin. Le mot de passe n'est PAS connu du portail : il est généré
 *  sur le réseau de la PME et affiché en fin d'installation (ADR-005). */
export interface PgAdminAcces {
  url: string
  email: string
}

export interface EspacePme {
  tenant: Tenant
  active_license: License | null
  devices: Device[]
  active_devices: number
  pgadmin: PgAdminAcces
  /** Deux serveurs et aucun témoin : la panne de l'un bloquerait les écritures. */
  besoin_temoin: boolean
  relay_url: string
}

export interface OnboardingDownloads {
  compose: string
  installer_linux: string
  installer_windows: string
  installer_temoin_linux: string
  installer_temoin_windows: string
}

export interface Onboarding {
  tenant: Tenant
  active_license: License | null
  active_devices: number
  relay_url: string
  downloads: OnboardingDownloads
}

// ---------------------------------------------------------------------------
// Erreurs
// ---------------------------------------------------------------------------

/** Erreur de validation DRF renvoyée champ par champ : {"champ": ["message", ...]}. */
export type FieldErrors = Record<string, string[]>

export interface ApiError {
  /** Message global prêt à afficher (ex: dans un bandeau d'erreur). */
  message: string
  /** Erreurs par champ, si le backend en a renvoyé (formulaires DRF). */
  fieldErrors: FieldErrors | null
  status: number | null
}

// ---------------------------------------------------------------------------
// Déploiement — image du nœud PME et registre Docker local
// ---------------------------------------------------------------------------

/** État du registre Docker local qui distribue l'image aux machines PME. */
export interface NodeImageRegistry {
  /** Hôte:port du registre, dérivé de la requête (jamais codé en dur). */
  hote: string
  /** Commande `docker pull` prête à copier, ou null si aucune image publiée. */
  commande_pull: string | null
  /** Tags publiés sur le registre, lus via son API v2. */
  tags: string[]
  /** false si le registre ne répond pas — état normal, pas une erreur. */
  joignable: boolean
}

/** Réponse de GET /api/node-image/. `disponible: false` est un état normal
 * (aucune image encore publiée) : tous les champs inconnus sont alors null. */
export interface NodeImage {
  disponible: boolean
  version: string | null
  image: string | null
  fichier: string | null
  taille_octets: number | null
  sha256: string | null
  construit_le: string | null
  url_telechargement: string
  registre: NodeImageRegistry
}
