import type React from 'react'

interface StatusBadgeProps {
  active: boolean
  label?: string
}

export default function StatusBadge({ active, label }: StatusBadgeProps) {
  const activeColor = '#2A7A3B'
  const inactiveColor = '#9E8E80'

  const color = active ? activeColor : inactiveColor

  const containerStyle: React.CSSProperties = {
    display: 'inline-flex',
    alignItems: 'center',
    gap: '5px',
    padding: '3px 10px',
    borderRadius: '20px',
    fontSize: '12px',
    fontWeight: 500,
    backgroundColor: active ? '#E8F5EB' : '#F0ECE8',
    color,
  }

  const dotStyle: React.CSSProperties = {
    width: '6px',
    height: '6px',
    borderRadius: '50%',
    backgroundColor: color,
    flexShrink: 0,
  }

  const displayLabel = label ?? (active ? 'Actif' : 'Inactif')

  return (
    <span style={containerStyle}>
      <span style={dotStyle} />
      {displayLabel}
    </span>
  )
}
