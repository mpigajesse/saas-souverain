# ============================================================
#  EL BARAA CONSULT — Nettoyage espace Docker
#  Usage : clic droit -> "Executer avec PowerShell"
#          ou : powershell -ExecutionPolicy Bypass -File "E:\SaaS souverain\docker-cleanup.ps1"
# ============================================================

Write-Host ""
Write-Host "====================================================" -ForegroundColor Cyan
Write-Host "  Nettoyage espace Docker" -ForegroundColor Cyan
Write-Host "====================================================" -ForegroundColor Cyan
Write-Host ""

# Verifier que Docker tourne
$dockerRunning = docker info --format "{{.ServerVersion}}" 2>$null
if (-not $dockerRunning) {
    Write-Host "ERREUR : Docker n'est pas demarre. Lance Docker Desktop d'abord." -ForegroundColor Red
    Read-Host "Appuie sur Entree pour fermer"
    exit 1
}

# Avant
Write-Host "--- Espace utilise AVANT ---" -ForegroundColor Yellow
docker system df
Write-Host ""

# Nettoyage conteneurs arretes, reseaux, volumes orphelins, images pendantes
Write-Host "--- Nettoyage systeme... ---" -ForegroundColor Yellow
docker system prune --volumes -f
Write-Host ""

# Nettoyage cache de build (couches intermediaires Rust/Docker)
Write-Host "--- Nettoyage cache de build... ---" -ForegroundColor Yellow
docker builder prune -f
Write-Host ""

# Apres
Write-Host "--- Espace utilise APRES ---" -ForegroundColor Green
docker system df
Write-Host ""
Write-Host "====================================================" -ForegroundColor Green
Write-Host "  Nettoyage termine !" -ForegroundColor Green
Write-Host "====================================================" -ForegroundColor Green
Write-Host ""
Read-Host "Appuie sur Entree pour fermer"
