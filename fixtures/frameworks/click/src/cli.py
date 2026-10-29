import click

@click.command()
@click.option("--name", default="World")
def greet(name):
    click.echo(f"Hello {name}")

if __name__ == "__main__":
    greet()
