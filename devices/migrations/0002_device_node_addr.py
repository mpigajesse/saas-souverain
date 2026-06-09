from django.db import migrations, models


class Migration(migrations.Migration):

    dependencies = [
        ('devices', '0001_initial'),
    ]

    operations = [
        migrations.AddField(
            model_name='device',
            name='node_addr',
            field=models.CharField(
                blank=True,
                max_length=255,
                help_text='IP:port annoncée par le nœud (ex: 192.168.200.130:9001)',
            ),
        ),
    ]
