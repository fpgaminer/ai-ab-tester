#!/usr/bin/env python3
import click
import requests
from urllib.parse import urljoin

@click.group()
def cli():
	pass

@cli.command()
@click.argument('url')
@click.argument('admin_key')
def new_project(url, admin_key):
	resp = requests.post(urljoin(url, "admin/new_project"), headers={"Authorization": "Bearer " + admin_key})
	resp.raise_for_status()
	resp = resp.json()
	click.echo(f"Project URL: {url}#{resp['project_id']}")
	click.echo(f"Project Admin Token: {resp['admin_token']}")


@cli.command()
@click.argument('url')
@click.argument('project_id')
def get_samples(url, project_id):
	resp = requests.get(urljoin(url, "project/get_samples"), headers={"Authorization": "Bearer " + project_id})
	resp.raise_for_status()
	click.echo(resp.json())


@cli.command()
@click.argument('url')
@click.argument('project_id')
def get_ratings(url, project_id):
	resp = requests.get(urljoin(url, "project/get_ratings"), headers={"Authorization": "Bearer " + project_id})
	resp.raise_for_status()
	click.echo(resp.json())


if __name__ == "__main__":
	cli()