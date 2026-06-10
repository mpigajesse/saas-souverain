# Build et push ss-relay vers le registre local
# Usage : .\build-relay.ps1 [-NoCache]
param([switch]$NoCache)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot   # E:\SaaS souverain
$SpikeDir = Join-Path $Root "spike"
$Tag = "localhost:5000/ss-relay:dev"

Write-Host ""
Write-Host "========================================================"
Write-Host "  EL BARAA CONSULT — Build ss-relay"
Write-Host "  Contexte : $SpikeDir"
Write-Host "========================================================"
Write-Host ""

$extraArgs = if ($NoCache) { @("--no-cache") } else { @() }

docker build @extraArgs `
    -f (Join-Path $SpikeDir "relay\Dockerfile") `
    -t $Tag `
    $SpikeDir

if ($LASTEXITCODE -ne 0) { throw "Build échoué" }

Write-Host ""
Write-Host "  Envoi vers le registre local..."
docker push $Tag
if ($LASTEXITCODE -ne 0) { throw "Push échoué" }

Write-Host "  Image disponible : $Tag"
Write-Host ""
