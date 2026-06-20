from django.urls import path

from . import views_web

urlpatterns = [
    path('', views_web.device_list, name='device-list'),
    path('clusters/', views_web.cluster_overview, name='cluster-overview'),
    path('clusters/<uuid:tenant_pk>/ai-analyze/', views_web.cluster_ai_analyze, name='cluster-ai-analyze'),
    path('agent/', views_web.agent_monitoring, name='agent-monitoring'),
    path('agent/live/', views_web.agent_live, name='agent-live'),
    path('agent/host-analyze/', views_web.agent_host_analyze, name='agent-host-analyze'),
    path('agent/relay-analyze/', views_web.agent_relay_analyze, name='agent-relay-analyze'),
    path('agent/analyze-all/', views_web.agent_analyze_all, name='agent-analyze-all'),
    path('agent/report/', views_web.agent_report_download, name='agent-report-download'),
    path('agent/report.pdf', views_web.agent_report_pdf, name='agent-report-pdf'),
    path('<uuid:pk>/toggle/', views_web.device_toggle, name='device-toggle'),
]
