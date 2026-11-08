// Pastilles des états V3 annoncés par un nœud : témoin etcd (ADR-005), santé
// de l'application hébergée (étape 3), chiffrement du disque (ADR-008),
// récupération de la DEK (ADR-010).
// Rien n'est affiché pour un état inconnu ou non annoncé.

import type React from 'react'
import type { Device } from '../types/index'

const pastille: React.CSSProperties = {
  fontSize: '11px',
  fontWeight: 700,
  padding: '2px 8px',
  borderRadius: '10px',
  whiteSpace: 'nowrap',
}

export default function EtatsNoeud({
  device,
}: {
  device: Pick<Device, 'node_kind' | 'etat_application' | 'chiffrement_disque'> & Partial<Pick<Device, 'recuperation'>>
}) {
  const elements: React.ReactNode[] = []
  if (device.node_kind === 'temoin') {
    elements.push(
      <span key="temoin" style={{ ...pastille, background: '#EDE9FE', color: '#5B21B6' }} title="Témoin etcd : vote pour les bascules, ne stocke aucune donnée">
        Témoin
      </span>,
    )
  }
  if (device.etat_application === 'ok') {
    elements.push(
      <span key="app" style={{ ...pastille, background: '#D1FAE5', color: '#065F46' }} title="L'application hébergée répond à sa sonde de santé">
        Application disponible
      </span>,
    )
  } else if (device.etat_application === 'ko') {
    elements.push(
      <span key="app" style={{ ...pastille, background: '#FEE2E2', color: '#991B1B' }} title="L'application hébergée ne répond pas à sa sonde de santé">
        Application indisponible
      </span>,
    )
  }
  if (device.chiffrement_disque === 'non') {
    elements.push(
      <span key="disque" style={{ ...pastille, background: '#FEF3C7', color: '#92400E' }} title="Les données sont stockées en clair au repos sur ce disque : activez BitLocker ou LUKS">
        Disque non chiffré
      </span>,
    )
  } else if (device.chiffrement_disque === 'oui') {
    elements.push(
      <span key="disque" style={{ ...pastille, background: '#E0F2FE', color: '#075985' }} title="Disque chiffré (BitLocker ou LUKS)">
        Disque chiffré
      </span>,
    )
  }
  if (device.recuperation === 'non_configuree' || device.recuperation === 'reportee') {
    elements.push(
      <span key="recuperation" style={{ ...pastille, background: '#FEE2E2', color: '#991B1B' }} title="Récupération de la DEK non configurée : si tous les serveurs étaient perdus, les sauvegardes seraient illisibles">
        Récupération {device.recuperation === 'reportee' ? 'reportée' : 'non configurée'}
      </span>,
    )
  } else if (device.recuperation === 'dek_differente') {
    elements.push(
      <span key="recuperation" style={{ ...pastille, background: '#FEE2E2', color: '#991B1B' }} title="Cluster réinstallé : le relais protège une autre DEK, sauvegardes suspendues jusqu'à ss-node recuperer">
        Récupération en attente
      </span>,
    )
  } else if (device.recuperation === 'configuree') {
    elements.push(
      <span key="recuperation" style={{ ...pastille, background: '#D1FAE5', color: '#065F46' }} title="Récupération de la DEK configurée (passphrase et parts papier)">
        Récupération configurée
      </span>,
    )
  }
  if (elements.length === 0) return null
  return <div style={{ display: 'flex', gap: '6px', flexWrap: 'wrap', marginTop: '4px' }}>{elements}</div>
}
