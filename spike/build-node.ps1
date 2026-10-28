# ============================================================
#  EL BARAA CONSULT — Build & publication image ss-node
#  A lancer depuis n'importe quel repertoire :
#    powershell -ExecutionPolicy Bypass -File spike\build-node.ps1
#    powershell -ExecutionPolicy Bypass -File spike\build-node.ps1 -NoCache
#    powershell -ExecutionPolicy Bypass -File spike\build-node.ps1 -SkipPush
#    powershell -ExecutionPolicy Bypass -File spike\build-node.ps1 -Version v3 -Registry localhost:5000
#
#  Etapes : verifier Docker -> build depuis spike/ -> tagger (v, latest,
#  registre) -> verifier le binaire -> push -> docker save + gzip + sha256
#  -> ecrire media/node-images/manifeste.json
# ============================================================

param(
    [switch]$NoCache,
    [switch]$SkipPush,
    [string]$Version  = "v2",
    [string]$Registry = "localhost:5000"
)

$ErrorActionPreference = "Stop"

# Chemins derives de l'emplacement du script (fonctionne quel que soit le
# repertoire courant depuis lequel il est lance)
$SpikeDir = $PSScriptRoot
$RepoRoot = Split-Path -Parent $SpikeDir
$MediaDir = Join-Path $RepoRoot "media\node-images"

$LocalTag    = "ss-node:$Version"
$LatestTag   = "ss-node:latest"
$RegistryTag = "$Registry/ss-node:$Version"

Write-Host ""
Write-Host "====================================================" -ForegroundColor Cyan
Write-Host "  Build image : $LocalTag" -ForegroundColor Cyan
Write-Host "  Registre    : $RegistryTag" -ForegroundColor Cyan
Write-Host "====================================================" -ForegroundColor Cyan
Write-Host ""

# --- 1. Verifier que Docker tourne --------------------------------------
$dockerRunning = docker info --format "{{.ServerVersion}}" 2>$null
if (-not $dockerRunning) {
    Write-Host "ERREUR : Docker n'est pas demarre." -ForegroundColor Red
    exit 1
}

# --- 2. Build depuis la racine du workspace spike/ (requis pour Cargo workspace) ---
Push-Location $SpikeDir
try {
    $buildArgs = @("build", "-f", "node/Dockerfile", "-t", $LocalTag)
    if ($NoCache) {
        $buildArgs += "--no-cache"
        Write-Host "Mode : --no-cache (reconstruction complete)" -ForegroundColor Yellow
    }
    $buildArgs += "."

    Write-Host "Commande : docker $($buildArgs -join ' ')" -ForegroundColor Gray
    Write-Host ""

    docker @buildArgs
    if ($LASTEXITCODE -ne 0) {
        Write-Host ""
        Write-Host "ERREUR : Build echoue (code $LASTEXITCODE)" -ForegroundColor Red
        exit $LASTEXITCODE
    }
} finally {
    Pop-Location
}

# --- 3. Tagger --------------------------------------------------------
Write-Host ""
Write-Host "--- Tag des images... ---" -ForegroundColor Yellow
docker tag $LocalTag $LatestTag
if ($LASTEXITCODE -ne 0) { Write-Host "ERREUR : tag $LatestTag echoue" -ForegroundColor Red; exit $LASTEXITCODE }
docker tag $LocalTag $RegistryTag
if ($LASTEXITCODE -ne 0) { Write-Host "ERREUR : tag $RegistryTag echoue" -ForegroundColor Red; exit $LASTEXITCODE }
Write-Host "OK : $LocalTag, $LatestTag, $RegistryTag" -ForegroundColor Green

# --- 4. Verifier que le binaire tourne ---------------------------------
Write-Host ""
Write-Host "--- Verification du binaire (ss-node --help)... ---" -ForegroundColor Yellow
docker run --rm --entrypoint ss-node $LocalTag --help
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERREUR : le binaire ss-node ne repond pas correctement" -ForegroundColor Red
    exit $LASTEXITCODE
}
Write-Host "OK : binaire fonctionnel" -ForegroundColor Green

# --- 5. Push vers le registre -------------------------------------------
if ($SkipPush) {
    Write-Host ""
    Write-Host "Push ignore (-SkipPush)" -ForegroundColor Yellow
} else {
    Write-Host ""
    Write-Host "--- Push vers le registre $Registry... ---" -ForegroundColor Yellow
    docker push $RegistryTag
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERREUR : Push echoue" -ForegroundColor Red
        exit $LASTEXITCODE
    }
    Write-Host "OK : image publiee sur $RegistryTag" -ForegroundColor Green
}

# --- 6. Archive telechargeable (docker save + gzip + sha256) -----------
Write-Host ""
Write-Host "--- Generation de l'archive telechargeable... ---" -ForegroundColor Yellow

if (-not (Test-Path $MediaDir)) {
    New-Item -ItemType Directory -Force -Path $MediaDir | Out-Null
}

$TarPath    = Join-Path $MediaDir "ss-node-$Version.tar"
$GzPath     = "$TarPath.gz"
$ArchiveName = "ss-node-$Version.tar.gz"
$ManifestPath = Join-Path $MediaDir "manifeste.json"

if (Test-Path $TarPath)  { Remove-Item -Force $TarPath }
if (Test-Path $GzPath)   { Remove-Item -Force $GzPath }

docker save $LocalTag -o $TarPath
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERREUR : docker save echoue" -ForegroundColor Red
    exit $LASTEXITCODE
}

# Compression gzip (utilise l'utilitaire gzip s'il est present, sinon .NET)
$gzipCmd = Get-Command gzip -ErrorAction SilentlyContinue
if ($gzipCmd) {
    & gzip -f $TarPath
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERREUR : compression gzip echouee" -ForegroundColor Red
        exit $LASTEXITCODE
    }
} else {
    Add-Type -AssemblyName System.IO.Compression
    $inStream  = [System.IO.File]::OpenRead($TarPath)
    $outStream = [System.IO.File]::Create($GzPath)
    $gzStream  = New-Object System.IO.Compression.GZipStream($outStream, [System.IO.Compression.CompressionMode]::Compress)
    $inStream.CopyTo($gzStream)
    $gzStream.Dispose()
    $outStream.Dispose()
    $inStream.Dispose()
    Remove-Item -Force $TarPath
}

if (-not (Test-Path $GzPath)) {
    Write-Host "ERREUR : archive compressee introuvable ($GzPath)" -ForegroundColor Red
    exit 1
}

$fileInfo = Get-Item $GzPath
$tailleOctets = $fileInfo.Length
$sha256 = (Get-FileHash -Path $GzPath -Algorithm SHA256).Hash.ToLower()
$construitLe = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

Write-Host "OK : $GzPath ($tailleOctets octets, sha256=$sha256)" -ForegroundColor Green

# --- 7. Manifeste --------------------------------------------------------
$manifest = [ordered]@{
    version        = $Version
    image          = $LocalTag
    tag_registre   = $LocalTag
    fichier        = $ArchiveName
    taille_octets  = $tailleOctets
    sha256         = $sha256
    construit_le   = $construitLe
}
$manifestJson = $manifest | ConvertTo-Json
# Ecriture sans BOM (Set-Content -Encoding utf8 ajoute un BOM en PowerShell 5.1,
# ce qui casse certains parseurs JSON stricts consommant ce manifeste)
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllText($ManifestPath, $manifestJson, $utf8NoBom)

Write-Host "OK : manifeste ecrit dans $ManifestPath" -ForegroundColor Green

# --- Recapitulatif ---------------------------------------------------------
Write-Host ""
Write-Host "====================================================" -ForegroundColor Green
Write-Host "  Termine" -ForegroundColor Green
Write-Host "  Images   : $LocalTag, $LatestTag, $RegistryTag" -ForegroundColor Green
Write-Host "  Archive  : $GzPath" -ForegroundColor Green
if (-not $SkipPush) {
    Write-Host "  Sur les VMs PME : docker compose pull && docker compose up -d" -ForegroundColor Green
}
Write-Host "====================================================" -ForegroundColor Green
Write-Host ""
