# ============================================================
#  EL BARAA CONSULT — Build image ss-node
#  A lancer depuis : E:\SaaS souverain\spike\
#  Usage : powershell -ExecutionPolicy Bypass -File build-node.ps1
#          ou : powershell -ExecutionPolicy Bypass -File build-node.ps1 --no-cache
# ============================================================

param(
    [switch]$NoCache
)

$REGISTRY = "192.168.200.1:5000"
$IMAGE    = "$REGISTRY/ss-node:dev"

Write-Host ""
Write-Host "====================================================" -ForegroundColor Cyan
Write-Host "  Build image : $IMAGE" -ForegroundColor Cyan
Write-Host "====================================================" -ForegroundColor Cyan
Write-Host ""

# Verifier que Docker tourne
$dockerRunning = docker info --format "{{.ServerVersion}}" 2>$null
if (-not $dockerRunning) {
    Write-Host "ERREUR : Docker n'est pas demarre." -ForegroundColor Red
    exit 1
}

# Build depuis la racine du workspace spike/ (requis pour Cargo workspace)
$args = @("build", "-f", "node/Dockerfile", "-t", $IMAGE)
if ($NoCache) {
    $args += "--no-cache"
    Write-Host "Mode : --no-cache (reconstruction complete)" -ForegroundColor Yellow
}
$args += "."

Write-Host "Commande : docker $($args -join ' ')" -ForegroundColor Gray
Write-Host ""

docker @args
if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "ERREUR : Build echoue (code $LASTEXITCODE)" -ForegroundColor Red
    exit $LASTEXITCODE
}

Write-Host ""
Write-Host "--- Push vers le registre local... ---" -ForegroundColor Yellow
docker push $IMAGE
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERREUR : Push echoue" -ForegroundColor Red
    exit $LASTEXITCODE
}

Write-Host ""
Write-Host "====================================================" -ForegroundColor Green
Write-Host "  Image publiee : $IMAGE" -ForegroundColor Green
Write-Host "  Sur les VMs PME : docker compose pull && docker compose up -d" -ForegroundColor Green
Write-Host "====================================================" -ForegroundColor Green
Write-Host ""
