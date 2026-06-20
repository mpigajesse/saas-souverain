from datetime import timedelta

from django.conf import settings
from django.contrib import messages
from django.contrib.auth.decorators import login_required
from django.http import HttpResponse, JsonResponse
from django.shortcuts import get_object_or_404, redirect, render
from django.utils import timezone

from tenants.models import Tenant
from .models import AgentVerdict, Device

RISK_LABELS = {
    'sain': 'Sain',
    'surveiller': 'À surveiller',
    'critique': 'Critique',
}

ONLINE_THRESHOLD = timedelta(minutes=2)


def compute_cluster_snapshot(tenant, threshold):
    """Calcule l'état de supervision d'un cluster pour un tenant donné.

    Réutilisé par la vue d'ensemble et par l'endpoint d'analyse IA.
    """
    devices = tenant.devices.filter(is_active=True)
    primary = devices.filter(node_role='primary').first()
    standbys = list(devices.filter(node_role='standby'))
    license_status = tenant.licence_status

    def enrich(d):
        return {
            'device': d,
            'online': d.last_seen >= threshold if d.last_seen else False,
        }

    primary_info = enrich(primary) if primary else None
    standbys_info = [enrich(s) for s in standbys]

    # Vérité de réplication : nb de standbys réellement en streaming, mesuré
    # par le primaire (pg_stat_replication). Un standby simplement enregistré
    # et "en ligne" ne prouve PAS que la réplication WAL fonctionne.
    streaming = primary.streaming_standby_count if primary else 0

    total = devices.count()
    online_count = devices.filter(last_seen__gte=threshold).count()

    if total == 0:
        health = 'empty'
    elif primary is None:
        health = 'no_primary'
    elif not (primary_info and primary_info['online']):
        health = 'primary_offline'
    elif len(standbys) == 0:
        health = 'no_standby'
    elif streaming == 0:
        # Un standby existe mais aucun flux WAL → split-brain ou conninfo périmé.
        health = 'replication_down'
    elif not license_status['ok']:
        health = 'over_license'
    else:
        health = 'ok'

    is_alert = health not in ('ok', 'empty')

    return {
        'tenant': tenant,
        'primary': primary_info,
        'standbys': standbys_info,
        'streaming': streaming,
        'total': total,
        'online_count': online_count,
        'license': license_status,
        'health': health,
        'is_alert': is_alert,
    }


@login_required
def device_list(request):
    devices = Device.objects.select_related('tenant').order_by('-last_seen')
    return render(request, 'devices/list.html', {'devices': devices})


@login_required
def cluster_overview(request):
    """Supervision des clusters PME — vue par tenant avec santé et licence."""
    threshold = timezone.now() - ONLINE_THRESHOLD

    tenants = (
        Tenant.objects
        .prefetch_related('devices', 'licenses')
        .filter(is_active=True)
        .order_by('name')
    )

    clusters = [compute_cluster_snapshot(t, threshold) for t in tenants]
    alerts_total = sum(1 for c in clusters if c['is_alert'])

    # Trier : alertes en premier
    clusters.sort(key=lambda c: (0 if c['is_alert'] else 1, c['tenant'].name))

    return render(request, 'devices/clusters.html', {
        'clusters': clusters,
        'alerts_total': alerts_total,
        'total_tenants': len(clusters),
    })


def _run_and_store(tenant, snapshot):
    """Lance l'agent sur un cluster, persiste le verdict, et le renvoie."""
    from .ai_monitor import analyze_with_features

    verdict, features = analyze_with_features(tenant, snapshot)
    AgentVerdict.objects.create(
        tenant=tenant,
        health=snapshot['health'],
        risk_level=verdict.risk_level,
        anomaly_score=verdict.anomaly_score,
        summary=verdict.summary,
        recommendation=verdict.recommendation,
        details=verdict.details,
        source=verdict.source,
        features=features,
    )
    return verdict, features


@login_required
def cluster_ai_analyze(request, tenant_pk):
    """Analyse IA à la demande d'un cluster — renvoie un verdict JSON.

    Appelé en AJAX depuis le dashboard (bouton « Analyse IA »). L'appel au
    modèle Mistral est fait ici, à la demande, jamais à chaque heartbeat.
    """
    tenant = get_object_or_404(Tenant, pk=tenant_pk, is_active=True)
    threshold = timezone.now() - ONLINE_THRESHOLD
    snapshot = compute_cluster_snapshot(tenant, threshold)

    verdict, _features = _run_and_store(tenant, snapshot)
    return JsonResponse({
        'tenant': tenant.name,
        'health': snapshot['health'],
        **verdict.as_dict(),
    })


@login_required
def agent_monitoring(request):
    """Page « Agent de supervision » — vue consolidée de tout ce que l'agent
    surveille : métriques techniques + dernier verdict par cluster.
    """
    threshold = timezone.now() - ONLINE_THRESHOLD
    tenants = (
        Tenant.objects
        .prefetch_related('devices', 'licenses', 'agent_verdicts')
        .filter(is_active=True)
        .order_by('name')
    )

    rows = []
    counts = {'sain': 0, 'surveiller': 0, 'critique': 0, 'unknown': 0}
    for tenant in tenants:
        snapshot = compute_cluster_snapshot(tenant, threshold)
        last = tenant.agent_verdicts.first()  # ordering = -created_at
        if last:
            counts[last.risk_level] = counts.get(last.risk_level, 0) + 1
        else:
            counts['unknown'] += 1
        rows.append({
            'tenant': tenant,
            'snapshot': snapshot,
            'verdict': last,
        })

    # Trier : critiques d'abord, puis à surveiller, puis sains/inconnus
    risk_order = {'critique': 0, 'surveiller': 1, 'sain': 2}
    rows.sort(key=lambda r: risk_order.get(r['verdict'].risk_level if r['verdict'] else '', 3))

    last_verdict = AgentVerdict.objects.order_by('-created_at').first()

    from .host_monitor import collect_host_metrics, host_health
    host = collect_host_metrics()
    host['health'] = host_health(host)

    return render(request, 'devices/agent_monitoring.html', {
        'rows': rows,
        'counts': counts,
        'total': len(rows),
        'mistral_active': bool(getattr(settings, 'MISTRAL_API_KEY', '')),
        'last_run': last_verdict.created_at if last_verdict else None,
        'host': host,
    })


@login_required
def agent_live(request):
    """Flux temps réel (polling JSON) : état du serveur éditeur + clusters.

    Léger et sans appel IA — destiné à être interrogé toutes les quelques
    secondes par le dashboard pour animer la supervision en direct.
    """
    from .host_monitor import collect_host_metrics, host_health

    metrics = collect_host_metrics()
    threshold = timezone.now() - ONLINE_THRESHOLD

    tenants = Tenant.objects.filter(is_active=True).order_by('name')
    clusters = []
    counts = {'sain': 0, 'surveiller': 0, 'critique': 0}
    for tenant in tenants:
        snap = compute_cluster_snapshot(tenant, threshold)
        # Mapping santé technique → niveau de risque pour l'affichage live
        if snap['health'] in ('replication_down', 'no_primary', 'primary_offline'):
            level = 'critique'
        elif snap['health'] in ('no_standby', 'over_license'):
            level = 'surveiller'
        elif snap['health'] == 'ok':
            level = 'sain'
        else:
            level = 'sain' if snap['total'] == 0 else 'surveiller'
        counts[level] = counts.get(level, 0) + 1
        clusters.append({
            'name': tenant.name,
            'health': snap['health'],
            'level': level,
            'total': snap['total'],
            'online': snap['online_count'],
            'streaming': snap['streaming'],
        })

    return JsonResponse({
        'ts': metrics['captured_at'],
        'host': {**metrics, 'health': host_health(metrics)},
        'clusters': clusters,
        'counts': counts,
    })


@login_required
def agent_host_analyze(request):
    """Diagnostic IA à la demande du serveur éditeur (JSON)."""
    from .ai_monitor import analyze_host
    from .host_monitor import collect_host_metrics

    metrics = collect_host_metrics()
    verdict = analyze_host(metrics)
    return JsonResponse({'metrics': metrics, **verdict.as_dict()})


@login_required
def agent_analyze_all(request):
    """Lance l'agent sur tous les clusters actifs et persiste les verdicts."""
    if request.method != 'POST':
        return redirect('agent-monitoring')

    threshold = timezone.now() - ONLINE_THRESHOLD
    tenants = Tenant.objects.filter(is_active=True).order_by('name')

    analyzed = 0
    critiques = 0
    for tenant in tenants:
        snapshot = compute_cluster_snapshot(tenant, threshold)
        if snapshot['total'] == 0:
            continue  # rien à analyser pour un tenant sans nœud
        verdict, _ = _run_and_store(tenant, snapshot)
        analyzed += 1
        if verdict.risk_level == 'critique':
            critiques += 1

    if analyzed == 0:
        messages.info(request, "Aucun cluster avec des nœuds à analyser.")
    elif critiques:
        messages.warning(request, f"Analyse terminée : {analyzed} cluster(s) · {critiques} en état critique.")
    else:
        messages.success(request, f"Analyse terminée : {analyzed} cluster(s) analysé(s).")
    return redirect('agent-monitoring')


@login_required
def agent_report_download(request):
    """Génère un rapport Markdown téléchargeable des derniers verdicts."""
    now = timezone.now()
    tenants = (
        Tenant.objects
        .prefetch_related('agent_verdicts')
        .filter(is_active=True)
        .order_by('name')
    )

    lines = [
        "# Rapport de supervision — Agent IA",
        "",
        f"*Généré le {now:%d/%m/%Y à %H:%M} (UTC) · SaaS Souverain*",
        "",
        "> Les métriques analysées sont strictement techniques (infrastructure).",
        "> Aucune donnée métier n'est traitée — promesse zero-knowledge préservée.",
        "",
        "---",
        "",
    ]

    for tenant in tenants:
        v = tenant.agent_verdicts.first()
        lines.append(f"## {tenant.name}")
        lines.append("")
        if not v:
            lines.append("_Aucune analyse enregistrée pour ce cluster._")
            lines.append("")
            lines.append("---")
            lines.append("")
            continue

        lines.append(f"- **Niveau de risque** : {RISK_LABELS.get(v.risk_level, v.risk_level)}")
        lines.append(f"- **Score d'anomalie** : {v.anomaly_score}/100")
        lines.append(f"- **Santé cluster** : {v.health}")
        lines.append(f"- **Source** : {v.get_source_display()}")
        lines.append(f"- **Analysé le** : {v.created_at:%d/%m/%Y %H:%M} UTC")
        lines.append("")
        lines.append(f"**Diagnostic** : {v.summary}")
        lines.append("")
        lines.append(f"**Recommandation** : {v.recommendation}")
        lines.append("")
        if v.features:
            lines.append("**Métriques analysées :**")
            lines.append("")
            lines.append("| Métrique | Valeur |")
            lines.append("|----------|--------|")
            for k, val in v.features.items():
                lines.append(f"| {k} | {val} |")
            lines.append("")
        lines.append("---")
        lines.append("")

    content = "\n".join(lines)
    filename = f"rapport_agent_supervision_{now:%Y%m%d_%H%M}.md"
    response = HttpResponse(content, content_type='text/markdown; charset=utf-8')
    response['Content-Disposition'] = f'attachment; filename="{filename}"'
    return response


@login_required
def agent_report_pdf(request):
    """Génère un rapport PDF des derniers verdicts + état du serveur éditeur."""
    import io

    from reportlab.lib import colors
    from reportlab.lib.enums import TA_LEFT
    from reportlab.lib.pagesizes import A4
    from reportlab.lib.styles import ParagraphStyle, getSampleStyleSheet
    from reportlab.lib.units import mm
    from reportlab.platypus import (
        Paragraph, SimpleDocTemplate, Spacer, Table, TableStyle,
    )

    from .ai_monitor import analyze_host
    from .host_monitor import collect_host_metrics

    now = timezone.now()
    NAVY = colors.HexColor('#003B71')
    ACCENT = colors.HexColor('#0072B5')
    RISK_COLORS = {
        'critique': colors.HexColor('#C0392B'),
        'surveiller': colors.HexColor('#B45309'),
        'sain': colors.HexColor('#2E7D32'),
    }

    styles = getSampleStyleSheet()
    h1 = ParagraphStyle('h1', parent=styles['Title'], textColor=NAVY, fontSize=20, spaceAfter=4)
    sub = ParagraphStyle('sub', parent=styles['Normal'], textColor=colors.grey, fontSize=9, spaceAfter=2)
    h2 = ParagraphStyle('h2', parent=styles['Heading2'], textColor=NAVY, fontSize=13, spaceBefore=10, spaceAfter=4)
    body = ParagraphStyle('body', parent=styles['Normal'], fontSize=9.5, leading=13, alignment=TA_LEFT)
    note = ParagraphStyle('note', parent=styles['Normal'], fontSize=8, textColor=colors.grey, leading=11)

    buf = io.BytesIO()
    doc = SimpleDocTemplate(
        buf, pagesize=A4,
        leftMargin=18 * mm, rightMargin=18 * mm, topMargin=16 * mm, bottomMargin=16 * mm,
        title='Rapport de supervision — Agent IA',
    )
    story = []

    from .metrics_catalog import explain_cluster, explain_host

    STATE_FILL = {
        'sain': colors.HexColor('#E6F4EA'),
        'surveiller': colors.HexColor('#FEF3C7'),
        'critique': colors.HexColor('#FDE8E6'),
        'info': colors.HexColor('#F4F6F9'),
    }
    STATE_TXT = {
        'sain': colors.HexColor('#2E7D32'),
        'surveiller': colors.HexColor('#B45309'),
        'critique': colors.HexColor('#C0392B'),
        'info': colors.grey,
    }
    cell = ParagraphStyle('cell', parent=styles['Normal'], fontSize=8, leading=10.5)
    cellb = ParagraphStyle('cellb', parent=cell, fontName='Helvetica-Bold')

    story.append(Paragraph('Rapport de supervision — Agent IA', h1))
    story.append(Paragraph(f"Généré le {now:%d/%m/%Y à %H:%M} (UTC) · SaaS Souverain", sub))
    story.append(Spacer(1, 6))

    # ── Méthodologie ─────────────────────────────────────────────────────────
    story.append(Paragraph('Ce que supervise l\'agent', h2))
    story.append(Paragraph(
        "L'agent de supervision analyse en continu deux périmètres : (1) le <b>serveur "
        "éditeur SaaS</b> qui héberge les comptes, licences et le suivi du parc ; et "
        "(2) les <b>clusters PostgreSQL</b> de chaque PME (réplication primaire/standby). "
        "Pour chaque périmètre, il évalue des métriques techniques par rapport à des seuils, "
        "puis attribue un niveau de risque (Normal, À surveiller, Critique) et un score "
        "d'anomalie sur 100, accompagnés d'un diagnostic et d'une recommandation.", body))
    story.append(Paragraph(
        "Garantie zero-knowledge : seules des métriques d'infrastructure sont traitées. "
        "Aucune donnée métier (stock, factures, clients) n'est lue ni transmise.", note))
    story.append(Spacer(1, 6))

    def state_cell(state, label):
        return Table(
            [[Paragraph(f"<font color='#{STATE_TXT[state].hexval()[2:]}'><b>{label}</b></font>", cell)]],
            colWidths=[24 * mm],
            style=TableStyle([
                ('BACKGROUND', (0, 0), (-1, -1), STATE_FILL[state]),
                ('TOPPADDING', (0, 0), (-1, -1), 3), ('BOTTOMPADDING', (0, 0), (-1, -1), 3),
                ('LEFTPADDING', (0, 0), (-1, -1), 5), ('RIGHTPADDING', (0, 0), (-1, -1), 5),
            ]),
        )

    def verdict_block(level, score, summary, recommendation, details):
        c = RISK_COLORS.get(level, colors.grey)
        story.append(Paragraph(
            f"Niveau de risque : <font color='#{c.hexval()[2:]}'><b>{RISK_LABELS.get(level, level)}</b></font> "
            f"· Score d'anomalie : <b>{score}/100</b>", body))
        story.append(Paragraph(f"<b>Diagnostic :</b> {summary}", body))
        if details:
            story.append(Paragraph(f"<b>Analyse détaillée de l'agent :</b> {details}", body))
        story.append(Paragraph(f"<b>Recommandation :</b> {recommendation}", body))

    def detail_table(rows, with_threshold):
        if with_threshold:
            header = ['Métrique', 'Valeur', 'Seuil', 'État', 'Signification']
            widths = [33 * mm, 22 * mm, 30 * mm, 20 * mm, 69 * mm]
        else:
            header = ['Métrique', 'Valeur', 'État', 'Signification']
            widths = [38 * mm, 20 * mm, 20 * mm, 96 * mm]
        data = [[Paragraph(f"<b>{h}</b>", cellb) for h in header]]
        styl = [
            ('BACKGROUND', (0, 0), (-1, 0), NAVY),
            ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
            ('GRID', (0, 0), (-1, -1), 0.4, colors.HexColor('#D8DEE6')),
            ('VALIGN', (0, 0), (-1, -1), 'MIDDLE'),
            ('TOPPADDING', (0, 0), (-1, -1), 4), ('BOTTOMPADDING', (0, 0), (-1, -1), 4),
            ('LEFTPADDING', (0, 0), (-1, -1), 5), ('RIGHTPADDING', (0, 0), (-1, -1), 5),
        ]
        for i, r in enumerate(rows, start=1):
            stt = STATE_TXT[r['state']]
            state_par = Paragraph(f"<font color='#{stt.hexval()[2:]}'><b>{r['state_label']}</b></font>", cell)
            if with_threshold:
                row = [Paragraph(r['label'], cellb), Paragraph(r['value'], cell),
                       Paragraph(r['threshold'], cell), state_par, Paragraph(r['meaning'], cell)]
            else:
                row = [Paragraph(r['label'], cellb), Paragraph(r['value'], cell),
                       state_par, Paragraph(r['meaning'], cell)]
            data.append(row)
            styl.append(('BACKGROUND', (0, i), (-1, i),
                         colors.white if i % 2 else colors.HexColor('#F7F9FB')))
        t = Table(data, colWidths=widths, repeatRows=1)
        t.setStyle(TableStyle(styl))
        return t

    # ── Serveur éditeur ──────────────────────────────────────────────────────
    story.append(Paragraph('1. Serveur éditeur SaaS', h2))
    metrics = collect_host_metrics()
    host_verdict = analyze_host(metrics)
    verdict_block(host_verdict.risk_level, host_verdict.anomaly_score,
                  host_verdict.summary, host_verdict.recommendation, host_verdict.details)
    story.append(Paragraph(
        f"Source de l'analyse : {('Mistral AI' if host_verdict.source == 'mistral' else 'Analyse locale')}", note))
    story.append(Spacer(1, 4))
    story.append(Paragraph("Métriques analysées :", body))
    story.append(detail_table(explain_host(metrics), with_threshold=True))
    story.append(Spacer(1, 10))

    # ── Clusters PME ─────────────────────────────────────────────────────────
    story.append(Paragraph('2. Clusters PME', h2))
    tenants = (
        Tenant.objects.prefetch_related('agent_verdicts')
        .filter(is_active=True).order_by('name')
    )
    any_cluster = False
    for tenant in tenants:
        v = tenant.agent_verdicts.first()
        if not v:
            continue
        any_cluster = True
        story.append(Spacer(1, 6))
        story.append(Paragraph(f"<b>{tenant.name}</b> — état technique : {v.health}", body))
        verdict_block(v.risk_level, v.anomaly_score, v.summary, v.recommendation, v.details)
        story.append(Paragraph(
            f"Source : {v.get_source_display()} · Analysé le {v.created_at:%d/%m/%Y %H:%M} UTC", note))
        if v.features:
            story.append(Spacer(1, 3))
            story.append(Paragraph("Métriques analysées :", body))
            story.append(detail_table(explain_cluster(v.features), with_threshold=False))

    if not any_cluster:
        story.append(Paragraph(
            "Aucun cluster n'a encore été analysé. Lancez « Analyser tous les clusters » "
            "depuis la page Agent IA.", note))

    doc.build(story)
    buf.seek(0)
    filename = f"rapport_agent_supervision_{now:%Y%m%d_%H%M}.pdf"
    response = HttpResponse(buf.getvalue(), content_type='application/pdf')
    response['Content-Disposition'] = f'attachment; filename="{filename}"'
    return response


@login_required
def device_toggle(request, pk):
    """Activer ou désactiver une machine du parc."""
    device = get_object_or_404(Device, pk=pk)
    if request.method == 'POST':
        device.is_active = not device.is_active
        device.save(update_fields=['is_active'])
        etat = 'activée' if device.is_active else 'désactivée'
        if not device.is_active:
            messages.warning(
                request,
                f'Machine « {device.hostname or str(device.installation_id)[:8]} » désactivée. '
                f'⚠ Pensez à effectuer une rotation de la DEK côté cluster PME.'
            )
        else:
            messages.success(request, f'Machine « {device.hostname or str(device.installation_id)[:8]} » réactivée.')
        next_url = request.POST.get('next', 'device-list')
        return redirect(next_url)
    return redirect('device-list')
