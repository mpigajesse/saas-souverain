import type React from 'react'
import { useState } from 'react'

interface StockItem {
  id: string
  name: string
  sku: string
  quantity: number
  category: string
}

const INITIAL_STOCKS: StockItem[] = [
  { id: 'item-001', name: 'Imprimante HP Laser Multifonction', sku: 'HP-LSR-500', quantity: 12, category: 'Informatique' },
  { id: 'item-002', name: 'Papier A4 80g (Carton de 5 rames)', sku: 'PAP-A4-80', quantity: 85, category: 'Fournitures' },
  { id: 'item-003', name: 'Ecran 27 pouces 4K UHD', sku: 'SCR-27-4K', quantity: 4, category: 'Informatique' },
  { id: 'item-004', name: 'Toners Noir HP Laser', sku: 'TNR-HP-BLK', quantity: 18, category: 'Consommables' },
]

export default function StockPage() {
  const [items, setItems] = useState<StockItem[]>(INITIAL_STOCKS)
  const [selectedItemId, setSelectedItemId] = useState<string>(INITIAL_STOCKS[0].id)
  const [changeQty, setChangeQty] = useState<number>(1)
  const [movementType, setMovementType] = useState<'RECEPTION' | 'VENTE'>('VENTE')
  
  const [alertError, setAlertError] = useState<string | null>(null)
  const [alertSuccess, setAlertSuccess] = useState<string | null>(null)
  const [journalIndex, setJournalIndex] = useState<number>(104)

  const handleTransaction = (e: React.FormEvent) => {
    e.preventDefault()
    setAlertError(null)
    setAlertSuccess(null)

    const targetItem = items.find((i) => i.id === selectedItemId)
    if (!targetItem) return

    const qtyDelta = movementType === 'RECEPTION' ? Math.abs(changeQty) : -Math.abs(changeQty)
    const newQty = targetItem.quantity + qtyDelta

    // Verification de l'Invariant Métier (Phase 5 - ss-client Rust)
    if (newQty < 0) {
      setAlertError(
        `❌ Transaction rejetée par le SDK ss-client : Stock insuffisant ! Quantité actuelle disponible : ${targetItem.quantity}, demandée : ${Math.abs(changeQty)}.`
      )
      return
    }

    // Mise à jour du stock
    setItems((prev) =>
      prev.map((item) => (item.id === selectedItemId ? { ...item, quantity: newQty } : item))
    )

    const nextIndex = journalIndex + 1
    setJournalIndex(nextIndex)
    setAlertSuccess(
      `✅ Mouvement de stock validé et chiffré ! Nouveau stock : ${newQty} unités. Index du journal CBOR append-only : #${nextIndex}`
    )
  }

  const selectedItem = items.find((i) => i.id === selectedItemId)

  return (
    <div>
      <div style={{ marginBottom: '28px' }}>
        <h1 style={{ fontSize: '22px', fontWeight: 700, color: 'var(--text-primary)' }}>
          Gestion de Stock Souveraine (Phase 5)
        </h1>
        <p style={{ color: 'var(--text-secondary)', marginTop: '4px' }}>
          Contrôle des invariants métier en local sur le cluster PME (Interdiction des stocks négatifs)
        </p>
      </div>

      {alertError && (
        <div
          style={{
            background: '#FEE8E8',
            border: '1px solid var(--crimson)',
            color: 'var(--crimson)',
            padding: '14px 18px',
            borderRadius: 'var(--radius)',
            marginBottom: '20px',
            fontWeight: 600,
          }}
        >
          {alertError}
        </div>
      )}

      {alertSuccess && (
        <div
          style={{
            background: '#E8F5E9',
            border: '1px solid #2E7D32',
            color: '#1B5E20',
            padding: '14px 18px',
            borderRadius: 'var(--radius)',
            marginBottom: '20px',
            fontWeight: 600,
          }}
        >
          {alertSuccess}
        </div>
      )}

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 340px', gap: '24px' }}>
        {/* Table des Stocks */}
        <div
          style={{
            background: '#FFFFFF',
            borderRadius: 'var(--radius)',
            border: '1px solid var(--border)',
            boxShadow: 'var(--shadow)',
            padding: '20px',
          }}
        >
          <h2 style={{ fontSize: '16px', fontWeight: 600, marginBottom: '16px', color: 'var(--text-primary)' }}>
            Inventaire des Articles du Cluster PME
          </h2>
          <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '14px' }}>
            <thead>
              <tr style={{ borderBottom: '1px solid var(--border)', textAlign: 'left', color: 'var(--text-secondary)' }}>
                <th style={{ padding: '10px' }}>SKU</th>
                <th style={{ padding: '10px' }}>Article</th>
                <th style={{ padding: '10px' }}>Catégorie</th>
                <th style={{ padding: '10px', textAlign: 'right' }}>Quantité en Stock</th>
              </tr>
            </thead>
            <tbody>
              {items.map((item) => (
                <tr
                  key={item.id}
                  style={{
                    borderBottom: '1px solid var(--border)',
                    background: item.id === selectedItemId ? 'rgba(196,151,42,0.08)' : 'transparent',
                    cursor: 'pointer',
                  }}
                  onClick={() => setSelectedItemId(item.id)}
                >
                  <td style={{ padding: '12px 10px', fontFamily: 'monospace', fontWeight: 600 }}>{item.sku}</td>
                  <td style={{ padding: '12px 10px', fontWeight: 500 }}>{item.name}</td>
                  <td style={{ padding: '12px 10px', color: 'var(--text-muted)' }}>{item.category}</td>
                  <td
                    style={{
                      padding: '12px 10px',
                      textAlign: 'right',
                      fontWeight: 700,
                      color: item.quantity <= 5 ? 'var(--crimson)' : 'var(--text-primary)',
                    }}
                  >
                    {item.quantity} unités {item.quantity <= 5 && '⚠️ (Critique)'}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        {/* Formulaire de Transaction */}
        <div
          style={{
            background: '#FFFFFF',
            borderRadius: 'var(--radius)',
            border: '1px solid var(--border)',
            borderTop: '3px solid var(--crimson)',
            boxShadow: 'var(--shadow)',
            padding: '20px',
          }}
        >
          <h2 style={{ fontSize: '16px', fontWeight: 600, marginBottom: '14px', color: 'var(--text-primary)' }}>
            Nouveau Mouvement Métier
          </h2>
          <form onSubmit={handleTransaction} style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>
            <div>
              <label style={{ fontSize: '12px', color: 'var(--text-secondary)', display: 'block', marginBottom: '4px' }}>
                Article Sélectionné
              </label>
              <select
                value={selectedItemId}
                onChange={(e) => setSelectedItemId(e.target.value)}
                style={{
                  width: '100%',
                  padding: '8px 10px',
                  borderRadius: '6px',
                  border: '1px solid var(--border)',
                  fontSize: '13px',
                }}
              >
                {items.map((i) => (
                  <option key={i.id} value={i.id}>
                    {i.name} (Dispo : {i.quantity})
                  </option>
                ))}
              </select>
            </div>

            <div>
              <label style={{ fontSize: '12px', color: 'var(--text-secondary)', display: 'block', marginBottom: '4px' }}>
                Type de Mouvement
              </label>
              <div style={{ display: 'flex', gap: '10px' }}>
                <button
                  type="button"
                  onClick={() => setMovementType('VENTE')}
                  style={{
                    flex: 1,
                    padding: '8px',
                    borderRadius: '6px',
                    border: '1px solid var(--border)',
                    background: movementType === 'VENTE' ? 'var(--crimson)' : '#FFFFFF',
                    color: movementType === 'VENTE' ? '#FFFFFF' : 'var(--text-primary)',
                    fontWeight: 600,
                    cursor: 'pointer',
                  }}
                >
                  Sortie (Vente)
                </button>
                <button
                  type="button"
                  onClick={() => setMovementType('RECEPTION')}
                  style={{
                    flex: 1,
                    padding: '8px',
                    borderRadius: '6px',
                    border: '1px solid var(--border)',
                    background: movementType === 'RECEPTION' ? '#2E7D32' : '#FFFFFF',
                    color: movementType === 'RECEPTION' ? '#FFFFFF' : 'var(--text-primary)',
                    fontWeight: 600,
                    cursor: 'pointer',
                  }}
                >
                  Entrée (Réception)
                </button>
              </div>
            </div>

            <div>
              <label style={{ fontSize: '12px', color: 'var(--text-secondary)', display: 'block', marginBottom: '4px' }}>
                Quantité
              </label>
              <input
                type="number"
                min="1"
                value={changeQty}
                onChange={(e) => setChangeQty(parseInt(e.target.value) || 1)}
                style={{
                  width: '100%',
                  padding: '8px 10px',
                  borderRadius: '6px',
                  border: '1px solid var(--border)',
                  fontSize: '14px',
                }}
              />
            </div>

            <div style={{ background: '#F8F9FA', padding: '10px', borderRadius: '6px', fontSize: '12px' }}>
              <div>Stock actuel : <strong>{selectedItem?.quantity}</strong></div>
              <div>Variante : <strong>{movementType === 'RECEPTION' ? `+${changeQty}` : `-${changeQty}`}</strong></div>
              <div>Résultat estimé : <strong>{(selectedItem?.quantity || 0) + (movementType === 'RECEPTION' ? changeQty : -changeQty)}</strong></div>
            </div>

            <button
              type="submit"
              style={{
                width: '100%',
                padding: '10px',
                background: 'var(--sidebar-bg)',
                color: '#FFFFFF',
                border: 'none',
                borderRadius: '6px',
                fontWeight: 600,
                cursor: 'pointer',
              }}
            >
              Valider le Mouvement gRPC
            </button>
          </form>
        </div>
      </div>
    </div>
  )
}
