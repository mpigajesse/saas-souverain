from django.db import migrations, models


class Migration(migrations.Migration):
    dependencies = [
        ('devices', '0005_device_failover_fields'),
    ]

    operations = [
        migrations.AddField(
            model_name='device',
            name='streaming_standby_count',
            field=models.IntegerField(
                default=0,
                help_text="Nb de standbys réellement en streaming, rapporté par le primaire "
                          "(pg_stat_replication). Source de vérité de la réplication.",
            ),
        ),
    ]
