# ============================================================
#  AMANE - Ouverture du pare-feu Windows pour le banc de test
#
#  Rend le PC de l'editeur joignable par les autres machines du reseau local :
#  console React, API Django, registre Docker, relais, Grafana.
#
#  A LANCER EN ADMINISTRATEUR :
#    Clic droit sur PowerShell > Executer en tant qu'administrateur, puis :
#    powershell -ExecutionPolicy Bypass -File scripts\ouvrir-pare-feu-lan.ps1
#
#  Pour tout retirer :
#    powershell -ExecutionPolicy Bypass -File scripts\ouvrir-pare-feu-lan.ps1 -Supprimer
# ============================================================

param(
    # Ouvre aussi le profil "Public". A n'utiliser que si le reseau de test
    # est classe Public dans Windows ET qu'il s'agit bien d'un reseau maitrise.
    [switch]$InclurePublic,
    # Retire toutes les regles posees par ce script.
    [switch]$Supprimer
)

$ErrorActionPreference = "Stop"
$PREFIXE = "AMANE"

# Ports publies par docker-compose.yml, avec leur role.
$PORTS = @(
    @{ Port = 5173; Nom = "Console React (point d'entree)" },
    @{ Port = 8000; Nom = "API Django (acces direct)" },
    @{ Port = 5000; Nom = "Registre Docker (images du noeud PME)" },
    @{ Port = 8080; Nom = "Relais zero-knowledge" },
    @{ Port = 3000; Nom = "Grafana" }
)

# --- Verifier l'elevation ------------------------------------------------
$estAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $estAdmin) {
    Write-Host ""
    Write-Host "ERREUR : ce script doit etre lance en administrateur." -ForegroundColor Red
    Write-Host "  Clic droit sur PowerShell > Executer en tant qu'administrateur." -ForegroundColor Yellow
    Write-Host ""
    exit 1
}

# --- Suppression ---------------------------------------------------------
if ($Supprimer) {
    Write-Host ""
    Write-Host "--- Suppression des regles $PREFIXE... ---" -ForegroundColor Yellow
    $regles = Get-NetFirewallRule -DisplayName "$PREFIXE - *" -ErrorAction SilentlyContinue
    if (-not $regles) {
        Write-Host "Aucune regle $PREFIXE trouvee." -ForegroundColor Gray
    } else {
        foreach ($r in $regles) {
            Remove-NetFirewallRule -Name $r.Name
            Write-Host "  Supprimee : $($r.DisplayName)" -ForegroundColor Gray
        }
        Write-Host "OK : $($regles.Count) regle(s) supprimee(s)." -ForegroundColor Green
    }
    Write-Host ""
    exit 0
}

# --- Profils cibles ------------------------------------------------------
$profils = @("Private", "Domain")
if ($InclurePublic) { $profils += "Public" }
$profilsTexte = $profils -join ","

Write-Host ""
Write-Host "====================================================" -ForegroundColor Cyan
Write-Host "  Ouverture du pare-feu - banc de test AMANE" -ForegroundColor Cyan
Write-Host "  Profils : $profilsTexte" -ForegroundColor Cyan
Write-Host "====================================================" -ForegroundColor Cyan
Write-Host ""

# --- Avertir si le reseau actif est classe Public ------------------------
# Une regle posee sur "Private" ne s'applique PAS a une carte classee
# "Public" : c'est la cause no 1 d'un port qui reste injoignable malgre la
# regle. On le signale explicitement plutot que de laisser chercher.
$cartesPublic = Get-NetConnectionProfile | Where-Object { $_.NetworkCategory -eq 'Public' }
if ($cartesPublic -and -not $InclurePublic) {
    Write-Host "AVERTISSEMENT : ces cartes reseau sont classees Public :" -ForegroundColor Yellow
    foreach ($c in $cartesPublic) {
        Write-Host "   - $($c.InterfaceAlias) ($($c.Name))" -ForegroundColor Yellow
    }
    Write-Host ""
    Write-Host "   Les regles posees ici ne s'y appliqueront PAS." -ForegroundColor Yellow
    Write-Host "   Deux solutions :" -ForegroundColor Yellow
    Write-Host "     1. (recommande) Passer le reseau de test en Prive :" -ForegroundColor Yellow
    Write-Host "        Set-NetConnectionProfile -InterfaceAlias '<nom>' -NetworkCategory Private" -ForegroundColor Gray
    Write-Host "     2. Relancer ce script avec -InclurePublic" -ForegroundColor Yellow
    Write-Host ""
}

# --- Poser les regles ----------------------------------------------------
foreach ($p in $PORTS) {
    $nom = "$PREFIXE - $($p.Nom) (TCP $($p.Port))"

    # Idempotent : on retire l'eventuelle regle homonyme avant de la reposer,
    # sinon relancer le script empilerait les doublons.
    $existante = Get-NetFirewallRule -DisplayName $nom -ErrorAction SilentlyContinue
    if ($existante) { $existante | Remove-NetFirewallRule }

    New-NetFirewallRule `
        -DisplayName $nom `
        -Direction Inbound `
        -Action Allow `
        -Protocol TCP `
        -LocalPort $p.Port `
        -Profile $profilsTexte `
        -RemoteAddress LocalSubnet `
        -Description "Banc de test AMANE. Limite au sous-reseau local." | Out-Null

    Write-Host "  OK  TCP $($p.Port)  $($p.Nom)" -ForegroundColor Green
}

# --- Recapitulatif -------------------------------------------------------
# `RemoteAddress LocalSubnet` limite l'acces aux machines du meme sous-reseau :
# le port n'est pas expose au-dela du LAN, meme si la machine est routee.
Write-Host ""
Write-Host "Regles limitees au sous-reseau local (LocalSubnet)." -ForegroundColor Gray
Write-Host ""
Write-Host "--- Adresses par lesquelles ce serveur est joignable ---" -ForegroundColor Cyan
Get-NetIPAddress -AddressFamily IPv4 |
    Where-Object { $_.IPAddress -notlike '127.*' -and $_.IPAddress -notlike '169.254.*' } |
    ForEach-Object {
        Write-Host ("  http://{0}:5173/   ({1})" -f $_.IPAddress, $_.InterfaceAlias) -ForegroundColor Green
    }
Write-Host ""
Write-Host "Depuis une autre machine du reseau, ouvrir l'une de ces adresses." -ForegroundColor Cyan
Write-Host "Pour tout retirer : ...\ouvrir-pare-feu-lan.ps1 -Supprimer" -ForegroundColor Gray
Write-Host ""
