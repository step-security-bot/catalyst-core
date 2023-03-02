import asyncio
from datetime import datetime
import json
import os.path
from typing import Any, List, Optional, Tuple
import rich
import rich.logging
import typer
import marshmallow_dataclass

import ideascale_importer.db
from ideascale_importer.db import models
from ideascale_importer.gvc.client import Client as GvcClient
import ideascale_importer.utils


app = typer.Typer(add_completion=False)


class DbSyncDatabaseConfig:
    host: str
    user: str
    password: str
    db: str


class SnapshotToolConfig:
    path: str
    max_time: datetime


class CatalystToolboxConfig:
    path: str


class GvcConfig:
    api_url: str


class Config:
    output_dir: str
    db_sync_database: DbSyncDatabaseConfig
    snapshot_tool: SnapshotToolConfig
    catalyst_toolbox: CatalystToolboxConfig
    gvc: GvcConfig


DbSyncDatabaseConfigSchema = marshmallow_dataclass.class_schema(DbSyncDatabaseConfig)
SnapshotToolConfigSchema = marshmallow_dataclass.class_schema(SnapshotToolConfig)
CatalystToolboxConfigSchema = marshmallow_dataclass.class_schema(CatalystToolboxConfig)
GvcConfigSchema = marshmallow_dataclass.class_schema(GvcConfig)
ConfigSchema = marshmallow_dataclass.class_schema(Config)


def config_from_json_file(path: str) -> Config:
    """
    Loads configuration from a JSON file.
    """

    with open(path) as f:
        config = ConfigSchema().loads(f.read())
        assert isinstance(config, Config)
        return config


class Contribution:
    reward_address: str
    stake_public_key: str
    value: int


class HIR:
    voting_group: str
    voting_key: str
    voting_power: int


class ProcessedSnapshotEntry:
    contributions: List[Contribution]
    hir: HIR


ContributionSchema = marshmallow_dataclass.class_schema(Contribution)
HIRSchema = marshmallow_dataclass.class_schema(HIR)
ProcessedSnapshotEntrySchema = marshmallow_dataclass.class_schema(ProcessedSnapshotEntry)


class Registration:
    delegations: List[Tuple[str, int]] | str
    reward_address: str
    stake_public_key: str
    voting_power: int
    voting_purpose: int


RegistrationSchema = marshmallow_dataclass.class_schema(Registration)


@app.command(name="import")
def import_snapshot(
    config_path: str = typer.Option(..., help="Path to the configuration file"),
    event_id: int = typer.Option(..., help="Database event id to link all snapshot data to"),
    database_url: str = typer.Option(..., help="URL of the Postgres database in which to import the data to"),
):
    """
    TODO
    """

    async def inner():
        console = rich.console.Console()

        config = config_from_json_file(config_path)

        if not os.path.exists(config.output_dir):
            console.print(f"Output directory does not exist: {config.output_dir}")
            exit(1)

        console.print("Querying max slot parameter")
        conn = await ideascale_importer.db.connect(f"postgres://{config.db_sync_database.user}:{config.db_sync_database.password}@{config.db_sync_database.host}/{config.db_sync_database.db}")

        row = await conn.fetchrow("SELECT slot_no FROM block WHERE time <= $1 ORDER BY time DESC LIMIT 1", config.snapshot_tool.max_time)
        if row is None:
            console.print("Failed to get max slot parameter from db_sync database")
            exit(1)

        max_slot = row["slot_no"]
        console.print(f"Got max_slot = {max_slot} for max_time = \"{config.snapshot_tool.max_time.isoformat()}\"")

        await conn.close()

        snapshot_tool_out_file = os.path.join(config.output_dir, "snapshot_tool_out.json")
        snapshot_tool_cmd = f"{config.snapshot_tool.path} --db-user {config.db_sync_database.user} --db-pass {config.db_sync_database.password} --db-host {config.db_sync_database.host} --db {config.db_sync_database.db} --min-slot 0 --max-slot {max_slot} --out-file {snapshot_tool_out_file}"

        await ideascale_importer.utils.run_cmd(console, "snapshot_tool", snapshot_tool_cmd)

        # REMOVE ME
        snapshot_tool_out_file = "~/Downloads/preprod_old_reward_address.json"

        console.print("Fetching drep list from GVC")
        gvc_client = GvcClient(config.gvc.api_url)

        try:
            with gvc_client.inner.request_progress_observer:
                dreps = await gvc_client.dreps()
        except Exception:
            console.print("Failed to get dreps, using drep cache")
            dreps = []

        min_stake_threshold = 100
        voting_power_cap = 1.2
        catalyst_toolbox_out_file = os.path.join(config.output_dir, "voter_groups.json")
        catalyst_toolbox_cmd = f"{config.catalyst_toolbox.path} snapshot -s {snapshot_tool_out_file} -m {min_stake_threshold} -v {voting_power_cap} --output-format json {catalyst_toolbox_out_file}"

        await ideascale_importer.utils.run_cmd(console, "catalyst-toolbox", catalyst_toolbox_cmd)

        with open(os.path.expanduser(snapshot_tool_out_file)) as f:
            snapshot_tool_data_json = f.read()
        with open(catalyst_toolbox_out_file) as f:
            catalyst_toolbox_data_json = f.read()

        catalyst_toolbox_data: Optional[List[ProcessedSnapshotEntry]] = ProcessedSnapshotEntrySchema().loads(
            catalyst_toolbox_data_json, many=True)

        if catalyst_toolbox_data is None:
            console.print("Failed to load catalyst-toolbox generated data")
            exit(1)

        snapshot_tool_data: Optional[List[Registration]] = RegistrationSchema().loads(
            snapshot_tool_data_json, many=True) or []

        if snapshot_tool_data is None:
            console.print("Failed to load snapshot_tool generated data")
            exit(1)

        snapshot = models.Snapshot(
            event=event_id,
            as_at=datetime.utcnow(),
            last_updated=datetime.utcnow(),
            final=False,
            dbsync_snapshot_cmd=os.path.basename(config.snapshot_tool.path),
            dbsync_snapshot_data=snapshot_tool_data_json,
            drep_data=json.dumps(dreps),
            catalyst_snapshot_cmd=os.path.basename(config.catalyst_toolbox.path),
            catalyst_snapshot_data=catalyst_toolbox_data_json
        )

        conn = await ideascale_importer.db.connect(database_url)

        snapshot_row_id = await ideascale_importer.db.upsert_many(conn, [snapshot], ["event"], returning="row_id")

    asyncio.run(inner())
