// Contrat du contexte d'authentification + instance du contexte.
// Isole du composant AuthProvider : un module qui exporte un composant ne doit
// rien exporter d'autre (react-refresh/only-export-components).

import { createContext } from 'react'
import type { LoginPayload, RegisterPayload, User, UserRole } from '../types/index'

export interface AuthContextType {
  /** Utilisateur connecté, ou null tant que non authentifié. */
  user: User | null
  /** Raccourci vers user.role — null tant que la session n'est pas connue. */
  role: UserRole | null
  /** true pendant la vérification de session au montage. */
  loading: boolean
  /** Dernier message d'erreur d'authentification (login/register), le cas échéant. */
  error: string | null
  isAuthenticated: boolean
  login: (payload: LoginPayload) => Promise<User>
  register: (payload: RegisterPayload) => Promise<User>
  logout: () => Promise<void>
  clearError: () => void
}

export const AuthContext = createContext<AuthContextType | undefined>(undefined)
