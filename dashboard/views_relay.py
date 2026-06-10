import json
import urllib.request
import urllib.error
from datetime import datetime, timezone

from django.contrib.admin.views.decorators import staff_member_required
from django.conf import settings
from django.shortcuts import render

from tenants.models import Tenant


def _fetch_json(url: str, timeout: int = 5) -> dict:
    with urllib.request.urlopen(url, timeout=timeout) as resp:
        return json.loads(resp.read())


def _time_ago(iso: str) -> str:
    try:
        dt = datetime.fromisoformat(iso.replace("Z", "+00:00"))
        delta = datetime.now(timezone.utc) - dt
        s = int(delta.total_seconds())
        if s < 60:
            return f"{s}s"
        if s < 3600:
            return f"{s // 60}min"
        return f"{s // 3600}h"
    except Exception:
        return "?"


def _format_uptime(secs: int) -> str:
    if secs < 60:
        return f"{secs}s"
    if secs < 3600:
        return f"{secs // 60}min {secs % 60}s"
    h = secs // 3600
    m = (secs % 3600) // 60
    return f"{h}h {m}min"


@staff_member_required
def relay_monitor(request):
    relay_url = getattr(settings, "RELAY_URL", "http://localhost:8080").rstrip("/")

    # ── Health ────────────────────────────────────────────────
    health = {
        "reachable": False,
        "status": "unreachable",
        "version": "—",
        "hostname": "—",
        "uptime_secs": 0,
        "uptime_fmt": "—",
        "port": "8080",
        "blobs_dir": "—",
        "blob_tenants": 0,
        "networks": [],
        "zero_knowledge": True,
    }
    try:
        data = _fetch_json(f"{relay_url}/health")
        uptime = data.get("uptime_secs", 0)
        health = {
            **data,
            "reachable": True,
            "uptime_fmt": _format_uptime(uptime),
        }
    except Exception as e:
        health["error"] = str(e)

    # ── Nœuds par tenant ─────────────────────────────────────
    tenants_data = []
    total_nodes = 0

    for tenant in Tenant.objects.order_by("name"):
        nodes = []
        try:
            result = _fetch_json(f"{relay_url}/api/nodes?tenant_id={tenant.id}")
            nodes = [
                {
                    **n,
                    "time_ago": _time_ago(n.get("last_seen", "")),
                    "short_id": n.get("node_id", "")[:8],
                }
                for n in result.get("nodes", [])
            ]
        except Exception:
            pass

        total_nodes += len(nodes)
        tenants_data.append({"tenant": tenant, "nodes": nodes})

    return render(request, "dashboard/relay.html", {
        "relay_url": relay_url,
        "health": health,
        "tenants_data": tenants_data,
        "total_nodes": total_nodes,
    })
